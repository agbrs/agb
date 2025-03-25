use std::env;

use clap::{Arg, ArgAction, ArgMatches};
use serde::Deserialize;
use xshell::{Shell, cmd};

pub fn command() -> clap::Command {
    clap::Command::new("deploy").arg(
        Arg::new("Dry run")
            .long("dry-run")
            .help("Don't actually deploy")
            .action(ArgAction::SetTrue),
    )
}

const LAUNCH_SCRIPT: &str = r#"
#!/usr/bin/env bash

apt-get install unattended-upgrades docker.io -y --no-install-recommends

cat >>/etc/docker/daemon.json <<EOF
{
    "storage-driver": "overlay2"
}
EOF

systemctl restart docker

docker pull ghcr.io/agbrs/playground-builder:latest
docker network create agbnet

docker run -v /run/docker.sock:/run/docker.sock \
    -v /run/agbrs-playground:/run/agbrs-playground \
    --detach --restart unless-stopped \
    --init --network=agbnet \
    --name=playground-server \
    ghcr.io/agbrs/playground-server:latest

docker run \
    --network=agbnet \
    --detach --restart unless-stopped \
    --name=cloudflare-tunnel \
    cloudflare/cloudflared:latest tunnel --no-autoupdate run --token \
    $CLOUDFLARE_TOKEN
"#;

pub fn deploy(matches: &ArgMatches) -> Result<(), Error> {
    let sh = Shell::new()?;

    let dry_run = *matches.get_one::<bool>("Dry run").expect("defined by clap");

    let cloudflare_token = match env::var("AGB_CLOUDFLARE_TUNNEL_SECRET") {
        Ok(secret) => secret,
        Err(_) => {
            if dry_run {
                "<cloudflare token>".to_string()
            } else {
                return Err(Error::MissingCloudflareSecret);
            }
        }
    };

    let existing_droplets = cmd!(
        sh,
        "doctl -o json compute droplet list --tag-name playground"
    )
    .quiet()
    .read()?;

    #[derive(Deserialize)]
    struct DropletListResult {
        name: String,
        id: i64,
    }

    let existing_droplets: Vec<DropletListResult> = serde_json::from_str(&existing_droplets)?;

    eprintln!(
        "Existing playground droplets: {}",
        existing_droplets
            .iter()
            .map(|r| r.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );

    let launch_script = LAUNCH_SCRIPT.replace("$CLOUDFLARE_TOKEN", &cloudflare_token);

    let timestamp = std::time::UNIX_EPOCH.elapsed().unwrap().as_secs();
    let new_droplet_name = format!("agb-playground-{timestamp}");

    if dry_run {
        eprintln!("Would create droplet with name {new_droplet_name}");
    } else {
        cmd!(
            sh,
            "doctl compute droplet create
                -o json
                --image debian-12-x64 --size s-1vcpu-512mb-10gb 
                --enable-monitoring --region ams3 --wait --user-data {launch_script}
                --tag-names playground {new_droplet_name}"
        )
        .quiet()
        .run()?;

        eprintln!("Created droplet {new_droplet_name}");
    }

    if dry_run {
        eprintln!("Would delete existing droplets");
    } else {
        for droplet_to_delete in &existing_droplets {
            let droplet_id = droplet_to_delete.id.to_string();
            eprintln!(
                "Deleting droplet {} with id {}",
                droplet_to_delete.name, droplet_to_delete.id
            );

            cmd!(sh, "doctl compute droplet delete -f {droplet_id}").run()?;
        }
    }

    Ok(())
}

#[derive(Debug)]
pub enum Error {
    MissingCloudflareSecret,
    XShell(xshell::Error),
    JsonDeserialize(serde_json::Error),
}

impl From<xshell::Error> for Error {
    fn from(value: xshell::Error) -> Self {
        Self::XShell(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::JsonDeserialize(value)
    }
}
