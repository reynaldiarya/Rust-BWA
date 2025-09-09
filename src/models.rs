use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::{Validate, ValidationError};

fn validate_name(name: &str) -> Result<(), ValidationError> {
    if name.trim().is_empty() {
        return Err(ValidationError::new("name_cannot_be_empty"));
    }

    if name.chars().all(|c| c.is_whitespace()) {
        return Err(ValidationError::new("name_cannot_be_only_whitespace"));
    }

    let invalid_chars = ['&', '*', '<', '>', '?', '|', '"', '`', '\'', ';'];
    if name.chars().any(|c| invalid_chars.contains(&c)) {
        return Err(ValidationError::new("name_contains_invalid_characterss"));
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Item {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateItemPayload {
    #[validate(length(
        min = 1,
        max = 255,
        message = "Name must be between 1 and 255 characters"
    ))]
    #[validate(custom = "validate_name")]
    pub name: String,

    #[validate(length(max = 1000, message = "Description must be less than 1000 characters"))]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateItemPayload {
    #[validate(length(
        min = 1,
        max = 255,
        message = "Name must be between 1 and 255 characters"
    ))]
    pub name: Option<String>,

    #[validate(length(max = 1000, message = "Description must be less than 1000 characters"))]
    pub description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_create_item_payload_valid() {
        let payload = CreateItemPayload {
            name: "Product Valid".to_string(),
            description: Some("Description Valid".to_string()),
        };

        let result = payload.validate();
        assert!(result.is_ok(), "payload valid");
    }

    #[test]
    fn validate_create_item_payload_invalid() {
        let payload = CreateItemPayload {
            name: "".to_string(),
            description: Some("Description Valid".to_string()),
        };

        let result = payload.validate();
        assert!(result.is_err(), "payload invalid");
        let errors = result.err().unwrap();
        assert!(errors.field_errors().contains_key("name"));
    }

    #[test]
    fn validate_create_item_payload_dangerous_chars() {
        let payload = CreateItemPayload {
            name: "<script>alert('Hello')</script>".to_string(),
            description: Some("Description Valid".to_string()),
        };

        let result = payload.validate();
        assert!(result.is_err(), "karakter berbahaya");
        let errors = result.err().unwrap();
        assert!(errors.field_errors().contains_key("name"));
    }

    #[test]
    fn validate_update_item_optional_rules() {
        let payload = UpdateItemPayload {
            name: Some("".to_string()),
            description: None,
        };

        let result = payload.validate();
        assert!(result.is_err(), "name kosong pada update item");

        let payload_ok = UpdateItemPayload {
            name: None,
            description: Some("Ok".to_string()),
        };

        assert!(payload_ok.validate().is_ok());
    }
}
