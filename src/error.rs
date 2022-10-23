
use std::string::ToString;

// Results and error Handling
pub type Error = String;

// error constructor, works with all types that implement Display
pub fn error(err: impl ToString) -> Error {
    err.to_string()
}

pub type Res<T> = Result<T, Error>;

pub trait ConvertResult<T, E> {
    fn convert(self) -> Res<T>;
}

impl<T, E: ToString> ConvertResult<T, E> for Result<T, E> {
    fn convert(self) -> Res<T> { self.map_err(error) }
}