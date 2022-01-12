#!/usr/bin/env bash

# Fail if any command fails
set -e

PROJECT=$1
VERSION=$2
NO_COMMIT=$3

# Sanity check that we actually have a version
if [ "$VERSION" = "" ]; then
    echo "Usage $0 <project> <version> [--no-commit]"
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

# Set up $DIRECTORY and $TAGNAME
case "$PROJECT" in
    agb)
        DIRECTORY="agb"
        TAGNAME="v$VERSION"
        ;;
    agb-*)
        if [ -f "$PROJECT/Cargo.toml" ]; then
            DIRECTORY=$PROJECT
            TAGNAME="$PROJECT/v$VERSION"
        else
            echo "Unknown project name $PROJECT"
            exit 1
        fi
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
if [ -n "$(git status --porcelain)" ]; then
    echo "Uncommitted changes, please commit first"
    exit 1
fi

# Check that we are in the master branch, but only if actually committing
if [ ! "$NO_COMMIT" = "--no-commit" ] && [ "$(git symbolic-ref --short HEAD)" != "master" ]; then
    echo "You must be in the master branch before releasing"
    exit 1
fi

# Update the version in Cargo.toml
sed -i -e "s/^version = \".*\"/version = \"$VERSION\"/" "$DIRECTORY/Cargo.toml"

# Also update the lock file
(cd "$DIRECTORY" && cargo update)
git add "$DIRECTORY/Cargo.toml" "$DIRECTORY/Cargo.lock"

if [ "$PROJECT" = "agb" ]; then
    # also update the agb version in the template and the examples
    sed -i -e "s/^agb = \".*\"/agb = \"$VERSION\"/" template/Cargo.toml
    git add template/Cargo.toml

    for EXAMPLE_TOML_FILE in examples/*/Cargo.toml book/games/*/Cargo.toml; do
        EXAMPLE_DIR=$(dirname "$EXAMPLE_TOML_FILE")
        sed -E -i -e "/agb =/ s/version = \"[^\"]+\"/version = \"$VERSION\"/" "$EXAMPLE_DIR/Cargo.toml"
        (cd "$EXAMPLE_DIR" && cargo update)
        git add "$EXAMPLE_DIR"/{Cargo.toml,Cargo.lock}
    done
else
    PROJECT_NAME_WITH_UNDERSCORES=$(echo -n "$PROJECT" | tr - _)

    for CARGO_TOML_FILE in agb-*/Cargo.toml agb/Cargo.toml examples/*/Cargo.toml book/games/*/Cargo.toml; do
        sed -i -E -e "s/($PROJECT_NAME_WITH_UNDERSCORES = .*version = \")[^\"]+(\".*)/\1$VERSION\2/" "$CARGO_TOML_FILE"
        (cd "$(dirname "$CARGO_TOML_FILE")" && cargo generate-lockfile)

        git add "$CARGO_TOML_FILE" "${CARGO_TOML_FILE/.toml/.lock}"
    done
fi

# Sanity check to make sure the build works
for CARGO_TOML_FILE in agb-*/Cargo.toml agb/Cargo.toml; do
    (cd "$(dirname "$CARGO_TOML_FILE")" && cargo test)
done

for EXAMPLE_TOML_FILE in examples/*/Cargo.toml book/games/*/Cargo.toml; do
    EXAMPLE_DIR=$(dirname "$EXAMPLE_TOML_FILE")
    (cd "$EXAMPLE_DIR" && cargo check --release)
done

if [ ! "$NO_COMMIT" = "--no-commit" ]; then
    # Commit the Cargo.toml changes
    git commit -m "Release $PROJECT v$VERSION"

    # Tag the version
    git tag -a "$TAGNAME" -m "$PROJECT - v$VERSION"

    echo "Done! Push with"
    echo "git push --atomic origin master $TAGNAME"
fi