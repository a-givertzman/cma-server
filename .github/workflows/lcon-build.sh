#!/bin/bash

# rustup install nightly
# rustup default nightly
# rustup default stable

# cargo add llvm-tools-preview

# export CARGO_INCREMENTAL=0
export RUSTFLAGS='-Cinstrument-coverage'
export LLVM_PROFILE_FILE='target/coverage/%p-%m.profraw'
# cargo test
# cargo clean
# cargo build
rm -rf ./target/coverage
# cargo test --release --no-fail-fast -- --test-threads=1
cargo test --release --no-fail-fast

# grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./target/debug/coverage/
# grcov . -s . --binary-path ./target/release/ -t html --branch --ignore-not-existing -o ./target/release/coverage/

# grcov target/coverage -s . --binary-path target/debug -o target/coverage --keep-only 'src/*' --output-types html,cobertura
grcov target/coverage -s . --binary-path target/release -o target/coverage --keep-only 'src/*' --output-types html,cobertura,covdir,'coveralls+' --ignore 'src/tests/*'
    # Where the source directory is expected
        
    # This path must match the setting in LLVM_PROFILE_FILE. If you're not getting the coverage
    # you expect, look for '.profraw' files in other directories.
        
    # If your target dir is modified, this will need to match...
        
        # --binary-path target/debug
    # Where to write the output; this should be a directory that exists.
        
    # Exclude coverage of crates and Rust stdlib code. If you get unexpected coverage results from
    # this (empty, for example), try different combinations of '--ignore-not-existing',
    # '--ignore "$HOME/.cargo/**"' and see what kind of filtering gets you the coverage you're
    # looking for.
        
    # Doing both isn't strictly necessary, if you won't use the HTML version you can modify this
    # line.
