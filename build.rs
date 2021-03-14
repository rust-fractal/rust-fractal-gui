use vergen::{Config, Error, vergen};
use vergen::{TimestampKind, TimeZone};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::default();
    // Generate all three date/time instructions
    *config.build_mut().kind_mut() = TimestampKind::All;
    // Change the date/time instructions to show `Local` time
    *config.build_mut().timezone_mut() = TimeZone::Local;
    // Generate the default 'cargo:' instruction output
    vergen(config).map_err(Error::into)
}