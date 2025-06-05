use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use uuid::Uuid;

use super::types::{AuthError, Claims, User};

/// A service for handling JWT operations such as generating and verifying tokens.
#[derive(Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtService {
    /// Creates a new instance of `JwtService` with keys derived from the JWT secret.
    pub fn new() -> Self {
        let secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-secret-key-change-this-in-production".to_string());

        Self {
            encoding_key: EncodingKey::from_secret(secret.as_ref()),
            decoding_key: DecodingKey::from_secret(secret.as_ref()),
        }
    }

    /// Generates an access token for a user with a 1-hour expiration.
    pub fn generate_access_token(&self, user: &User) -> Result<String, AuthError> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(1))
            .expect("valid timestamp")
            .timestamp() as usize;

        let claims = Claims {
            sub: user.id.to_string(),
            email: user.email.clone(),
            role: user.role.clone(),
            exp: expiration,
            iat: Utc::now().timestamp() as usize,
        };

        let token = encode(&Header::default(), &claims, &self.encoding_key)?;
        Ok(token)
    }

    /// Generates a refresh token for a user with a 30-day expiration.
    pub fn generate_refresh_token(&self, user_id: &Uuid) -> Result<String, AuthError> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::days(30))
            .expect("valid timestamp")
            .timestamp() as usize;

        let claims = Claims {
            sub: user_id.to_string(),
            email: String::new(), // Empty for refresh tokens
            role: String::new(),  // Empty for refresh tokens
            exp: expiration,
            iat: Utc::now().timestamp() as usize,
        };

        let token = encode(&Header::default(), &claims, &self.encoding_key)?;
        Ok(token)
    }

    /// Verifies a JWT token and returns the claims if valid.
    pub fn verify_token(&self, token: &str) -> Result<Claims, AuthError> {
        let token_data = decode::<Claims>(
            token,
            &self.decoding_key,
            &Validation::new(Algorithm::HS256),
        )?;

        Ok(token_data.claims)
    }

    /// Extracts the user ID from a JWT token.
    pub fn extract_user_id_from_token(&self, token: &str) -> Result<Uuid, AuthError> {
        let claims = self.verify_token(token)?;
        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
            AuthError::Jwt(jsonwebtoken::errors::Error::from(
                jsonwebtoken::errors::ErrorKind::InvalidSubject,
            ))
        })?;

        Ok(user_id)
    }
}
