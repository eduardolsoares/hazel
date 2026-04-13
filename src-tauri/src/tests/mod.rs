#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct SaveResponse {
        pub success: bool,
        pub file_path: Option<String>,
        pub error: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct XelatexCheckResult {
        pub available: bool,
        pub version: Option<String>,
    }

    #[test]
    fn test_save_response_success() {
        let response = SaveResponse {
            success: true,
            file_path: Some("/path/to/file.md".to_string()),
            error: None,
        };

        assert!(response.success);
        assert_eq!(response.file_path, Some("/path/to/file.md".to_string()));
        assert!(response.error.is_none());

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":true"));
    }

    #[test]
    fn test_save_response_error() {
        let response = SaveResponse {
            success: false,
            file_path: None,
            error: Some("File not found".to_string()),
        };

        assert!(!response.success);
        assert!(response.file_path.is_none());
        assert_eq!(response.error, Some("File not found".to_string()));

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":false"));
    }

    #[test]
    fn test_save_response_serialize() {
        let response = SaveResponse {
            success: true,
            file_path: Some("test.md".to_string()),
            error: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("test.md"));
    }

    #[test]
    fn test_save_response_deserialize() {
        let json = r#"{"success":true,"file_path":"output.pdf","error":null}"#;
        let response: SaveResponse = serde_json::from_str(json).unwrap();

        assert!(response.success);
        assert_eq!(response.file_path, Some("output.pdf".to_string()));
    }

    #[test]
    fn test_xelatex_check_result_available() {
        let result = XelatexCheckResult {
            available: true,
            version: Some("XeTeX 3.14159265".to_string()),
        };

        assert!(result.available);
        assert!(result.version.is_some());
    }

    #[test]
    fn test_xelatex_check_result_unavailable() {
        let result = XelatexCheckResult {
            available: false,
            version: None,
        };

        assert!(!result.available);
        assert!(result.version.is_none());
    }

    #[test]
    fn test_export_request_serialization() {
        #[derive(Serialize)]
        struct ExportRequest {
            content: String,
            default_name: Option<String>,
        }

        let request = ExportRequest {
            content: "# Title\n\nContent".to_string(),
            default_name: Some("document".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("# Title"));
        assert!(json.contains("document"));
    }

    #[test]
    fn test_markdown_to_pdf_request() {
        #[derive(Serialize)]
        struct RenderRequest {
            content: String,
            profile: String,
        }

        let request = RenderRequest {
            content: "# Hello World\n\nThis is a test.".to_string(),
            profile: "abnt".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("abnt"));
    }
}
