use vergen::{Config, Error, ShaKind, TimestampKind, vergen};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::default();
    // Generate all three date/time instructions
    *config.git_mut().sha_kind_mut() = ShaKind::Short;
    *config.git_mut().commit_timestamp_kind_mut() = TimestampKind::DateAndTime;
    // Generate the default 'cargo:' instruction output
    vergen(config).map_err(Error::into)
}