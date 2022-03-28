#!/usr/bin/env bash

set -e # Fail if any command fails
set -x # print every command before it runs

# Updates the template repository to the content of the template directory
# Requires environment variable
# - GITHUB_USERNAME = a user who has push access to the template repository
# - API_TOKEN_GITHUB = an API token for the user

CLONE_DIR=$(mktemp -d)

git clone --single-branch --branch master "https://$GITHUB_USERNAME:$API_TOKEN_GITHUB@github.com/agbrs/template.git" "$CLONE_DIR"

# Copy the .git directory to a different place so we can ensure that only the relevant template stuff stays
TEMP_DIR=$(mktemp -d)
mv "$CLONE_DIR/.git" "$TEMP_DIR/.git"

cp -rva template/. "$TEMP_DIR"

# git describe will give a unique, friendly name for the current commit to make it easier to track where this came from
VERSION=$(git describe --tags)
COMMIT_MESSAGE="Update to $VERSION"

git -C "$TEMP_DIR" add .
git -C "$TEMP_DIR" -c user.email="gw@ilym.me" -c user.name="GBA bot" commit -m "$COMMIT_MESSAGE"
git -C "$TEMP_DIR" push origin HEAD