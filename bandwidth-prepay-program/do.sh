#!/usr/bin/env bash

cd "$(dirname "$0")"

usage() {
    cat <<EOF

Usage: do.sh action <project>

If relative_project_path is omitted then action will
be performed on all projects

Supported actions:
    build
    clean
    test
    clippy
    fmt

EOF
}

sdkDir=../node_modules/@solana/web3.js/bpf-sdk
targetDir="$PWD"/target
profile=bpfel-unknown-unknown/release

perform_action() {
    set -e
    case "$1" in
    build)
        "$sdkDir"/rust/build.sh "$PWD"
        mkdir -p ../dist/programs

        so_path="$targetDir/$profile"
        so_name="bandwidth_prepay.so"
        if [ -f "$so_path/$so_name" ]; then
            "$sdkDir"/dependencies/llvm-native/bin/llvm-objcopy --strip-all "$so_path/${so_name}" "$so_path/$so_name"
            cp "$so_path/$so_name" ../dist/programs/"$so_name"
        fi
        ;;
    clean)
        "$sdkDir"/rust/clean.sh "$PWD"
        rm -f ../dist/config.json
        ;;
    test)
        echo "test"
        cargo +nightly test
        ;;
    clippy)
        echo "clippy"
        cargo +nightly clippy
        ;;
    fmt)
        echo "formatting"
        cargo fmt
        ;;
    help)
        usage
        exit
        ;;
    *)
        echo "Error: Unknown command"
        usage
        exit
        ;;
    esac
}

set -e

perform_action "$1"
