use std::error::Error;
use std::fmt;

/// Error enum for the Epic API
#[derive(Debug)]
pub enum EpicAPIError {
    /// Wrong credentials
    InvalidCredentials,
    /// API error - see the contents
    APIError(String),
    /// Unknown error
    Unknown,
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
            EpicAPIError::Unknown => {
                write!(f, "Unknown Error")
            }
            EpicAPIError::Server => {
                write!(f, "Server Error")
            }
            EpicAPIError::APIError(e) => {
                write!(f, "API Error: {}", e)
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
            EpicAPIError::Unknown => "Unknown Error",
            EpicAPIError::Server => "Server Error",
            EpicAPIError::APIError(_) => "API Error",
            EpicAPIError::InvalidParams => "Invalid Input Parameters",
            EpicAPIError::FabTimeout => "Fab Timeout Error",
        }
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
    fn display_unknown() {
        assert_eq!(format!("{}", EpicAPIError::Unknown), "Unknown Error");
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
        assert_eq!(EpicAPIError::Unknown.description(), "Unknown Error");
        assert_eq!(EpicAPIError::Server.description(), "Server Error");
        assert_eq!(
            EpicAPIError::APIError("x".into()).description(),
            "API Error"
        );
        assert_eq!(
            EpicAPIError::InvalidParams.description(),
            "Invalid Input Parameters"
        );
        assert_eq!(EpicAPIError::FabTimeout.description(), "Fab Timeout Error");
    }

    #[test]
    fn error_is_debug() {
        let _ = format!("{:?}", EpicAPIError::Unknown);
        let _ = format!("{:?}", EpicAPIError::APIError("msg".into()));
    }
}
