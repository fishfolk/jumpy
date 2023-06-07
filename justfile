# This is a justfile. See https://github.com/casey/just

list:
    just --list

check:
    cargo clippy -- -W clippy::correctness -D warnings
    cargo fmt --check

build:
    cargo build

build-release:
    cargo build --release

build-web basepath='':
    ./scripts/build-web.sh

build-release-web basepath='':
    ./scripts/build-web.sh release

docs:
    RUSTDOCFLAGS="--html-in-header docs/rustdoc-mermaid-head.html" \
    cargo +nightly doc --document-private-items --workspace --no-deps
    for link in `grep -hor 'click .* call docLink\(.*\)' target/doc | awk -F '(' '{ print $2 }' | tr -d ')'`; do \
        if [ ! -e "target/doc/$link" ]; then \
            echo "broken doc link in diagram: $link" 1>&2; \
            exit 1; \
        fi \
    done

run *args:
    cargo run -- {{args}}

run-web port='4000' host='127.0.0.1': build-web
    @echo "Debug link: http://{{host}}:{{port}}?RUST_LOG=debug"
    basic-http-server -a '{{host}}:{{port}}' -x web-target/wasm-debug

run-release-web port='4000' host='127.0.0.1': build-release-web
    @echo "Debug link: http://{{host}}:{{port}}?RUST_LOG=debug"
    basic-http-server -a '{{host}}:{{port}}' -x web-target/wasm-release
