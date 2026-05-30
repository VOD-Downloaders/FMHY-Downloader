// pub const VERSION_TAG_FULL: &str = concat!("v", env!("CARGO_PKG_VERSION")); // ex. "v0.1.0"
pub const VERSION_TAG_MAJOR_MINOR: &str = concat!("v", env!("CARGO_PKG_VERSION_MAJOR"), ".", env!("CARGO_PKG_VERSION_MINOR")); // ex. "v0.1
// pub const VERSION_TAG_MAJOR_MINOR_SUFFIX: &str = concat!("v", env!("CARGO_PKG_VERSION_MAJOR"), ".", env!("CARGO_PKG_VERSION_MINOR"), "-", env!("CARGO_PKG_VERSION_PRE")); // ex. "v0.1-alpha2"
