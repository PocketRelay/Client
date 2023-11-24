/// Constant storing the application version
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Title used for created windows
pub const WINDOW_TITLE: &str = concat!("Pocket Relay Client v", env!("CARGO_PKG_VERSION"));

/// The host address to redirect in the hosts file
pub const HOST_KEY: &str = "gosredirector.ea.com";
/// Host address target (Localhost)
pub const HOST_VALUE: &str = "127.0.0.1";
/// The path to the system hosts file
#[cfg(target_family = "windows")]
pub const HOSTS_PATH: &str = "C:/Windows/System32/drivers/etc/hosts";
#[cfg(target_family = "unix")]
pub const HOSTS_PATH: &str = "/etc/hosts";

/// Window icon bytes
pub const ICON_BYTES: &[u8] = include_bytes!("resources/assets/icon.ico");

/// Name of the file that stores saved pocket relay configuration info
pub const CONFIG_FILE_NAME: &str = "pocket-relay-client.json";

/// The GitHub repository to use for releases
pub const GITHUB_REPOSITORY: &str = "PocketRelay/Client";
