#!/bin/bash

echo "Run Fmt+Clippy Fix ...."

cargo fix

cargo +nightly fmt --all

cargo clippy --fix --all-targets -- -D warnings