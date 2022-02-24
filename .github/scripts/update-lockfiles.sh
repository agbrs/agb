#!/usr/bin/env bash

function update_lockfiles() {
    find . -name Cargo.lock -execdir cargo update \;
}

update_lockfiles
update_lockfiles
update_lockfiles
update_lockfiles

git add -u

if [ "$(git diff --cached --name-only)" == '' ]; then
    echo "No files updated"
    exit 0
fi

git -c user.name="GBA bot" -c user.email="gw@ilym.me" commit -m 'Update lockfiles'
git push
