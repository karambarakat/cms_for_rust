use std::fmt;

#[derive(Debug, PartialEq)]
pub struct InitializationError(String);

// temperary impl
impl<T: fmt::Display> From<T> for InitializationError {
    fn from(value: T) -> Self {
        InitializationError(format!("{value}"))
    }
}

pub fn verify_initialization() -> Result<(), InitializationError>
{
    std::env::var("JWT_SALT").map_err(|_| "JWT_SALT not set")?;
    Ok(())
}
