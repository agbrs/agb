use clap::{Arg, ArgAction, ArgMatches};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use std::{env, thread};
use toml_edit::Document;

const CRATES_TO_PUBLISH: &[&str] = &[
    "agb-macros",
    "agb-fixnum",
    "agb-image-converter",
    "agb-sound-converter",
    "agb",
];

#[derive(Debug)]
pub enum Error {
    FindRootDirectory,
    PublishCrate,
    Poll,
    CrateVersion,
}

pub fn command() -> clap::Command<'static> {
    clap::Command::new("publish")
        .about("Publishes agb and all subcrates")
        .arg(
            Arg::new("Dry run")
                .long("dry-run")
                .help("Don't actually publish")
                .action(ArgAction::SetTrue),
        )
}

pub fn publish(matches: &ArgMatches) -> Result<(), Error> {
    let dry_run = matches.get_one::<bool>("Dry run").expect("defined by clap");

    let root_directory = find_agb_root_directory()?;

    for crate_to_publish in CRATES_TO_PUBLISH.iter() {
        let crate_dir = root_directory.join(crate_to_publish);

        if *dry_run {
            println!(
                "Would run `cargo publish` in {}",
                crate_dir.to_string_lossy()
            );
        } else {
            let publish_result = Command::new("cargo")
                .arg("publish")
                .current_dir(&crate_dir)
                .spawn();

            if let Err(err) = publish_result {
                println!("Error while publishing crate {crate_to_publish}: {err}");
                return Err(Error::PublishCrate);
            }
        }

        let expected_version = read_cargo_toml_version(&crate_dir)?;
        wait_for_release(crate_to_publish, &expected_version)?;
    }

    Ok(())
}

fn find_agb_root_directory() -> Result<PathBuf, Error> {
    let mut current_path = env::current_dir().map_err(|_| Error::FindRootDirectory)?;

    while !current_path.clone().join("justfile").exists() {
        current_path = current_path
            .parent()
            .ok_or(Error::FindRootDirectory)?
            .to_owned();
    }

    Ok(current_path)
}

fn wait_for_release(crate_to_publish: &str, expected_version: &str) -> Result<(), Error> {
    let url_to_poll = &get_url_to_poll(crate_to_publish);

    for attempt in 0..15 {
        println!(
            "Polling crates.io with URL {url_to_poll} for {crate_to_publish} hoping for version {expected_version}. Attempt {attempt}"
        );

        let curl_result = Command::new("curl")
            .arg(url_to_poll)
            .output()
            .map_err(|_| Error::Poll)?;

        if String::from_utf8_lossy(&curl_result.stdout).contains(expected_version) {
            return Ok(());
        }

        thread::sleep(Duration::from_secs(30));
    }

    Ok(())
}

fn get_url_to_poll(crate_name: &str) -> String {
    let crate_name_with_underscores = crate_name.replace('-', "_");

    let crate_folder = if crate_name_with_underscores.len() == 3 {
        format!("3/{}", crate_name_with_underscores.chars().next().unwrap())
    } else {
        let first_two_characters = &crate_name_with_underscores[0..2];
        let second_two_characters = &crate_name_with_underscores[2..4];

        format!("{first_two_characters}/{second_two_characters}")
    };

    format!("https://raw.githubusercontent.com/rust-lang/crates.io-index/master/{crate_folder}/{crate_name_with_underscores}")
}

fn read_cargo_toml_version(folder: &Path) -> Result<String, Error> {
    let cargo_toml_contents =
        fs::read_to_string(folder.join("Cargo.toml")).map_err(|_| Error::CrateVersion)?;
    let cargo_toml: Document = cargo_toml_contents
        .parse()
        .map_err(|_| Error::CrateVersion)?;

    let version_value = cargo_toml["package"]["version"]
        .as_value()
        .ok_or(Error::CrateVersion)?
        .as_str()
        .ok_or(Error::CrateVersion)?;

    Ok(version_value.to_owned())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn url_to_poll_should_return_correct_url() {
        let test_cases = [
            ["agb", "3/a/agb"],
            ["agb-image-converter", "ag/b_/agb_image_converter"],
            ["agb-fixnum", "ag/b_/agb_fixnum"],
        ];

        for [name, result] in test_cases {
            let url = get_url_to_poll(name);
            assert_eq!(
                url,
                format!(
                    "https://raw.githubusercontent.com/rust-lang/crates.io-index/master/{result}",
                )
            )
        }
    }

    #[test]
    fn should_find_root_directory() -> Result<(), Error> {
        assert_ne!(find_agb_root_directory()?.to_string_lossy(), "");

        Ok(())
    }

    #[test]
    fn should_read_version() -> Result<(), Error> {
        let root_directory = find_agb_root_directory()?;
        let my_version = read_cargo_toml_version(&root_directory.join("tools"))?;

        assert_eq!(my_version, "0.1.0");
        Ok(())
    }
}
