/// Environment variable for the AES-256 encryption key (base64-encoded 32 bytes).
pub const ENV_AUTH_KEY: &str = "NOOK_AUTH_KEY";

/// Environment variable to override the profiles file path.
pub const ENV_PROFILES_FILE: &str = "NOOK_PROFILES_FILE";

/// Environment variable to override the log file path.
pub const ENV_LOG_FILE: &str = "NOOK_LOG_FILE";

/// Environment variable to control stderr log level (standard tracing/log convention).
pub const ENV_RUST_LOG: &str = "RUST_LOG";

/// Environment variable to control the log file level (default: debug).
/// Set to `trace` to include raw API response bodies in the log file.
/// Accepts standard tracing level strings: error, warn, info, debug, trace.
pub const ENV_LOG_LEVEL: &str = "NOOK_LOG_LEVEL";

/// Environment variable to override the XDG cache directory.
pub const ENV_XDG_CACHE_HOME: &str = "XDG_CACHE_HOME";
