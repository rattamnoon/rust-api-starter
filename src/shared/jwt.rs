use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{errors::app_error::AppError, shared::types::UserRole};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: Uuid,
    pub email: String,
    pub role: UserRole,
    pub exp: usize,
    pub token_type: TokenType,
}

#[derive(Debug, Clone)]
pub struct TokenBundle {
    pub access_token: String,
    pub refresh_token: String,
    pub refresh_expires_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    access_ttl_seconds: i64,
    refresh_ttl_seconds: i64,
}

impl JwtService {
    pub fn new(secret: String, access_ttl_seconds: i64, refresh_ttl_seconds: i64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            access_ttl_seconds,
            refresh_ttl_seconds,
        }
    }

    pub fn generate_token_pair(
        &self,
        user_id: Uuid,
        email: &str,
        role: UserRole,
    ) -> Result<TokenBundle, AppError> {
        let access_expiry = Utc::now() + Duration::seconds(self.access_ttl_seconds);
        let refresh_expiry = Utc::now() + Duration::seconds(self.refresh_ttl_seconds);

        Ok(TokenBundle {
            access_token: self.generate_token(
                user_id,
                email,
                role,
                access_expiry,
                TokenType::Access,
            )?,
            refresh_token: self.generate_token(
                user_id,
                email,
                role,
                refresh_expiry,
                TokenType::Refresh,
            )?,
            refresh_expires_at: refresh_expiry,
        })
    }

    pub fn decode_token(&self, token: &str) -> Result<JwtClaims, AppError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;

        let token_data = decode::<JwtClaims>(token, &self.decoding_key, &validation)?;
        Ok(token_data.claims)
    }

    fn generate_token(
        &self,
        user_id: Uuid,
        email: &str,
        role: UserRole,
        expiry: DateTime<Utc>,
        token_type: TokenType,
    ) -> Result<String, AppError> {
        let claims = JwtClaims {
            sub: user_id,
            email: email.to_owned(),
            role,
            exp: expiry.timestamp() as usize,
            token_type,
        };

        Ok(encode(&Header::default(), &claims, &self.encoding_key)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_and_decodes_access_tokens() {
        let service = JwtService::new("secret".into(), 60, 120);
        let user_id = Uuid::now_v7();

        let bundle = service
            .generate_token_pair(user_id, "user@example.com", UserRole::Admin)
            .expect("token pair should be generated");
        let claims = service
            .decode_token(&bundle.access_token)
            .expect("access token should decode");

        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.role, UserRole::Admin);
        assert_eq!(claims.token_type, TokenType::Access);
    }
}
