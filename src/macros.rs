macro_rules! error {
    ($message:expr) => {
        RedError::generic($message)
    };
    ($message:expr, $($arg:tt)*) => {
        RedError::generic(format!($message, $($arg)+).as_str())
    }
}

macro_rules! log_debug {
    ($logger:expr, $target:expr) => {
        if cfg!(debug_assertions) {
            $logger.log_debug($target)
        }
    };
    ($logger:expr, $target:expr, $($arg:tt)*) => {
        if cfg!(debug_assertions) {
            $logger.log_debug(format!($target, $($arg)+).as_str())
        }
    }
}
