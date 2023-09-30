use semver::Version;

/// Constant storing the application version
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// User agent used for requests (PocketRelayClient/v0.2.8)
pub const PR_USER_AGENT: &str = concat!("PocketRelayClient/v", env!("CARGO_PKG_VERSION"));

/// The host address to redirect in the hosts file
pub const HOST_KEY: &str = "gosredirector.ea.com";
/// Host address target (Localhost)
pub const HOST_VALUE: &str = "127.0.0.1";
/// The path to the system hosts file
pub const HOSTS_PATH: &str = "C:/Windows/System32/drivers/etc/hosts";

/// Window icon bytes
pub const ICON_BYTES: &[u8] = include_bytes!("resources/assets/icon.ico");

/// Bytes of the origin binkw32.dll
pub const BINKW23_DLL_BYTES: &[u8] = include_bytes!("resources/binkw23.dll");
/// Bytes of the proxy binkw32.dll
pub const BINKW32_DLL_BYTES: &[u8] = include_bytes!("resources/binkw32.dll");

/// The local redirector server port
pub const REDIRECTOR_PORT: u16 = 42127;
/// The local proxy main server port
pub const MAIN_PORT: u16 = 42128;
/// The local proxy telemetry server port
pub const TELEMETRY_PORT: u16 = 42129;
/// The local quality of service server port
pub const QOS_PORT: u16 = 42130;
/// The local HTTP server port
pub const HTTP_PORT: u16 = 42131;

/// The minimum server version supported by this client
pub const MIN_SERVER_VERSION: Version = Version::new(0, 5, 0);

/// Server identifier
pub const SERVER_IDENT: &str = "POCKET_RELAY_SERVER";

/// Name of the file that stores saved pocket relay configuration info
pub const CONFIG_FILE_NAME: &str = "pocket-relay-client.json";

/// Constant for whether the client is the native or iced UI version
pub const IS_NATIVE_VERSION: bool = cfg!(feature = "native");
