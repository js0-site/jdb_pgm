#!/usr/bin/env bash

set -e
DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR
set -x

RUSTFLAGS="-C target-cpu=native" cargo run --bin count_keys --release
