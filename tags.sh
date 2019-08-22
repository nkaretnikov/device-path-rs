#!/usr/bin/env bash

set -euxo pipefail

CTAGS_RUST_FILE=ctags.rust
CTAGS_RUST_URL=https://raw.githubusercontent.com/rust-lang/rust/master/src/etc/ctags.rust

if ! [ -f $CTAGS_RUST_FILE ]; then
    curl -o $CTAGS_RUST_FILE $CTAGS_RUST_URL
fi

ctags -f rusty-tags.vi --options=$CTAGS_RUST_FILE --languages=Rust --recurse .
