use std::{fs, io, path::Path};

use clap::ArgMatches;
use ninja_writer::*;

use crate::utils::{FindRootDirectoryError, find_agb_root_directory};

pub fn command() -> clap::Command {
    clap::Command::new("configure")
}

pub fn configure(_arg_matches: &ArgMatches) -> Result<(), Error> {
    let root_directory = find_agb_root_directory()?;

    let ninja = Ninja::new();

    ninja.variable("builddir", root_directory.join("target"));

    let agb_metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(root_directory.join("agb/Cargo.toml"))
        .exec()?;

    let agb_package = agb_metadata
        .packages
        .iter()
        .find(|pkg| pkg.name.as_ref() == "agb")
        .unwrap();

    agb_examples(&ninja, &root_directory, agb_package);
    agb_tests(&ninja, &root_directory);
    agb_build(&ninja, &root_directory);

    let make_ninja_file = ninja.rule("make_ninja_file", "$in configure");

    make_ninja_file
        .build(["build.ninja"])
        .with([root_directory.join("target/debug/tools")]);

    let cargo_build_bin = ninja
        .cargo_command(
            "build_workspace_bin",
            &root_directory,
            ".",
            "build --bin=$bin",
        )
        .depfile(root_directory.join("target/debug/$bin.d"));
    cargo_build_bin
        .build([root_directory.join("target/debug/tools")])
        .variable("bin", "tools");

    fs::write(root_directory.join("build.ninja"), ninja.to_string())?;

    Ok(())
}

fn agb_examples(ninja: &Ninja, root_directory: &Path, agb_package: &cargo_metadata::Package) {
    let agb_examples_rule = ninja
        .cargo_command(
            "agb_examples",
            root_directory,
            "agb",
            "build -q --examples $cargo_flags && cat $depfiles > $depfile && touch $out",
        )
        .depfile("$depfile")
        .description("Building agb examples");

    let example_names = agb_package
        .targets
        .iter()
        .filter(|target| target.is_example())
        .map(|example| example.name.clone())
        .collect::<Vec<_>>();

    let make_examples = move |dir: &Path| {
        let example_exes = example_names
            .iter()
            .map(|name| dir.join(name))
            .collect::<Vec<_>>();
        agb_examples_rule
            .build(&example_exes)
            .variable("depfile", dir.join("examples.d"))
            .variable(
                "depfiles",
                example_exes
                    .iter()
                    .map(|exe| String::from(exe.with_extension("d").to_string_lossy()))
                    .collect::<Vec<_>>()
                    .join(" "),
            );
    };

    let thumbv4_dir = root_directory.join("agb/target/thumbv4t-none-eabi");
    let debug_examples_dir = thumbv4_dir.join("debug/examples");
    let release_examples_dir = thumbv4_dir.join("release/examples");

    make_examples(&debug_examples_dir);
    make_examples(&release_examples_dir);
}

fn agb_build(ninja: &Ninja, root_directory: &Path) {
    let build_agb = ninja.cargo_command("build_agb", root_directory, "agb", "build $cargo_flags");

    let build_agb_phony = ninja.phony(["build_agb"]);

    let build_agb_case = |name: &str, flags: &str| {
        build_agb.build([name]).variable("cargo_flags", flags);
        build_agb_phony.dependencies.add(name.to_owned());
    };

    for release in [true, false] {
        for target in ["thumbv4t-none-eabi", "armv4t-none-eabi"] {
            let build_output_name = format!(
                "agb_build_{}_{target}",
                if release { "release" } else { "debug" }
            );
            let flags = format!(
                "--target={target}{}",
                if release { " --release" } else { "" }
            );

            build_agb_case(&build_output_name, &flags);
        }
    }

    build_agb_case("agb_build_without_features", "--no-default-features");
    build_agb_case(
        "agb_build_just_testing",
        "--no-default-features --features=testing",
    )
}

fn agb_tests(ninja: &Ninja, root_directory: &Path) {
    let agb_test_rule = ninja.cargo_command("agb_test", root_directory, "agb", "test $cargo_flags");

    let test_agb = ninja.phony(["test"]);

    for release in [true, false] {
        for target in ["thumbv4t-none-eabi", "armv4t-none-eabi"] {
            let build_output_name = format!(
                "agb_test_{}_{target}",
                if release { "release" } else { "debug" }
            );
            agb_test_rule.build([&build_output_name]).variable(
                "cargo_flags",
                format!(
                    "--target={target}{}",
                    if release { " --release" } else { "" }
                ),
            );

            test_agb.dependencies.add(build_output_name);
        }
    }

    let agb_test_multiboot_rule = ninja.cargo_command(
        "agb_test_multiboot",
        root_directory,
        "agb",
        CargoCommand {
            env_vars: Some("AGB_MULTIBOOT=true".to_owned()),
            command: "test --features=multiboot --test=test_multiboot".to_owned(),
        },
    );

    agb_test_multiboot_rule.build(["agb_test_multiboot"]);
    test_agb.dependencies.add("agb_test_multiboot".to_owned());
}

trait NinjaExt {
    fn cargo_command(
        &self,
        name: &str,
        root_directory: &Path,
        directory: impl AsRef<Path>,
        command: impl Into<CargoCommand>,
    ) -> RuleRef;
}

impl NinjaExt for Ninja {
    fn cargo_command(
        &self,
        name: &str,
        root_directory: &Path,
        directory: impl AsRef<Path>,
        command: impl Into<CargoCommand>,
    ) -> RuleRef {
        let cargo_command = command.into();

        self.rule(
            name,
            format!(
                "cd {} && CARGO_TARGET_DIR={} {} cargo {}",
                root_directory.join(directory).display(),
                root_directory.join("target").display(),
                cargo_command.env_vars.unwrap_or(String::new()),
                cargo_command.command
            ),
        )
    }
}

struct CargoCommand {
    env_vars: Option<String>,
    command: String,
}

impl From<&str> for CargoCommand {
    fn from(value: &str) -> Self {
        Self {
            env_vars: None,
            command: value.to_owned(),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    FindRootDirectory,
    CargoMetadata(cargo_metadata::Error),
    Io(io::Error),
}

impl From<FindRootDirectoryError> for Error {
    fn from(_value: FindRootDirectoryError) -> Self {
        Self::FindRootDirectory
    }
}

impl From<cargo_metadata::Error> for Error {
    fn from(value: cargo_metadata::Error) -> Self {
        Self::CargoMetadata(value)
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
