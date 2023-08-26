/// Logs a message at the error level.
///
/// # Examples
///
/// ```edition2018
/// use log::error;
///
/// # fn main() {
/// let (err_info, port) = ("No connection", 22);
///
/// error!("Error: {} on port {}", err_info, port);
/// error!(target: "app_events", "App Error: {}, Port: {}", err_info, 22);
/// # }
/// ```
#[macro_export]
macro_rules! error {
    // error!(target: "my_target", key1 = 42, key2 = true; "a {} event", "log")
    // error!(target: "my_target", "a {} event", "log")
    (target: $target:expr, $($arg:tt)+) => (log::log!(target: $target, log::Level::Error, $($arg)+));

    // error!("a {} event", "log")
    ($($arg:tt)+) => (log::log!(target: "Rust", log::Level::Error, $($arg)+))
}

/// Logs a message at the warn level.
///
/// # Examples
///
/// ```edition2018
/// use log::warn;
///
/// # fn main() {
/// let warn_description = "Invalid Input";
///
/// warn!("Warning! {}!", warn_description);
/// warn!(target: "input_events", "App received warning: {}", warn_description);
/// # }
/// ```
#[macro_export]
macro_rules! warn {
    // warn!(target: "my_target", key1 = 42, key2 = true; "a {} event", "log")
    // warn!(target: "my_target", "a {} event", "log")
    (target: $target:expr, $($arg:tt)+) => (log::log!(target: $target, log::Level::Warn, $($arg)+));

    // warn!("a {} event", "log")
    ($($arg:tt)+) => (log::log!(target: "Rust", log::Level::Warn, $($arg)+))
}

/// Logs a message at the info level.
///
/// # Examples
///
/// ```edition2018
/// use log::info;
///
/// # fn main() {
/// # struct Connection { port: u32, speed: f32 }
/// let conn_info = Connection { port: 40, speed: 3.20 };
///
/// info!("Connected to port {} at {} Mb/s", conn_info.port, conn_info.speed);
/// info!(target: "connection_events", "Successful connection, port: {}, speed: {}",
///       conn_info.port, conn_info.speed);
/// # }
/// ```
#[macro_export]
macro_rules! info {
    // info!(target: "my_target", key1 = 42, key2 = true; "a {} event", "log")
    // info!(target: "my_target", "a {} event", "log")
    (target: $target:expr, $($arg:tt)+) => (log::log!(target: $target, log::Level::Info, $($arg)+));

    // info!("a {} event", "log")
    ($($arg:tt)+) => (log::log!(target: "Rust", log::Level::Info, $($arg)+))
}

/// Logs a message at the debug level.
///
/// # Examples
///
/// ```edition2018
/// use log::debug;
///
/// # fn main() {
/// # struct Position { x: f32, y: f32 }
/// let pos = Position { x: 3.234, y: -1.223 };
///
/// debug!("New position: x: {}, y: {}", pos.x, pos.y);
/// debug!(target: "app_events", "New position: x: {}, y: {}", pos.x, pos.y);
/// # }
/// ```
#[macro_export]
macro_rules! debug {
    // debug!(target: "my_target", key1 = 42, key2 = true; "a {} event", "log")
    // debug!(target: "my_target", "a {} event", "log")
    (target: $target:expr, $($arg:tt)+) => (log::log!(target: $target, log::Level::Debug, $($arg)+));

    // debug!("a {} event", "log")
    ($($arg:tt)+) => (log::log!(target: "Rust", log::Level::Debug, $($arg)+))
}

/// Logs a message at the trace level.
///
/// # Examples
///
/// ```edition2018
/// use log::trace;
///
/// # fn main() {
/// # struct Position { x: f32, y: f32 }
/// let pos = Position { x: 3.234, y: -1.223 };
///
/// trace!("Position is: x: {}, y: {}", pos.x, pos.y);
/// trace!(target: "app_events", "x is {} and y is {}",
///        if pos.x >= 0.0 { "positive" } else { "negative" },
///        if pos.y >= 0.0 { "positive" } else { "negative" });
/// # }
/// ```
#[macro_export]
macro_rules! trace {
    // trace!(target: "my_target", key1 = 42, key2 = true; "a {} event", "log")
    // trace!(target: "my_target", "a {} event", "log")
    (target: $target:expr, $($arg:tt)+) => (log::log!(target: $target, log::Level::Trace, $($arg)+));

    // trace!("a {} event", "log")
    ($($arg:tt)+) => (log::log!(target: "Rust", log::Level::Trace, $($arg)+))
}

#[tauri::command]
pub fn js_error(message: String) {
    error!(target: "JavaScript", "{}", message);
}

#[tauri::command]
pub fn js_warn(message: String) {
    warn!(target: "JavaScript", "{}", message);
}

#[tauri::command]
pub fn js_info(message: String) {
    info!(target: "JavaScript", "{}", message);
}

#[tauri::command]
pub fn js_debug(message: String) {
    debug!(target: "JavaScript", "{}", message);
}

#[tauri::command]
pub fn js_trace(message: String) {
    trace!(target: "JavaScript", "{}", message);
}

pub fn to_algebraic(location: (i16, i16)) -> String {
    let mut file = 'a';
    let mut rank = '1';
    match location.0 {
        0 => file = 'a',
        1 => file = 'b',
        2 => file = 'c',
        3 => file = 'd',
        4 => file = 'e',
        5 => file = 'f',
        6 => file = 'g',
        7 => file = 'h',
        _ => {}
    }
    match location.1 {
        0 => rank = '1',
        1 => rank = '2',
        2 => rank = '3',
        3 => rank = '4',
        4 => rank = '5',
        5 => rank = '6',
        6 => rank = '7',
        7 => rank = '8',
        _ => {}
    }
    format!("{}{}", file, rank)
}

pub fn from_algebraic(location: String) -> (i16, i16) {
    let mut chars = location.chars();
    let file = chars.next().unwrap();
    let rank = chars.next().unwrap();
    let mut file_num = 0;
    let mut rank_num = 0;
    match file {
        'a' => file_num = 0,
        'b' => file_num = 1,
        'c' => file_num = 2,
        'd' => file_num = 3,
        'e' => file_num = 4,
        'f' => file_num = 5,
        'g' => file_num = 6,
        'h' => file_num = 7,
        _ => {}
    }
    match rank {
        '1' => rank_num = 0,
        '2' => rank_num = 1,
        '3' => rank_num = 2,
        '4' => rank_num = 3,
        '5' => rank_num = 4,
        '6' => rank_num = 5,
        '7' => rank_num = 6,
        '8' => rank_num = 7,
        _ => {}
    }
    (file_num, rank_num)
}

pub fn from_algebraic_str(location: &str) -> (i16, i16) {
    let mut chars = location.chars();
    let file = chars.next().unwrap();
    let rank = chars.next().unwrap();
    let mut file_num = 0;
    let mut rank_num = 0;
    match file {
        'a' => file_num = 0,
        'b' => file_num = 1,
        'c' => file_num = 2,
        'd' => file_num = 3,
        'e' => file_num = 4,
        'f' => file_num = 5,
        'g' => file_num = 6,
        'h' => file_num = 7,
        _ => {}
    }
    match rank {
        '1' => rank_num = 0,
        '2' => rank_num = 1,
        '3' => rank_num = 2,
        '4' => rank_num = 3,
        '5' => rank_num = 4,
        '6' => rank_num = 5,
        '7' => rank_num = 6,
        '8' => rank_num = 7,
        _ => {}
    }
    (file_num, rank_num)
}
