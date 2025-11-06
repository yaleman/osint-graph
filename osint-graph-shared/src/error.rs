use openidconnect::{reqwest, ConfigurationError, DiscoveryError};

#[derive(Debug)]
pub enum OsintError {
    Configuration(String),
    IOError(String),
    NotFound(String),
    DatabaseError(String),
    ValidationError(String),
    Unauthorized(String),
    Other(String),
    OidcDiscovery(String),
    OidcStateParameterExpired,
}

impl From<std::io::Error> for OsintError {
    fn from(err: std::io::Error) -> Self {
        OsintError::IOError(err.to_string())
    }
}

impl From<DiscoveryError<reqwest::Error>> for OsintError {
    fn from(err: DiscoveryError<reqwest::Error>) -> Self {
        OsintError::OidcDiscovery(format!("OIDC discovery error: {:?}", err))
    }
}

impl From<sea_orm::DbErr> for OsintError {
    fn from(err: sea_orm::DbErr) -> Self {
        OsintError::DatabaseError(err.to_string())
    }
}

impl From<ConfigurationError> for OsintError {
    fn from(err: ConfigurationError) -> Self {
        OsintError::Configuration(err.to_string())
    }
}
