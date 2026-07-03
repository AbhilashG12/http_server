use std::fmt;

#[derive(Debug,Clone,Copy,PartialEq)]
pub enum HttpStatus {
    Continue = 100,
    SwitchingProtocols = 101,
    ok = 200,
    Created = 201,
    Accepted = 202,
    NoContent = 204,
    MovedPermanently = 301,
    Found = 302,
    SeeOther = 303,
    NotModified = 304,
    TemporaryRedirect = 307,
    PermanentRedirect = 308,
    BadRequest = 400,
    Unauthorized = 401,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    RequestTimeout = 408,
    Conflict = 409,
    PayloadTooLarge = 413,
    UriTooLong = 414,
    UnsupportedMediaType = 415,
    TooManyRequests = 429,
    InternalServerError = 500,
    NotImplemented = 501,
    BadGateway = 502,
    ServiceUnavailable = 503,
    GatewayTimeout = 504,
}

impl HttpStatus {
    pub fn code(&self) -> u16{
        *self as u16
    }

    pub fn reason_phrase(&self) -> &'static str {
        match self {
            HttpStatus::Continue => "Continue",
            HttpStatus::SwitchingProtocols => "Switching Protocols",
            
            HttpStatus::ok => "OK",
            HttpStatus::Created => "Created",
            HttpStatus::Accepted => "Accepted",
            HttpStatus::NoContent => "No Content",
              
            HttpStatus::MovedPermanently => "Moved Permanently",
            HttpStatus::Found => "Found",
            HttpStatus::SeeOther => "See Other",
            HttpStatus::NotModified => "Not Modified",
            HttpStatus::TemporaryRedirect => "Temporary Redirect",
            HttpStatus::PermanentRedirect => "Permanent Redirect",
            
            HttpStatus::BadRequest => "Bad Request",
            HttpStatus::Unauthorized => "Unauthorized",
            HttpStatus::Forbidden => "Forbidden",
            HttpStatus::NotFound => "Not Found",
            HttpStatus::MethodNotAllowed => "Method Not Allowed",
            HttpStatus::RequestTimeout => "Request Timeout",
            HttpStatus::Conflict => "Conflict",
            HttpStatus::PayloadTooLarge => "Payload Too Large",
            HttpStatus::UriTooLong => "URI Too Long",
            HttpStatus::UnsupportedMediaType => "Unsupported Media Type",
            HttpStatus::TooManyRequests => "Too Many Requests",
            
            HttpStatus::InternalServerError => "Internal Server Error",
            HttpStatus::NotImplemented => "Not Implemented",
            HttpStatus::BadGateway => "Bad Gateway",
            HttpStatus::ServiceUnavailable => "Service Unavailable",
            HttpStatus::GatewayTimeout => "Gateway Timeout",
        }
    }

    pub fn from_code(code: u16) -> Option<Self> {
        match code {
            100 => Some(HttpStatus::Continue),
            101 => Some(HttpStatus::SwitchingProtocols),
            200 => Some(HttpStatus::ok),
            201 => Some(HttpStatus::Created),
            202 => Some(HttpStatus::Accepted),
            204 => Some(HttpStatus::NoContent),
            301 => Some(HttpStatus::MovedPermanently),
            302 => Some(HttpStatus::Found),
            303 => Some(HttpStatus::SeeOther),
            304 => Some(HttpStatus::NotModified),
            307 => Some(HttpStatus::TemporaryRedirect),
            308 => Some(HttpStatus::PermanentRedirect),
            400 => Some(HttpStatus::BadRequest),
            401 => Some(HttpStatus::Unauthorized),
            403 => Some(HttpStatus::Forbidden),
            404 => Some(HttpStatus::NotFound),
            405 => Some(HttpStatus::MethodNotAllowed),
            408 => Some(HttpStatus::RequestTimeout),
            409 => Some(HttpStatus::Conflict),
            413 => Some(HttpStatus::PayloadTooLarge),
            414 => Some(HttpStatus::UriTooLong),
            415 => Some(HttpStatus::UnsupportedMediaType),
            429 => Some(HttpStatus::TooManyRequests),
            500 => Some(HttpStatus::InternalServerError),
            501 => Some(HttpStatus::NotImplemented),
            502 => Some(HttpStatus::BadGateway),
            503 => Some(HttpStatus::ServiceUnavailable),
            504 => Some(HttpStatus::GatewayTimeout),
            _ => None,
        }
    }
    /// Returns true if it is informational
    pub fn is_informational(&self) -> bool {
        self.code() >= 100 && self.code() < 200
    }

    /// Returns true if this is a success status (2xx)
    pub fn is_success(&self) -> bool {
        self.code() >= 200 && self.code() < 300
    }

    /// Returns true if this is a redirection status (3xx)
    pub fn is_redirection(&self) -> bool {
        self.code() >= 300 && self.code() < 400
    }

    /// Returns true if this is a client error (4xx)
    pub fn is_client_error(&self) -> bool {
        self.code() >= 400 && self.code() < 500
    }

    /// Returns true if this is a server error (5xx)
    pub fn is_server_error(&self) -> bool {
        self.code() >= 500 && self.code() < 600
    }
}

impl fmt::Display for HttpStatus {
    fn fmt(&self,f:&mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{} {}",self.code(),self.reason_phrase())
    }
}

// Common shortcuts 

pub mod status {
    use super::HttpStatus;
    
    pub const OK: HttpStatus = HttpStatus::ok;
    pub const CREATED: HttpStatus = HttpStatus::Created;
    pub const BAD_REQUEST: HttpStatus = HttpStatus::BadRequest;
    pub const NOT_FOUND: HttpStatus = HttpStatus::NotFound;
    pub const INTERNAL_SERVER_ERROR: HttpStatus = HttpStatus::InternalServerError;
}
