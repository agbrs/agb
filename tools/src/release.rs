pub fn command() -> clap::Command {
    clap::Command::new("release")
        .about("Prepares and commits the changes required to release agb")
        .arg(
            clap::Arg::new("version")
                .required(true)
                .help("New version to release")
                .value_parser(version_parser),
        )
        .arg(
            clap::Arg::new("Dry run")
                .long("dry-run")
                .help("Don't do anything with git (but does everything else)")
                .action(clap::ArgAction::SetTrue),
        )
}

pub fn release(matches: &clap::ArgMatches) -> Result<(), Error> {
    let dry_run = matches.get_one::<bool>("Dry run").expect("defined by clap");
    let version = matches
        .get_one::<Version>("version")
        .expect("defined by clap");

    println!("dry run: {}, version: {:?}", dry_run, version);
    todo!()
}

#[derive(Debug)]
pub enum Error {}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

impl Version {
    #[cfg(test)]
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ParseVersionError;

impl std::str::FromStr for Version {
    type Err = ParseVersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let version_array: Vec<_> = s
            .split('.')
            .map(|v| v.parse())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| ParseVersionError)?;

        if version_array.len() > 3 || version_array.is_empty() {
            return Err(ParseVersionError);
        }

        Ok(Version {
            major: version_array[0],
            minor: *version_array.get(1).unwrap_or(&0),
            patch: *version_array.get(2).unwrap_or(&0),
        })
    }
}

fn version_parser(maybe_version: &str) -> Result<Version, &'static str> {
    maybe_version
        .parse()
        .map_err(|_| "Failed to parse version, must be of the format x.y.z")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn verify_cli() {
        command().debug_assert();
    }

    #[test]
    fn can_parse_versions() {
        assert_eq!(Version::from_str("0.1.2").unwrap(), Version::new(0, 1, 2));
        assert_eq!(Version::from_str("0.1").unwrap(), Version::new(0, 1, 0));
        assert_eq!(
            Version::from_str("33.23.4000").unwrap(),
            Version::new(33, 23, 4000)
        );

        assert_eq!(Version::from_str("abc").unwrap_err(), ParseVersionError);
        assert_eq!(Version::from_str("").unwrap_err(), ParseVersionError);
        assert_eq!(Version::from_str("0.2.4.5").unwrap_err(), ParseVersionError);
        assert_eq!(Version::from_str("0.2.4a").unwrap_err(), ParseVersionError);
    }
}
