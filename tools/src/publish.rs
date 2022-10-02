use clap::{Arg, ArgAction, ArgMatches};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;
use toml_edit::Document;

use crate::utils::*;

#[derive(Debug)]
pub enum Error {
    FindRootDirectory,
    PublishCrate,
    Poll,
    CrateVersion,
    ReadingDependencies,
    CargoToml,
}

pub fn command() -> clap::Command {
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

    let root_directory = find_agb_root_directory().map_err(|_| Error::FindRootDirectory)?;

    let mut fully_published_crates: HashSet<String> = HashSet::new();
    let mut published_crates: HashSet<String> = HashSet::new();

    let dependencies = build_dependency_graph(&root_directory)?;
    let crates_to_publish: Vec<_> = dependencies.keys().collect();

    while published_crates.len() != crates_to_publish.len() {
        // find all crates which can be published now but haven't
        let publishable_crates: Vec<_> = crates_to_publish
            .iter()
            .filter(|&&crate_to_publish| !published_crates.contains(crate_to_publish))
            .filter(|&&crate_to_publish| {
                let dependencies_of_crate = &dependencies[crate_to_publish];
                for dependency_of_crate in dependencies_of_crate {
                    if !fully_published_crates.contains(dependency_of_crate) {
                        return false;
                    }
                }

                true
            })
            .collect();

        for publishable_crate in publishable_crates {
            if *dry_run {
                println!("Would execute cargo publish for {publishable_crate}");
            } else {
                assert!(Command::new("cargo")
                    .arg("publish")
                    .current_dir(&root_directory.join(publishable_crate))
                    .status()
                    .map_err(|_| Error::PublishCrate)?
                    .success());
            }

            published_crates.insert(publishable_crate.to_string());
        }

        for published_crate in published_crates.iter() {
            if !fully_published_crates.contains(published_crate) {
                let expected_version =
                    read_cargo_toml_version(&root_directory.join(published_crate))?;
                if check_if_released(published_crate, &expected_version)? {
                    fully_published_crates.insert(published_crate.clone());
                }
            }
        }

        thread::sleep(Duration::from_secs(10));
    }

    Ok(())
}

fn check_if_released(crate_to_publish: &str, expected_version: &str) -> Result<bool, Error> {
    let url_to_poll = &get_url_to_poll(crate_to_publish);

    println!("Polling crates.io with URL {url_to_poll} for {crate_to_publish} hoping for version {expected_version}.");

    let curl_result = Command::new("curl")
        .arg(url_to_poll)
        .output()
        .map_err(|_| Error::Poll)?;

    Ok(String::from_utf8_lossy(&curl_result.stdout).contains(expected_version))
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
    let cargo_toml = read_cargo_toml(folder)?;

    let version_value = cargo_toml["package"]["version"]
        .as_value()
        .ok_or(Error::CrateVersion)?
        .as_str()
        .ok_or(Error::CrateVersion)?;

    Ok(version_value.to_owned())
}

fn build_dependency_graph(root: &Path) -> Result<HashMap<String, Vec<String>>, Error> {
    let mut result = HashMap::new();
    result.insert("agb".to_owned(), get_agb_dependencies(&root.join("agb"))?);

    let mut added_new_crates = true;
    while added_new_crates {
        added_new_crates = false;

        let all_crates: HashSet<String> = HashSet::from_iter(result.values().flatten().cloned());

        for dep_crate in all_crates {
            if result.contains_key(&dep_crate) {
                continue;
            }

            added_new_crates = true;
            result.insert(
                dep_crate.to_owned(),
                get_agb_dependencies(&root.join(dep_crate))?,
            );
        }
    }

    Ok(result)
}

fn get_agb_dependencies(folder: &Path) -> Result<Vec<String>, Error> {
    let cargo_toml = read_cargo_toml(folder)?;

    let dependencies = cargo_toml["dependencies"]
        .as_table()
        .ok_or(Error::ReadingDependencies)?
        .get_values();

    let mut result = vec![];

    for (key, _) in dependencies {
        let dep = key[0].get();
        if dep.starts_with("agb") {
            result.push(dep.replace('_', "-"))
        }
    }

    Ok(result)
}

fn read_cargo_toml(folder: &Path) -> Result<Document, Error> {
    let cargo_toml_contents =
        fs::read_to_string(folder.join("Cargo.toml")).map_err(|_| Error::CargoToml)?;
    let cargo_toml: Document = cargo_toml_contents.parse().map_err(|_| Error::CargoToml)?;
    Ok(cargo_toml)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn verify_cli() {
        command().debug_assert();
    }

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
    fn should_read_version() -> Result<(), Error> {
        let root_directory = crate::utils::find_agb_root_directory().unwrap();
        let my_version = read_cargo_toml_version(&root_directory.join("tools"))?;

        assert_eq!(my_version, "0.1.0");
        Ok(())
    }

    #[test]
    fn should_detect_dependencies() -> Result<(), Error> {
        let root_directory = crate::utils::find_agb_root_directory().unwrap();
        let deps = get_agb_dependencies(&root_directory.join("agb"))?;

        assert_eq!(
            deps,
            &[
                "agb-image-converter",
                "agb-sound-converter",
                "agb-macros",
                "agb-fixnum"
            ]
        );
        Ok(())
    }
}
