#!/usr/bin/env bash

# Fail if any command fails
set -e

VERSION=$1
NO_COMMIT=$2

# Sanity check that we actually have a version
if [ "$VERSION" = "" ]; then
    echo "Usage $0 <version> [--no-commit]"
    exit 1
fi

# Check the format of version
if echo "$VERSION" | grep -q -Ev "^[0-9]+\.[0-9]+\.[0-9]+$"; then
    echo "Version must be of the form x.y.z, got $VERSION"
    exit 1
fi

# Check if no commit option is valid
if [ ! "$NO_COMMIT" = "" ] && [ ! "$NO_COMMIT" = "--no-commit" ]; then
    echo "Must pass either no last argument or --no-commit"
    exit 1
fi

function maybe_git() {
    if [ "$NO_COMMIT" = "--no-commit" ]; then
        echo "Would run: git $*"
    else
        git "$@"
    fi
}

# Check that no out-standing changes in git
if [ "$NO_COMMIT" = "" ] && [ -n "$(git status --porcelain)" ]; then
    echo "Uncommitted changes, please commit first"
    exit 1
fi

# Check that we are in the master branch, but only if actually committing
if [ ! "$NO_COMMIT" = "--no-commit" ] && [ "$(git symbolic-ref --short HEAD)" != "master" ]; then
    echo "You must be in the master branch before releasing"
    exit 1
fi

TAGNAME="v$VERSION"

for PROJECT_TOML_FILE in agb/Cargo.toml agb-*/Cargo.toml template/Cargo.toml; do
    DIRECTORY=$(dirname "$PROJECT_TOML_FILE")

    # Update the version in Cargo.toml
    sed -i -e "s/^version = \".*\"/version = \"$VERSION\"/" "$DIRECTORY/Cargo.toml"

    # Also update the lock file
    (cd "$DIRECTORY" && cargo update)

    if [ "$DIRECTORY" = "agb" ]; then
        # also update the agb version in the template and the examples
        sed -i -e "s/^agb = \".*\"/agb = \"$VERSION\"/" template/Cargo.toml

        for EXAMPLE_TOML_FILE in examples/*/Cargo.toml book/games/*/Cargo.toml; do
            EXAMPLE_DIR=$(dirname "$EXAMPLE_TOML_FILE")
            sed -E -i -e "/agb =/ s/version = \"[^\"]+\"/version = \"$VERSION\"/" "$EXAMPLE_DIR/Cargo.toml"
            (cd "$EXAMPLE_DIR" && cargo update)
        done
    else
        PROJECT_NAME_WITH_UNDERSCORES=$(echo -n "$DIRECTORY" | tr - _)

        for CARGO_TOML_FILE in agb-*/Cargo.toml agb/Cargo.toml examples/*/Cargo.toml book/games/*/Cargo.toml; do
            sed -i -E -e "s/($PROJECT_NAME_WITH_UNDERSCORES = .*version = \")[^\"]+(\".*)/\1$VERSION\2/" "$CARGO_TOML_FILE"
            (cd "$(dirname "$CARGO_TOML_FILE")" && cargo generate-lockfile)
        done
    fi
done

# Sanity check to make sure the build works
just ci

for EXAMPLE_TOML_FILE in examples/*/Cargo.toml book/games/*/Cargo.toml; do
    EXAMPLE_DIR=$(dirname "$EXAMPLE_TOML_FILE")
    (cd "$EXAMPLE_DIR" && cargo check --release)
done

# Commit the Cargo.toml changes
maybe_git commit -am "Release v$VERSION"

# Tag the version
maybe_git tag -a "$TAGNAME" -m "v$VERSION"

echo "Done! Push with"
echo "git push --atomic origin master $TAGNAME"
