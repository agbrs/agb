use std::{path::Path, process::Command};

use crate::utils::find_agb_root_directory;

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

    let root_directory = find_agb_root_directory().map_err(|_| Error::FindRootDirectory)?;

    // if not dry run, check that there are no out-standing changes in git
    if !dry_run && !execute_git_command(&root_directory, &["status", "--porcelain"])?.is_empty() {
        println!("Uncommitted changes, please commit first");
        return Ok(());
    }

    // Check that we are in the master branch
    if !dry_run
        && execute_git_command(&root_directory, &["symbolic-ref", "--short", "HEAD"])? != "master"
    {
        println!("You must be on the master branch before releasing");
        return Ok(());
    }

    let project_toml_files = glob_many(&root_directory, &["agb-*/Cargo.toml"])?;

    update_to_version(
        &root_directory,
        &root_directory.join("agb/Cargo.toml"),
        version,
    )?;

    for toml_file in project_toml_files {
        update_to_version(&root_directory, &toml_file, version)?;
    }

    Command::new("just")
        .arg("ci")
        .spawn()
        .map_err(|_| Error::JustCiFailed)?;

    Ok(())
}

fn update_to_version(
    root_directory: &Path,
    toml_file: &Path,
    new_version: &Version,
) -> Result<(), Error> {
    let directory_name = toml_file.parent().unwrap().file_name().unwrap();
    let project_name = directory_name.to_string_lossy().replace('-', "_");

    let toml_file_content = std::fs::read_to_string(toml_file).map_err(|_| Error::ReadTomlFile)?;
    let mut cargo_toml = toml_file_content
        .parse::<toml_edit::Document>()
        .map_err(|_| Error::InvalidToml(toml_file.to_string_lossy().into_owned()))?;

    let new_version = format!("{new_version}");
    cargo_toml["package"]["version"] = toml_edit::value(&new_version);

    std::fs::write(toml_file, cargo_toml.to_string()).map_err(|_| Error::WriteTomlFile)?;

    for cargo_toml_file in glob_many(
        root_directory,
        &[
            "agb-*/Cargo.toml",
            "agb/Cargo.toml",
            "examples/*/Cargo.toml",
            "book/games/*/Cargo.toml",
            "template/Cargo.toml",
        ],
    )? {
        let toml_file_content =
            std::fs::read_to_string(&cargo_toml_file).map_err(|_| Error::ReadTomlFile)?;
        let mut cargo_toml = toml_file_content
            .parse::<toml_edit::Document>()
            .map_err(|_| Error::InvalidToml(cargo_toml_file.to_string_lossy().into_owned()))?;

        println!(
            "{}: {} {:?}",
            cargo_toml_file.to_string_lossy(),
            project_name,
            cargo_toml["dependencies"].get(&project_name)
        );
        if let Some(this_dep) = cargo_toml["dependencies"].get_mut(&project_name) {
            match this_dep {
                toml_edit::Item::Value(s @ toml_edit::Value::String(_)) => {
                    *s = new_version.clone().into()
                }
                toml_edit::Item::Value(toml_edit::Value::InlineTable(t)) => {
                    t["version"] = new_version.clone().into()
                }
                toml_edit::Item::None => continue,
                _ => {
                    return Err(Error::InvalidToml(format!(
                        "{:?} while seaching dependencies in {}",
                        this_dep,
                        cargo_toml_file.to_string_lossy()
                    )))
                }
            }
        }

        std::fs::write(cargo_toml_file, cargo_toml.to_string())
            .map_err(|_| Error::WriteTomlFile)?;
    }

    Ok(())
}

fn execute_git_command(root_directory: &Path, args: &[&str]) -> Result<String, Error> {
    let git_cmd = Command::new("git")
        .args(args)
        .current_dir(root_directory)
        .output()
        .map_err(|_| Error::Git("Failed to run command"))?;

    String::from_utf8(git_cmd.stdout).map_err(|_| Error::Git("Output not utf-8"))
}

fn glob_many(root_directory: &Path, globs: &[&str]) -> Result<Vec<std::path::PathBuf>, Error> {
    let mut result = vec![];

    for g in globs.iter() {
        for path in glob::glob(&root_directory.join(g).to_string_lossy()).expect("Invalid glob") {
            result.push(path.map_err(|_| Error::Glob)?);
        }
    }

    Ok(result)
}

#[derive(Debug)]
pub enum Error {
    FindRootDirectory,
    Git(&'static str),
    Glob,
    ReadTomlFile,
    InvalidToml(String),
    WriteTomlFile,
    JustCiFailed,
}

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

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
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
