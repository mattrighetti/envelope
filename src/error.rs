#[macro_export]
macro_rules! err {
    ($($tt:tt)*) => { std::io::Error::new(std::io::ErrorKind::Other, format!($($tt)*)) }
}
