#!/usr/bin/env bash

echo "Install nightly rust from day of release" 1>&2

LATEST_RELEASE_TAG=$(git describe --tags --abbrev=0)
LATEST_RELEASE_DATE=$(git log --format=%as "$LATEST_RELEASE_TAG" -1)

NIGHTLY_VERSION="$LATEST_RELEASE_DATE"

TRIES_REMAINING=10
while true; do
    URL="https://static.rust-lang.org/dist/$NIGHTLY_VERSION"

    echo "Checking $URL" 1>&2
    STATUS_CODE=$(curl -s -o /dev/null -w "%{http_code}" --head "$URL")

    if [[ "$STATUS_CODE" -eq 200 ]]; then
        echo "Found valid rust nightly version $NIGHTLY_VERSION" 1>&2
        break
    fi

    NIGHTLY_VERSION=$(date -d "$NIGHTLY_VERSION - 1 day" +%Y-%m-%d)

    if [ "$TRIES_REMAINING" -ge 0 ]; then
        TRIES_REMAINING=$(( $TRIES_REMAINING - 1 ))
    else
        echo "Failed to find nightly version after 10 tries" 1>&2
        exit 1
    fi
done

echo "$NIGHTLY_VERSION"