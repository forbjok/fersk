Fersk
=====

[![CI](https://github.com/forbjok/fersk/actions/workflows/ci.yml/badge.svg)](https://github.com/forbjok/fersk/actions/workflows/ci.yml)
![GitHub release (latest by date)](https://img.shields.io/github/v/release/forbjok/fersk)

## Introduction

Fersk is a convenience utility for running a command (ex. a script) in a clean copy of the current git repository without interfering with the original repository's working directory. Mainly useful for local build and deployment operations.

## Installing

Pre-built binaries, and even a Chocolatey package (not currently published to the official Chocolatey repository or planned to be), can be downloaded from [Releases](https://github.com/forbjok/fersk/releases).

## Building
1. Install Rust using the instructions [here](https://www.rust-lang.org/tools/install) or your distro's package manager.
2. Clone this repository and execute the following command in it:
```
$ cargo build --release
```

Voila! You should now have a usable executable in the `target/release` subdirectory.
