#[macro_export]
macro_rules! debug {
    (target: $target:expr, $($arg:tt)*) => { };
    ($($arg:tt)*) => { };
}

#[macro_export]
macro_rules! error {
    (target: $target:expr, $($arg:tt)*) => { };
    ($($arg:tt)*) => { };
}

#[macro_export]
macro_rules! info {
    (target: $target:expr, $($arg:tt)*) => { };
    ($($arg:tt)*) => { };
}

#[macro_export]
macro_rules! trace {
    (target: $target:expr, $($arg:tt)*) => { };
    ($($arg:tt)*) => { };
}

#[macro_export]
macro_rules! warn {
    (target: $target:expr, $($arg:tt)*) => { };
    ($($arg:tt)*) => { };
}