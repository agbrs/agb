#!/usr/bin/env bash

# Fail if any command fails
set -e

PROJECT=$1
VERSION=$2

# Sanity check that we actually have a version
if [ "$VERSION" = "" ]; then
    echo "Usage $0 <project> <version>"
    exit 1
fi

# Check the format of version
if [ ! "$(echo "$VERSION" | grep -E "^[0-9]+\.[0-9]+\.[0-9]+$")" ]; then
    echo "Version must be of the form x.y.z, got $VERSION"
    exit 1
fi

# Set up $DIRECTORY and $TAGNAME
case "$PROJECT" in
    agb)
        DIRECTORY="agb"
        TAGNAME="v$VERSION"
        ;;
    agb-image-converter)
        DIRECTORY="agb-image-converter"
        TAGNAME="agb-image-converter/v$VERSION"
        ;;
    mgba-test-runner)
        DIRECTORY="mgba-test-runner"
        TAGNAME="mgba-test-runner/v$VERSION"
        ;;

    *)
        echo "Unknown project name $PROJECT"
        exit 1
        ;;
esac

# Check that no out-standing changes in git
if [ ! -z "$(git status --porcelain)" ]; then
    echo "Uncommitted changes, please commit first"
    exit 1
fi

# Check that we are in the master branch
if [ "$(git symbolic-ref --short HEAD)" != "master" ]; then
    echo "You must be in the master branch before releasing"
    exit 1
fi

# Sanity check to make sure the build works
(cd agb && cargo test)
(cd agb-image-converter && cargo test)

# Update the version in Cargo.toml
sed -i -e "s/^version = .*/version = $VERSION/" "$DIRECTORY/Cargo.toml"

# Also update the lock file
(cd "$DIRECTORY" && cargo update)
git add "$DIRECTIORY/Cargo.toml"

if [ "$PROJECT" = "agb" ]; then
    # also update the agb version in the template
    sed -i -e "s/agb = \"\(.*\)\"/$VERSION/" template/Cargo.toml
    git add template/Cargo.toml
fi

# Commit the Cargo.toml changes
git commit -m "Release $PROJECT v$VERSION"

# Tag the version
git tag -a $TAGNAME -m "$PROJECT - v$VERSION"

echo "Done! Push with"
echo "git push origin $TAGNAME"