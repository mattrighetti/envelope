#[macro_export]
macro_rules! other_err {
    ($msg:expr, $err:expr) => {
        std::io::Error::new(std::io::ErrorKind::Other, format!("{}: {}", $msg, $err))
    };
}

#[macro_export]
macro_rules! other_str_err {
    ($msg:expr) => {
        std::io::Error::new(std::io::ErrorKind::Other, format!($msg))
    };
}
