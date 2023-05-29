alias t := test
alias b := build
alias br := build-release


test:
	cargo test

build:
	cargo build

build-release:
	cargo build --release