use actix_web::{web, HttpResponse, Result};
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;

async fn search_recreation_areas(query: String) -> Result<Value, Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = format!(
        "https://ridb.recreation.gov/api/v1/recareas?query={}&activity=CAMPING&limit=50",
        urlencoding::encode(&query)
    );

    let response = client
        .get(&url)
        .header("apikey", "da7bd758-b219-4a80-b885-556101d03afb")
        .send()
        .await?;

    let data: Value = response.json().await?;
    Ok(data)
}

async fn get_facilities_for_recarea(recarea_id: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = format!(
        "https://ridb.recreation.gov/api/v1/recareas/{}/facilities?activity=CAMPING&limit=50",
        recarea_id
    );

    let response = client
        .get(&url)
        .header("apikey", "da7bd758-b219-4a80-b885-556101d03afb")
        .send()
        .await?;

    let data: Value = response.json().await?;
    Ok(data)
}

/// Handler for searching facilities based on a query parameter
pub async fn facilities_search(query: web::Query<HashMap<String, String>>) -> Result<HttpResponse> {
    println!("🔍 Facilities search called with query: {:?}", query);

    if let Some(q) = query.get("q") {
        println!("📝 Searching for recreation areas: {}", q);

        match search_recreation_areas(q.to_string()).await {
            Ok(recarea_data) => {
                let mut all_facilities = Vec::new();

                // For each recreation area found, get its facilities
                if let Some(recareas) = recarea_data.get("RECDATA").and_then(|v| v.as_array()) {
                    println!("🏞️ Found {} recreation areas", recareas.len());

                    for recarea in recareas {
                        if let Some(recarea_id) = recarea.get("RecAreaID").and_then(|v| v.as_str())
                        {
                            if let Some(recarea_name) =
                                recarea.get("RecAreaName").and_then(|v| v.as_str())
                            {
                                // FILTER: Only include recreation areas that actually match the search query
                                let query_lower = q.to_lowercase();
                                let name_lower = recarea_name.to_lowercase();

                                // Check if the recreation area name contains the search term
                                if name_lower.contains(&query_lower) {
                                    println!(
                                        "✅ Getting facilities for: {} (ID: {}) - MATCHES query",
                                        recarea_name, recarea_id
                                    );

                                    match get_facilities_for_recarea(recarea_id).await {
                                        Ok(facilities_data) => {
                                            if let Some(facilities) = facilities_data
                                                .get("RECDATA")
                                                .and_then(|v| v.as_array())
                                            {
                                                println!(
                                                    "  📍 Found {} facilities",
                                                    facilities.len()
                                                );
                                                all_facilities.extend(facilities.iter().cloned());
                                            }
                                        }
                                        Err(e) => {
                                            println!(
                                                "  ❌ Error getting facilities for {}: {}",
                                                recarea_name, e
                                            );
                                        }
                                    }
                                } else {
                                    println!(
                                        "⏭️ Skipping: {} - doesn't match query '{}'",
                                        recarea_name, q
                                    );
                                }
                            }
                        }
                    }
                }

                // Filter to show only National Park Service campgrounds (ParentOrgID = "128")
                // let nps_facilities: Vec<Value> = all_facilities
                //     .into_iter()
                //     .filter(|facility| {
                //         facility
                //             .get("ParentOrgID")
                //             .and_then(|id| id.as_str())
                //             .map(|id| id == "128")
                //             .unwrap_or(false)
                //     })
                //     .collect();

                println!("🎯 Returning {} NPS facilities", all_facilities.len());

                // Create response in the same format as the original API
                let response = serde_json::json!({
                    "RECDATA": all_facilities,
                    "METADATA": {
                        "RESULTS": {
                            "CURRENT_COUNT": all_facilities.len(),
                            "TOTAL_COUNT": all_facilities.len()
                        }
                    }
                });

                Ok(HttpResponse::Ok().json(response))
            }
            Err(e) => {
                eprintln!("❌ Error searching recreation areas: {}", e);
                Ok(HttpResponse::InternalServerError().json("Search failed"))
            }
        }
    } else {
        println!("❌ Missing query parameter");
        Ok(HttpResponse::BadRequest().json("Missing query parameter"))
    }
}
