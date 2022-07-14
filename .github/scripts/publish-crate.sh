#!/usr/bin/env bash

set -e # Fail if any command fails

function wait_for_release() {
   local package="$0"
   local package_with_underscores="${package/-/_}"
   
   local first_two_characters="${package_with_underscores:0:2}"
   local second_two_characters="${package_with_underscores:2:2}"

   local path="$first_two_characters/$second_two_characters"

   if [ "$package" == "agb" ]; then
      path="3/a"
   fi

   local url_to_poll="https://raw.githubusercontent.com/rust-lang/crates.io-index/master/$path/$package_with_underscores"

   local expected_version
   expected_version=$(grep -E '^version' Cargo.toml | grep -oE '[0-9.]+')

   local attempts=1

   while [ $attempts -le 15 ]; do
      echo "Polling crates.io to see if the version has updated (attempt $attempts)"
      if curl "$url_to_poll" | grep "$expected_version"; then
         return
      fi

      sleep 30s
      attempts=$((attempts + 1))
   done
}

PROJECTS_TO_RELEASE_IN_ORDER="agb-macros agb-fixnum agb-image-converter agb-sound-converter agb"

for PROJECT in $PROJECTS_TO_RELEASE_IN_ORDER; do
   pushd "$PROJECT"
   echo "Publishing $PROJECT"
   cargo publish
   wait_for_release "$PROJECT"
   popd
done
