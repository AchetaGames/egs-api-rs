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