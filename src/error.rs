#[macro_export]
macro_rules! std_err {
    ($($tt:tt)*) => { std::io::Error::new(std::io::ErrorKind::Other, format!($($tt)*)) }
}

#[macro_export]
macro_rules! err {
    ($($tt:tt)*) => { Err(std::io::Error::new(std::io::ErrorKind::Other, format!($($tt)*))) }
}
