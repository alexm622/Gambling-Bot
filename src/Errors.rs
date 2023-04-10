//errors
use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct GenericError {
    msg: String,
}

impl GenericError {
    pub fn new(msg: &impl ToString) -> Self {
        GenericError {
            msg: msg.to_string().to_owned(),
        }
    }
}

impl Error for GenericError {}

impl Display for GenericError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}
