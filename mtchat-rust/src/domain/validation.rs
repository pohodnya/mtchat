//! Input validation constants and helpers

/// Maximum length for message content (HTML)
pub const MAX_MESSAGE_LENGTH: usize = 50_000;

/// Maximum length for dialog title
pub const MAX_TITLE_LENGTH: usize = 500;

/// Maximum length for display name
pub const MAX_DISPLAY_NAME_LENGTH: usize = 200;

/// Maximum length for company name
pub const MAX_COMPANY_LENGTH: usize = 200;

/// Maximum length for email
pub const MAX_EMAIL_LENGTH: usize = 254;

/// Maximum length for phone
pub const MAX_PHONE_LENGTH: usize = 50;

/// Maximum length for external identifiers (user_id, object_id, tenant_uid)
pub const MAX_IDENTIFIER_LENGTH: usize = 255;

/// Validation error with field name and limit
#[derive(Debug)]
pub struct ValidationError {
    pub field: &'static str,
    pub message: String,
}

impl ValidationError {
    pub fn too_long(field: &'static str, max_length: usize) -> Self {
        Self {
            field,
            message: format!(
                "{} exceeds maximum length of {} characters",
                field, max_length
            ),
        }
    }

    pub fn required(field: &'static str) -> Self {
        Self {
            field,
            message: format!("{} is required", field),
        }
    }
}

/// Validate text field length
pub fn validate_length(
    value: &str,
    field: &'static str,
    max_length: usize,
) -> Result<(), ValidationError> {
    if value.len() > max_length {
        return Err(ValidationError::too_long(field, max_length));
    }
    Ok(())
}

/// Validate optional text field length
pub fn validate_optional_length(
    value: &Option<String>,
    field: &'static str,
    max_length: usize,
) -> Result<(), ValidationError> {
    if let Some(v) = value {
        validate_length(v, field, max_length)?;
    }
    Ok(())
}

/// Validate message content
pub fn validate_message_content(content: &str) -> Result<(), ValidationError> {
    if content.trim().is_empty() {
        return Err(ValidationError::required("content"));
    }
    validate_length(content, "content", MAX_MESSAGE_LENGTH)
}

/// Validate dialog title
pub fn validate_title(title: &Option<String>) -> Result<(), ValidationError> {
    validate_optional_length(title, "title", MAX_TITLE_LENGTH)
}

/// Validate display name
pub fn validate_display_name(name: &str) -> Result<(), ValidationError> {
    if name.trim().is_empty() {
        return Err(ValidationError::required("display_name"));
    }
    validate_length(name, "display_name", MAX_DISPLAY_NAME_LENGTH)
}

/// Validate company name
pub fn validate_company(company: &Option<String>) -> Result<(), ValidationError> {
    validate_optional_length(company, "company", MAX_COMPANY_LENGTH)
}

/// Validate email
pub fn validate_email(email: &Option<String>) -> Result<(), ValidationError> {
    validate_optional_length(email, "email", MAX_EMAIL_LENGTH)
}

/// Validate phone
pub fn validate_phone(phone: &Option<String>) -> Result<(), ValidationError> {
    validate_optional_length(phone, "phone", MAX_PHONE_LENGTH)
}

/// Validate external identifier (user_id, object_id, tenant_uid)
pub fn validate_identifier(value: &str, field: &'static str) -> Result<(), ValidationError> {
    if value.is_empty() {
        return Err(ValidationError::required(field));
    }
    if value.len() > MAX_IDENTIFIER_LENGTH {
        return Err(ValidationError::too_long(field, MAX_IDENTIFIER_LENGTH));
    }
    Ok(())
}

/// Validate optional external identifier
pub fn validate_optional_identifier(
    value: &Option<String>,
    field: &'static str,
) -> Result<(), ValidationError> {
    if let Some(v) = value {
        validate_identifier(v, field)?;
    }
    Ok(())
}

/// Validate S3 key for path traversal attacks and dialog ownership
pub fn validate_s3_key(s3_key: &str, dialog_id: uuid::Uuid) -> Result<(), ValidationError> {
    // Check for path traversal sequences
    if s3_key.contains("..") {
        return Err(ValidationError {
            field: "s3_key",
            message: "Invalid S3 key: path traversal not allowed".to_string(),
        });
    }

    // Normalize and check for encoded path traversal (e.g., %2e%2e)
    if s3_key.contains("%2e") || s3_key.contains("%2E") {
        return Err(ValidationError {
            field: "s3_key",
            message: "Invalid S3 key: encoded characters not allowed".to_string(),
        });
    }

    // Check for null bytes (truncation attack)
    if s3_key.contains('\0') {
        return Err(ValidationError {
            field: "s3_key",
            message: "Invalid S3 key: null bytes not allowed".to_string(),
        });
    }

    // Verify S3 key belongs to the correct dialog
    let expected_prefix = format!("dialogs/{}/", dialog_id);
    if !s3_key.starts_with(&expected_prefix) {
        return Err(ValidationError {
            field: "s3_key",
            message: "Invalid S3 key: file does not belong to this dialog".to_string(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_length_ok() {
        assert!(validate_length("hello", "field", 10).is_ok());
    }

    #[test]
    fn test_validate_length_too_long() {
        let result = validate_length("hello world", "field", 5);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.field, "field");
    }

    #[test]
    fn test_validate_message_content_empty() {
        assert!(validate_message_content("").is_err());
        assert!(validate_message_content("   ").is_err());
    }

    #[test]
    fn test_validate_message_content_ok() {
        assert!(validate_message_content("hello").is_ok());
    }

    #[test]
    fn test_validate_s3_key_valid() {
        let dialog_id = uuid::Uuid::parse_str("12345678-1234-1234-1234-123456789abc").unwrap();
        let key = format!("dialogs/{}/pending/file.txt", dialog_id);
        assert!(validate_s3_key(&key, dialog_id).is_ok());
    }

    #[test]
    fn test_validate_s3_key_path_traversal() {
        let dialog_id = uuid::Uuid::parse_str("12345678-1234-1234-1234-123456789abc").unwrap();
        let key = format!("dialogs/{}/../../../etc/passwd", dialog_id);
        assert!(validate_s3_key(&key, dialog_id).is_err());
    }

    #[test]
    fn test_validate_s3_key_encoded_traversal() {
        let dialog_id = uuid::Uuid::parse_str("12345678-1234-1234-1234-123456789abc").unwrap();
        let key = format!("dialogs/{}/%2e%2e/secret", dialog_id);
        assert!(validate_s3_key(&key, dialog_id).is_err());
    }

    #[test]
    fn test_validate_s3_key_wrong_dialog() {
        let dialog_id = uuid::Uuid::parse_str("12345678-1234-1234-1234-123456789abc").unwrap();
        let other_dialog = uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();
        let key = format!("dialogs/{}/pending/file.txt", other_dialog);
        assert!(validate_s3_key(&key, dialog_id).is_err());
    }

    #[test]
    fn test_validate_s3_key_null_byte() {
        let dialog_id = uuid::Uuid::parse_str("12345678-1234-1234-1234-123456789abc").unwrap();
        let key = format!("dialogs/{}/pending/file\0.txt", dialog_id);
        assert!(validate_s3_key(&key, dialog_id).is_err());
    }
}
