use std::error;
use std::fmt;

#[derive(Debug)]
pub struct ValidationError {
    pub kind: ValidationErrorKind,
}

impl ValidationError {
    pub fn new(kind: ValidationErrorKind) -> ValidationError {
        ValidationError { kind }
    }
}

#[derive(Debug)]
pub enum ValidationErrorKind {
    BothOrgAndUserNotSet,
    BothOrgAndUserSet,
}

impl ValidationErrorKind {
    pub fn as_str(&self) -> &'static str {
        use ValidationErrorKind::*;
        match *self {
            BothOrgAndUserSet => "cannot set both organisation and user flags",
            BothOrgAndUserNotSet => "please specify a user or organisation to clone",
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ValidationError: {}", self.kind.as_str())
    }
}

impl error::Error for ValidationError {}
