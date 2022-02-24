#!/usr/bin/env bash

function update_lockfiles() {
    for toml in **/Cargo.toml; do 
        (cd "$(dirname "$toml")" && cargo generate-lockfile)
    done
}

update_lockfiles
update_lockfiles
update_lockfiles
update_lockfiles

git -c user.name="GBA bot" -c user.email="gw@ilym.me" commit -a -m 'Update lockfiles'
git push
