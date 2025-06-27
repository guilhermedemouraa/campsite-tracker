use chrono::{Datelike, NaiveDate, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use tracing::{debug, warn};

use crate::executor::{CampgroundAvailability, SiteAvailability};
use crate::scan_types::ScanError;

/// Client for interacting with recreation.gov API
pub struct RecGovClient {
    client: Client,
    ridb_base_url: String,
    internal_base_url: String,
    api_key: Option<String>,
}

/// Response structure from recreation.gov campsite availability API
#[derive(Debug, Deserialize)]
pub struct RecGovAvailabilityResponse {
    pub count: i32,
    #[serde(rename = "RECDATA")]
    pub rec_data: Vec<RecGovCampsite>,
}

/// Individual campsite data from recreation.gov
#[derive(Debug, Deserialize)]
pub struct RecGovCampsite {
    #[serde(rename = "CampsiteID")]
    pub campsite_id: String,

    #[serde(rename = "CampsiteName")]
    pub campsite_name: Option<String>,

    #[serde(rename = "CampsiteType")]
    pub campsite_type: Option<String>,

    #[serde(rename = "TypeOfUse")]
    pub type_of_use: Option<String>,

    #[serde(rename = "Loop")]
    pub campsite_loop: Option<String>,

    #[serde(rename = "CampsiteAccessible")]
    pub accessible: Option<String>,

    #[serde(rename = "CampsiteLatitude")]
    pub latitude: Option<f64>,

    #[serde(rename = "CampsiteLongitude")]
    pub longitude: Option<f64>,

    #[serde(rename = "Availabilities")]
    pub availabilities: Option<HashMap<String, String>>,
}

/// Facility search response from recreation.gov
#[derive(Debug, Deserialize)]
pub struct RecGovFacilityResponse {
    #[serde(rename = "RECDATA")]
    pub rec_data: Vec<RecGovFacility>,
}

#[derive(Debug, Deserialize)]
pub struct RecGovFacility {
    #[serde(rename = "FacilityID")]
    pub facility_id: String,

    #[serde(rename = "FacilityName")]
    pub facility_name: String,

    #[serde(rename = "FacilityDescription")]
    pub description: Option<String>,

    #[serde(rename = "FacilityLatitude")]
    pub latitude: Option<f64>,

    #[serde(rename = "FacilityLongitude")]
    pub longitude: Option<f64>,

    #[serde(rename = "FacilityPhone")]
    pub phone: Option<String>,

    #[serde(rename = "FacilityEmail")]
    pub email: Option<String>,

    #[serde(rename = "AddressStateCode")]
    pub state_code: Option<String>,
}

impl RecGovClient {
    /// Create a new recreation.gov API client
    pub fn new(api_key: Option<String>) -> Result<Self, ScanError> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| ScanError::ApiError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            ridb_base_url: "https://ridb.recreation.gov/api/v1".to_string(),
            internal_base_url: "https://www.recreation.gov/api".to_string(),
            api_key,
        })
    }

    /// Get campground availability for a date range
    pub async fn get_campground_availability(
        &self,
        facility_id: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<CampgroundAvailability, ScanError> {
        debug!(
            "Fetching availability for facility {} from {} to {}",
            facility_id, start_date, end_date
        );

        let url = format!(
            "{}/facilities/{}/campsites",
            self.ridb_base_url, facility_id
        );

        let mut params = vec![("limit", "1000".to_string()), ("offset", "0".to_string())];

        if let Some(ref api_key) = self.api_key {
            params.push(("apikey", api_key.clone()));
        }

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| ScanError::ApiError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            match status.as_u16() {
                429 => return Err(ScanError::RateLimited),
                401 | 403 => return Err(ScanError::AuthenticationFailed),
                404 => return Err(ScanError::NotFound),
                _ => return Err(ScanError::ApiError(format!("HTTP {}", status))),
            }
        }

        let rec_response: RecGovAvailabilityResponse = response
            .json()
            .await
            .map_err(|e| ScanError::ApiError(format!("Failed to parse response: {}", e)))?;

        let rec_data_len = rec_response.rec_data.len();

        // Convert to our internal format
        let mut available_sites = Vec::new();

        for campsite in rec_response.rec_data {
            if let Some(ref availabilities) = campsite.availabilities {
                let sites_for_campsite =
                    self.parse_availability_data(&campsite, availabilities, start_date, end_date);
                available_sites.extend(sites_for_campsite);
            }
        }

        Ok(CampgroundAvailability {
            campground_id: facility_id.to_string(),
            available_sites,
            total_sites: rec_data_len,
            checked_at: Utc::now(),
        })
    }

    /// Search for facilities by name or location
    pub async fn search_facilities(
        &self,
        query: &str,
        state: Option<&str>,
        activity: Option<&str>,
    ) -> Result<Vec<RecGovFacility>, ScanError> {
        debug!("Searching facilities with query: {}", query);

        let url = format!("{}/facilities", self.ridb_base_url);

        let mut params = vec![
            ("limit", "50".to_string()),
            ("offset", "0".to_string()),
            ("query", query.to_string()),
        ];

        if let Some(state_code) = state {
            params.push(("state", state_code.to_string()));
        }

        if let Some(activity_id) = activity {
            params.push(("activity", activity_id.to_string()));
        }

        if let Some(ref api_key) = self.api_key {
            params.push(("apikey", api_key.clone()));
        }

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| ScanError::ApiError(format!("Facility search failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            match status.as_u16() {
                429 => return Err(ScanError::RateLimited),
                401 | 403 => return Err(ScanError::AuthenticationFailed),
                404 => return Err(ScanError::NotFound),
                _ => return Err(ScanError::ApiError(format!("HTTP {}", status))),
            }
        }

        let facility_response: RecGovFacilityResponse = response.json().await.map_err(|e| {
            ScanError::ApiError(format!("Failed to parse facility response: {}", e))
        })?;

        Ok(facility_response.rec_data)
    }

    /// Get detailed information about a specific facility
    pub async fn get_facility_details(
        &self,
        facility_id: &str,
    ) -> Result<RecGovFacility, ScanError> {
        debug!("Getting facility details for {}", facility_id);

        let url = format!("{}/facilities/{}", self.ridb_base_url, facility_id);

        let mut params = Vec::new();
        if let Some(ref api_key) = self.api_key {
            params.push(("apikey", api_key.clone()));
        }

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| ScanError::ApiError(format!("Facility details request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            match status.as_u16() {
                429 => return Err(ScanError::RateLimited),
                401 | 403 => return Err(ScanError::AuthenticationFailed),
                404 => return Err(ScanError::NotFound),
                _ => return Err(ScanError::ApiError(format!("HTTP {}", status))),
            }
        }

        let facility: RecGovFacility = response
            .json()
            .await
            .map_err(|e| ScanError::ApiError(format!("Failed to parse facility details: {}", e)))?;

        Ok(facility)
    }

    /// Parse availability data from the API response
    fn parse_availability_data(
        &self,
        campsite: &RecGovCampsite,
        availabilities: &HashMap<String, String>,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Vec<SiteAvailability> {
        let mut sites = Vec::new();

        for (date_str, status) in availabilities {
            // Parse the date string (format: "2024-01-15T00:00:00Z")
            let date = match NaiveDate::parse_from_str(&date_str[..10], "%Y-%m-%d") {
                Ok(date) => date,
                Err(_) => {
                    warn!("Failed to parse date: {}", date_str);
                    continue;
                }
            };

            // Only include dates in our requested range
            if date < start_date || date > end_date {
                continue;
            }

            // Determine availability based on status
            let (available, price) = self.parse_availability_status(status);

            sites.push(SiteAvailability {
                site_id: campsite.campsite_id.clone(),
                site_name: campsite
                    .campsite_name
                    .clone()
                    .unwrap_or_else(|| campsite.campsite_id.clone()),
                available,
                date,
                price,
            });
        }

        sites
    }

    /// Parse availability status from recreation.gov internal API format
    fn parse_availability_status(&self, status: &str) -> (bool, Option<f64>) {
        match status {
            "Available" => (true, None),
            "Reserved" => (false, None),
            "Not Available" => (false, None),
            "Not Reservable" => (false, None),
            "Walk-up" => (false, None),
            // Legacy RIDB format support
            "A" => (true, None),  // Available
            "R" => (false, None), // Reserved
            "X" => (false, None), // Not available
            "W" => (false, None), // Walk-up only
            "N" => (false, None), // Not reservable
            s if s.starts_with("$") => {
                // Price string, means available
                let price = s[1..].parse::<f64>().ok();
                (true, price)
            }
            _ => {
                debug!("Unknown availability status: {}", status);
                (false, None)
            }
        }
    }

    /// Get internal campground availability for a date range using Recreation.gov's internal API
    pub async fn get_internal_campground_availability(
        &self,
        facility_id: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<CampgroundAvailability, ScanError> {
        debug!(
            "Fetching internal availability for facility {} from {} to {}",
            facility_id, start_date, end_date
        );

        // Use the Recreation.gov internal API endpoint
        let url = format!(
            "{}/camps/availability/campground/{}/month",
            self.internal_base_url, facility_id
        );

        // Use the first day of the month from start_date, in the exact format the Python script uses
        let month_start = NaiveDate::from_ymd_opt(start_date.year(), start_date.month(), 1)
            .ok_or_else(|| ScanError::DataFormat("Invalid date".to_string()))?;

        let start_date_param = format!("{}T00:00:00.000Z", month_start.format("%Y-%m-%d"));

        let params = vec![("start_date", start_date_param.clone())];

        debug!("Making request to: {}?start_date={}", url, start_date_param);

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| ScanError::ApiError(format!("HTTP request failed: {}", e)))?;

        debug!("API response status: {}", response.status());

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read response body".to_string());
            warn!("API request failed with status {}: {}", status, body);

            match status.as_u16() {
                429 => return Err(ScanError::RateLimited),
                401 | 403 => return Err(ScanError::AuthenticationFailed),
                404 => return Err(ScanError::NotFound),
                _ => return Err(ScanError::ApiError(format!("HTTP {} - {}", status, body))),
            }
        }

        let rec_response: RecGovInternalAvailabilityResponse = response
            .json()
            .await
            .map_err(|e| ScanError::ApiError(format!("Failed to parse response: {}", e)))?;

        let campsites_count = rec_response.campsites.len();
        let mut available_sites = Vec::new();

        // Convert to our internal format
        for (campsite_id, data) in rec_response.campsites {
            let campsite = RecGovCampsite {
                campsite_id: campsite_id.clone(),
                campsite_name: None,
                campsite_type: data.campsite_type,
                type_of_use: None,
                campsite_loop: data.campsite_loop,
                accessible: None,
                latitude: None,
                longitude: None,
                availabilities: Some(data.availabilities),
            };

            if let Some(ref availabilities) = campsite.availabilities {
                let sites_for_campsite =
                    self.parse_availability_data(&campsite, availabilities, start_date, end_date);
                available_sites.extend(sites_for_campsite);
            }
        }

        Ok(CampgroundAvailability {
            campground_id: facility_id.to_string(),
            available_sites,
            total_sites: campsites_count,
            checked_at: Utc::now(),
        })
    }

    /// Get internal campground availability for a specific date using Recreation.gov's internal API
    pub async fn get_internal_campground_availability_by_date(
        &self,
        facility_id: &str,
        date: NaiveDate,
    ) -> Result<CampgroundAvailability, ScanError> {
        debug!(
            "Fetching internal availability for facility {} on {}",
            facility_id, date
        );

        // Use the Recreation.gov internal API endpoint
        let url = format!(
            "{}/camps/availability/campground/{}/month",
            self.internal_base_url, facility_id
        );

        let params = vec![(
            "start_date",
            format!("{}.000Z", date.format("%Y-%m-%dT00:00:00")),
        )];

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| ScanError::ApiError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            match status.as_u16() {
                429 => return Err(ScanError::RateLimited),
                401 | 403 => return Err(ScanError::AuthenticationFailed),
                404 => return Err(ScanError::NotFound),
                _ => return Err(ScanError::ApiError(format!("HTTP {}", status))),
            }
        }

        let rec_response: RecGovInternalAvailabilityResponse = response
            .json()
            .await
            .map_err(|e| ScanError::ApiError(format!("Failed to parse response: {}", e)))?;

        let campsites_count = rec_response.campsites.len();
        let mut available_sites = Vec::new();

        // Convert to our internal format
        for (campsite_id, data) in rec_response.campsites {
            let campsite = RecGovCampsite {
                campsite_id: campsite_id.clone(),
                campsite_name: None,
                campsite_type: data.campsite_type,
                type_of_use: None,
                campsite_loop: data.campsite_loop,
                accessible: None,
                latitude: None,
                longitude: None,
                availabilities: Some(data.availabilities),
            };

            if let Some(ref availabilities) = campsite.availabilities {
                let sites_for_campsite =
                    self.parse_availability_data(&campsite, availabilities, date, date);
                available_sites.extend(sites_for_campsite);
            }
        }

        Ok(CampgroundAvailability {
            campground_id: facility_id.to_string(),
            available_sites,
            total_sites: campsites_count,
            checked_at: Utc::now(),
        })
    }
}

/// Response structure from recreation.gov internal availability API
#[derive(Debug, Deserialize)]
pub struct RecGovInternalAvailabilityResponse {
    pub campsites: HashMap<String, CampsiteAvailabilityData>,
}

/// Campsite availability data from internal API
#[derive(Debug, Deserialize)]
pub struct CampsiteAvailabilityData {
    pub availabilities: HashMap<String, String>,
    #[serde(rename = "campsite_id")]
    pub campsite_id: Option<String>,
    #[serde(rename = "campsite_type")]
    pub campsite_type: Option<String>,
    #[serde(rename = "loop")]
    pub campsite_loop: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_availability_status() {
        let client = RecGovClient::new(None).unwrap();

        // Internal API format
        assert_eq!(client.parse_availability_status("Available"), (true, None));
        assert_eq!(client.parse_availability_status("Reserved"), (false, None));
        assert_eq!(
            client.parse_availability_status("Not Available"),
            (false, None)
        );
        assert_eq!(
            client.parse_availability_status("Not Reservable"),
            (false, None)
        );
        assert_eq!(client.parse_availability_status("Walk-up"), (false, None));

        // Legacy RIDB format
        assert_eq!(client.parse_availability_status("A"), (true, None));
        assert_eq!(client.parse_availability_status("R"), (false, None));
        assert_eq!(client.parse_availability_status("X"), (false, None));
        assert_eq!(
            client.parse_availability_status("$25.00"),
            (true, Some(25.0))
        );
        assert_eq!(
            client.parse_availability_status("$50.50"),
            (true, Some(50.5))
        );
        assert_eq!(client.parse_availability_status("unknown"), (false, None));
    }
}
