pub mod expressions;
pub mod precedence;
pub mod recursive_descent;
pub mod statements;

pub use recursive_descent::Parser;

pub(crate) fn parse_error<T>(msg: impl Into<String>) -> crate::error::Result<T> {
    Err(crate::error::ViperError::driver(msg.into()))
}
