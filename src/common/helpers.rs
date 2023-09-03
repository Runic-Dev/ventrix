use std::error::Error;

pub fn err_to_boxed<T>(err: T) -> Box<dyn Error> where T: Into<Box<dyn Error>> {
    err.into()
}

pub fn err_to_boxed_send_sync<T>(err: T) -> Box<dyn Error + Send + Sync> where T: Into<Box<dyn Error + Send + Sync>> {
    err.into()
}
