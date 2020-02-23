
use std::{error::Error, process, fmt};


#[derive(Debug)]
pub struct NoneValueError {}


impl std::error::Error for NoneValueError {
    fn description(&self) -> &str { "NoneValueError" }
    fn cause(&self) -> Option<&dyn Error> { None }
}

impl fmt::Display for NoneValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "NoneValueError") }
}



// trait OptionToResultErr
pub trait OptionToResultErr<T> {
    fn to_res_err(self) -> Result<T, NoneValueError>;
}


impl <T> OptionToResultErr<T> for Option<T> {
    fn to_res_err(self) -> Result<T, NoneValueError> {
        match self {
            Some(value) => Ok(value),
            None => Err(NoneValueError {})
        }
    }
}




// trait UnwrapOrExit
pub trait UnwrapOrExit<T, E: Error> {
    fn unwrap_or_exit(self, message:&'static str, code:i32) -> T;
}



fn exit_err (err: impl Error, message:&'static str, code:i32) -> ! {
    if message.eq("") { eprintln!("{:?}", err) }
    else { eprintln!("{} ({})", message, err.description()) }
    process::exit(code)
}



impl <T> UnwrapOrExit<T, NoneValueError> for Option<T> {
    fn unwrap_or_exit(self, message:&'static str, code:i32) -> T {
        match self {
            Some(value) => value,
            None => exit_err(NoneValueError {}, message, code)
        }
    }
}

impl <T, E:Error> UnwrapOrExit<T, E> for Result<T, E> {
    fn unwrap_or_exit(self, message:&'static str, code:i32) -> T {
        match self {
            Ok(value) => value,
            Err(err) => exit_err(err, message, code)
        }
    }
}