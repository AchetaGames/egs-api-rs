use std::error::Error;
use std::fmt;

/// Error enum for the Epic API
#[derive(Debug)]
pub enum EpicAPIError {
    /// Wrong credentials
    InvalidCredentials,
    /// API error - see the contents
    APIError(String),
    /// Network/transport error from reqwest
    NetworkError(reqwest::Error),
    /// Failed to deserialize response body
    DeserializationError(String),
    /// HTTP error with non-success status code
    HttpError {
        /// HTTP status code
        status: reqwest::StatusCode,
        /// Response body text
        body: String,
    },
    /// Invalid parameters
    InvalidParams,
    /// Server error
    Server,
    /// FAB Timeout
    FabTimeout,
}

impl fmt::Display for EpicAPIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EpicAPIError::InvalidCredentials => {
                write!(f, "Invalid Credentials")
            }
            EpicAPIError::Server => {
                write!(f, "Server Error")
            }
            EpicAPIError::APIError(e) => {
                write!(f, "API Error: {}", e)
            }
            EpicAPIError::NetworkError(e) => {
                write!(f, "Network Error: {}", e)
            }
            EpicAPIError::DeserializationError(e) => {
                write!(f, "Deserialization Error: {}", e)
            }
            EpicAPIError::HttpError { status, body } => {
                write!(f, "HTTP Error {}: {}", status, body)
            }
            EpicAPIError::InvalidParams => {
                write!(f, "Invalid Input Parameters")
            }
            EpicAPIError::FabTimeout => {
                write!(f, "Fab Timeout Error")
            }
        }
    }
}

impl Error for EpicAPIError {
    fn description(&self) -> &str {
        match *self {
            EpicAPIError::InvalidCredentials => "Invalid Credentials",
            EpicAPIError::Server => "Server Error",
            EpicAPIError::APIError(_) => "API Error",
            EpicAPIError::NetworkError(_) => "Network Error",
            EpicAPIError::DeserializationError(_) => "Deserialization Error",
            EpicAPIError::HttpError { .. } => "HTTP Error",
            EpicAPIError::InvalidParams => "Invalid Input Parameters",
            EpicAPIError::FabTimeout => "Fab Timeout Error",
        }
    }
}

impl From<reqwest::Error> for EpicAPIError {
    fn from(e: reqwest::Error) -> Self {
        EpicAPIError::NetworkError(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn display_invalid_credentials() {
        assert_eq!(
            format!("{}", EpicAPIError::InvalidCredentials),
            "Invalid Credentials"
        );
    }

    #[test]
    fn display_api_error() {
        assert_eq!(
            format!("{}", EpicAPIError::APIError("test".into())),
            "API Error: test"
        );
    }

    #[test]
    fn display_network_error() {
        let err = reqwest::blocking::Client::new()
            .get("http://")
            .send()
            .unwrap_err();
        let message = format!("{}", EpicAPIError::NetworkError(err));
        assert!(message.starts_with("Network Error: "));
    }

    #[test]
    fn display_deserialization_error() {
        assert_eq!(
            format!("{}", EpicAPIError::DeserializationError("test".into())),
            "Deserialization Error: test"
        );
    }

    #[test]
    fn display_http_error() {
        assert_eq!(
            format!(
                "{}",
                EpicAPIError::HttpError {
                    status: reqwest::StatusCode::BAD_REQUEST,
                    body: "bad".to_string()
                }
            ),
            "HTTP Error 400 Bad Request: bad"
        );
    }

    #[test]
    fn display_server() {
        assert_eq!(format!("{}", EpicAPIError::Server), "Server Error");
    }

    #[test]
    fn display_invalid_params() {
        assert_eq!(
            format!("{}", EpicAPIError::InvalidParams),
            "Invalid Input Parameters"
        );
    }

    #[test]
    fn display_fab_timeout() {
        assert_eq!(format!("{}", EpicAPIError::FabTimeout), "Fab Timeout Error");
    }

    #[test]
    #[allow(deprecated)]
    fn error_description() {
        assert_eq!(
            EpicAPIError::InvalidCredentials.description(),
            "Invalid Credentials"
        );
        assert_eq!(EpicAPIError::Server.description(), "Server Error");
        assert_eq!(
            EpicAPIError::APIError("x".into()).description(),
            "API Error"
        );
        let err = reqwest::blocking::Client::new()
            .get("http://")
            .send()
            .unwrap_err();
        assert_eq!(
            EpicAPIError::NetworkError(err).description(),
            "Network Error"
        );
        assert_eq!(
            EpicAPIError::DeserializationError("x".into()).description(),
            "Deserialization Error"
        );
        assert_eq!(
            EpicAPIError::HttpError {
                status: reqwest::StatusCode::INTERNAL_SERVER_ERROR,
                body: "fail".into()
            }
            .description(),
            "HTTP Error"
        );
        assert_eq!(
            EpicAPIError::InvalidParams.description(),
            "Invalid Input Parameters"
        );
        assert_eq!(EpicAPIError::FabTimeout.description(), "Fab Timeout Error");
    }

    #[test]
    fn error_is_debug() {
        let err = reqwest::blocking::Client::new()
            .get("http://")
            .send()
            .unwrap_err();
        let _ = format!("{:?}", EpicAPIError::NetworkError(err));
        let _ = format!("{:?}", EpicAPIError::APIError("msg".into()));
    }
}
