use clap::{Arg, ArgAction, ArgMatches};
use dependency_graph::DependencyGraph;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
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

struct Package {
    name: String,
    dependencies: Vec<String>,
    directory: PathBuf,
}

impl dependency_graph::Node for Package {
    type DependencyType = String;

    fn dependencies(&self) -> &[Self::DependencyType] {
        &self.dependencies
    }

    fn matches(&self, dependency: &Self::DependencyType) -> bool {
        &self.name == dependency
    }
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
    let dry_run = if *dry_run { vec!["--dry-run"] } else { vec![] };

    let root_directory = find_agb_root_directory().map_err(|_| Error::FindRootDirectory)?;

    let mut in_progress: HashMap<_, RefCell<std::process::Child>> = HashMap::new();

    let mut dependencies = build_dependency_graph(&root_directory)?;
    let mut tracker_dependencies = build_dependency_graph(&root_directory.join("tracker"))?;

    dependencies.append(&mut tracker_dependencies);

    let graph = DependencyGraph::from(&dependencies[..]);

    for package in graph {
        let package = package.as_resolved().unwrap();

        for dep in &package.dependencies {
            assert!(in_progress
                .get(dep)
                .unwrap()
                .borrow_mut()
                .wait()
                .map_err(|_| Error::PublishCrate)?
                .success());
        }

        println!("Publishing {}", package.name);

        let publish_cmd = Command::new("cargo")
            .arg("publish")
            .args(&dry_run)
            .current_dir(&package.directory)
            .spawn()
            .map_err(|_| Error::PublishCrate)?;

        in_progress.insert(package.name.clone(), RefCell::new(publish_cmd));
    }

    for (_, in_progress) in in_progress {
        assert!(in_progress
            .borrow_mut()
            .wait()
            .map_err(|_| Error::PublishCrate)?
            .success());
    }

    Ok(())
}

fn build_dependency_graph(root: &Path) -> Result<Vec<Package>, Error> {
    let dirs = fs::read_dir(root).map_err(|_| Error::ReadingDependencies)?;
    let mut packages = vec![];

    for dir in dirs {
        let dir = dir.map_err(|_| Error::ReadingDependencies)?;
        if !dir
            .file_type()
            .map_err(|_| Error::ReadingDependencies)?
            .is_dir()
        {
            continue;
        }

        if !dir.file_name().to_string_lossy().starts_with("agb") {
            continue;
        }

        let crate_path = root.join(dir.path());

        packages.push(Package {
            name: dir.file_name().to_string_lossy().to_string(),
            dependencies: get_agb_dependencies(&crate_path)?,
            directory: crate_path,
        });
    }

    Ok(packages)
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
    fn should_detect_dependencies() -> Result<(), Error> {
        let root_directory = crate::utils::find_agb_root_directory().unwrap();
        let deps = get_agb_dependencies(&root_directory.join("agb"))?;

        assert_eq!(
            deps,
            &[
                "agb-image-converter",
                "agb-sound-converter",
                "agb-macros",
                "agb-fixnum",
                "agb-hashmap",
            ]
        );
        Ok(())
    }
}
