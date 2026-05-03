default:
    @just --list

build:
    cargo build

release:
    cargo build --release
