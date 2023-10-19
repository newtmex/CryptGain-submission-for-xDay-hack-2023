#!/bin/sh

# builds all wasm targets

set -e

SC_META_FOLDERS=$(find . -name "meta")

TARGET_DIR=$PWD/target
home_dir=$PWD

for sc_meta_folder in $SC_META_FOLDERS; do
    cd "$sc_meta_folder"
    echo "in $sc_meta_folder"

    rm -rf ../output
    cargo run build --target-dir $TARGET_DIR

    cd "$home_dir"
done
