use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{error, info};
use uuid::Uuid;

use crate::executor::{CampgroundAvailability, NotificationError, NotificationService};

/// Implementation of notification service that supports email and SMS
pub struct NotificationServiceImpl {
    pool: PgPool,
    email_service: Option<Arc<dyn EmailService>>,
    sms_service: Option<Arc<dyn SmsService>>,
}

/// Trait for email service implementations
#[async_trait::async_trait]
pub trait EmailService: Send + Sync {
    async fn send_email(
        &self,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<String, NotificationError>;
}

/// Trait for SMS service implementations
#[async_trait::async_trait]
pub trait SmsService: Send + Sync {
    async fn send_sms(&self, to: &str, message: &str) -> Result<String, NotificationError>;
}

/// User preferences for notifications
#[derive(Debug, Deserialize)]
pub struct NotificationPreferences {
    pub email: bool,
    pub sms: bool,
}

/// Notification record for database storage
#[derive(Debug, Serialize)]
pub struct NotificationRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_scan_id: Option<Uuid>,
    pub notification_type: String,
    pub recipient: String,
    pub subject: Option<String>,
    pub message: String,
    pub availability_details: serde_json::Value,
    pub status: String,
    pub sent_at: Option<DateTime<Utc>>,
    pub external_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl NotificationServiceImpl {
    pub fn new(
        pool: PgPool,
        email_service: Option<Arc<dyn EmailService>>,
        sms_service: Option<Arc<dyn SmsService>>,
    ) -> Self {
        Self {
            pool,
            email_service,
            sms_service,
        }
    }
}

#[async_trait::async_trait]
impl NotificationService for NotificationServiceImpl {
    async fn send_availability_notification(
        &self,
        user_id: &Uuid,
        scan_id: &Uuid,
        availability: &CampgroundAvailability,
    ) -> Result<(), NotificationError> {
        info!(
            "Sending availability notification for scan {} to user {}",
            scan_id, user_id
        );

        // Get user details and preferences
        let user = self.get_user_details(user_id).await?;

        // Get scan details for context
        let scan = self.get_scan_details(scan_id).await?;

        // Create notification content
        let (subject, message) = self.create_notification_content(&scan, availability);

        // Send email if enabled and service available
        if user.preferences.email && user.email_verified && self.email_service.is_some() {
            if let Some(ref email_service) = self.email_service {
                match email_service
                    .send_email(&user.email, &subject, &message)
                    .await
                {
                    Ok(external_id) => {
                        info!(
                            "Email sent successfully to {} for scan {}",
                            user.email, scan_id
                        );
                        self.record_notification(
                            user_id,
                            Some(*scan_id),
                            "email",
                            &user.email,
                            Some(&subject),
                            &message,
                            availability,
                            "sent",
                            Some(&external_id),
                        )
                        .await?;
                    }
                    Err(e) => {
                        error!("Failed to send email to {}: {}", user.email, e);
                        self.record_notification(
                            user_id,
                            Some(*scan_id),
                            "email",
                            &user.email,
                            Some(&subject),
                            &message,
                            availability,
                            "failed",
                            None,
                        )
                        .await?;
                        return Err(e);
                    }
                }
            }
        }

        // Send SMS if enabled and service available
        if user.preferences.sms
            && user.phone_verified
            && user.phone.is_some()
            && self.sms_service.is_some()
        {
            if let (Some(phone), Some(sms_service)) = (&user.phone, &self.sms_service) {
                // Create shorter message for SMS
                let sms_message = self.create_sms_message(&scan, availability);

                match sms_service.send_sms(phone, &sms_message).await {
                    Ok(external_id) => {
                        info!("SMS sent successfully to {} for scan {}", phone, scan_id);
                        self.record_notification(
                            user_id,
                            Some(*scan_id),
                            "sms",
                            phone,
                            None,
                            &sms_message,
                            availability,
                            "sent",
                            Some(&external_id),
                        )
                        .await?;
                    }
                    Err(e) => {
                        error!("Failed to send SMS to {}: {}", phone, e);
                        self.record_notification(
                            user_id,
                            Some(*scan_id),
                            "sms",
                            phone,
                            None,
                            &sms_message,
                            availability,
                            "failed",
                            None,
                        )
                        .await?;
                        return Err(e);
                    }
                }
            }
        }

        Ok(())
    }
}

impl NotificationServiceImpl {
    /// Get user details including notification preferences
    async fn get_user_details(&self, user_id: &Uuid) -> Result<UserDetails, NotificationError> {
        let row = sqlx::query!(
            r#"
            SELECT email, name, phone, email_verified, phone_verified, notification_preferences
            FROM users
            WHERE id = $1 AND is_active = true
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let preferences: NotificationPreferences =
                    if let Some(prefs) = row.notification_preferences {
                        serde_json::from_value(prefs).unwrap_or(NotificationPreferences {
                            email: true,
                            sms: true,
                        })
                    } else {
                        NotificationPreferences {
                            email: true,
                            sms: true,
                        }
                    };

                Ok(UserDetails {
                    email: row.email,
                    name: row.name,
                    phone: row.phone,
                    email_verified: row.email_verified.unwrap_or(false),
                    phone_verified: row.phone_verified.unwrap_or(false),
                    preferences,
                })
            }
            None => Err(NotificationError::Database(sqlx::Error::RowNotFound)),
        }
    }

    /// Get scan details for notification context
    async fn get_scan_details(&self, scan_id: &Uuid) -> Result<ScanDetails, NotificationError> {
        let row = sqlx::query!(
            r#"
            SELECT 
                us.campground_id, us.check_in_date, us.check_out_date, us.nights,
                COALESCE(c.name, 'Unknown Campground') as "campground_name!: String"
            FROM user_scans us
            LEFT JOIN campgrounds c ON us.campground_id = c.id
            WHERE us.id = $1
            "#,
            scan_id
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(ScanDetails {
                campground_id: row.campground_id,
                campground_name: row.campground_name,
                check_in_date: row.check_in_date,
                check_out_date: row.check_out_date,
                nights: row.nights.unwrap_or(0),
            }),
            None => Err(NotificationError::Database(sqlx::Error::RowNotFound)),
        }
    }

    /// Create email notification content
    fn create_notification_content(
        &self,
        scan: &ScanDetails,
        availability: &CampgroundAvailability,
    ) -> (String, String) {
        let subject = format!(
            "🏕️ Campsite Available: {} ({} - {})",
            scan.campground_name,
            scan.check_in_date.format("%m/%d"),
            scan.check_out_date.format("%m/%d")
        );

        let available_sites = availability
            .available_sites
            .iter()
            .filter(|site| site.available)
            .collect::<Vec<_>>();

        let site_list = if available_sites.len() <= 5 {
            available_sites
                .iter()
                .map(|site| {
                    let price_info = if let Some(price) = site.price {
                        format!(" (${:.2})", price)
                    } else {
                        String::new()
                    };
                    format!(
                        "• {} on {}{}",
                        site.site_name,
                        site.date.format("%m/%d/%Y"),
                        price_info
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            format!(
                "{} sites available (showing first 5):\n{}",
                available_sites.len(),
                available_sites
                    .iter()
                    .take(5)
                    .map(|site| {
                        let price_info = if let Some(price) = site.price {
                            format!(" (${:.2})", price)
                        } else {
                            String::new()
                        };
                        format!(
                            "• {} on {}{}",
                            site.site_name,
                            site.date.format("%m/%d/%Y"),
                            price_info
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        };

        let message = format!(
            r#"Great news! New campsites are available for your search:

🏕️ Campground: {}
📅 Your Dates: {} to {} ({} nights)

Available Sites:
{}

Visit recreation.gov to book your site:
https://www.recreation.gov/camping/campgrounds/{}

This notification was sent because you set up a scan for this campground. You can manage your scans in the Campsite Tracker app.
"#,
            scan.campground_name,
            scan.check_in_date.format("%B %d, %Y"),
            scan.check_out_date.format("%B %d, %Y"),
            scan.nights,
            site_list,
            scan.campground_id
        );

        (subject, message)
    }

    /// Create SMS notification content (shorter version)
    fn create_sms_message(
        &self,
        scan: &ScanDetails,
        availability: &CampgroundAvailability,
    ) -> String {
        let available_count = availability
            .available_sites
            .iter()
            .filter(|site| site.available)
            .count();

        format!(
            "🏕️ {} campsites available at {} for {}-{}! Check recreation.gov to book. -Campsite Tracker",
            available_count,
            scan.campground_name,
            scan.check_in_date.format("%m/%d"),
            scan.check_out_date.format("%m/%d")
        )
    }

    /// Record notification in database
    async fn record_notification(
        &self,
        user_id: &Uuid,
        user_scan_id: Option<Uuid>,
        notification_type: &str,
        recipient: &str,
        subject: Option<&str>,
        message: &str,
        availability: &CampgroundAvailability,
        status: &str,
        external_id: Option<&str>,
    ) -> Result<(), NotificationError> {
        let availability_json = serde_json::to_value(availability)
            .map_err(|e| NotificationError::Database(sqlx::Error::Protocol(e.to_string())))?;

        let sent_at = if status == "sent" {
            Some(Utc::now())
        } else {
            None
        };

        sqlx::query!(
            r#"
            INSERT INTO notifications 
            (user_id, user_scan_id, type, recipient, subject, message, availability_details, status, sent_at, external_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            user_id,
            user_scan_id,
            notification_type,
            recipient,
            subject,
            message,
            availability_json,
            status,
            sent_at,
            external_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

/// User details for notifications
#[derive(Debug)]
struct UserDetails {
    pub email: String,
    pub name: String,
    pub phone: Option<String>,
    pub email_verified: bool,
    pub phone_verified: bool,
    pub preferences: NotificationPreferences,
}

/// Scan details for notifications
#[derive(Debug)]
struct ScanDetails {
    pub campground_id: String,
    pub campground_name: String,
    pub check_in_date: chrono::NaiveDate,
    pub check_out_date: chrono::NaiveDate,
    pub nights: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockEmailService;

    #[async_trait::async_trait]
    impl EmailService for MockEmailService {
        async fn send_email(
            &self,
            _to: &str,
            _subject: &str,
            _body: &str,
        ) -> Result<String, NotificationError> {
            Ok("mock-email-id".to_string())
        }
    }

    struct MockSmsService;

    #[async_trait::async_trait]
    impl SmsService for MockSmsService {
        async fn send_sms(&self, _to: &str, _message: &str) -> Result<String, NotificationError> {
            Ok("mock-sms-id".to_string())
        }
    }

    // Add tests here when we have a test database setup
}
