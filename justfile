alias t := test
alias b := build
alias br := build-release
alias c := clippy
alias d := doc


test:
	cargo test --all-features -- --nocapture

clippy:
	cargo clippy --all-features

build:
	cargo build

build-release:
	cargo build --release

doc:
	cargo doc --all-features --open