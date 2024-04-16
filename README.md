# RsEPL

**RsEPL** or **RsPL** is a cargo subcommand to run a Rust REPL, *with access to your crate*.

Works as a regular rust REPL, but if run inside a crate, you can also access your crate's functions and dependencies.

## Example

```shell
$ cargo new --lib foo
$ cd foo
$ cargo rspl

foo> foo::add(1, 2)
3
```

## Installation

Install with cargo:

```shell
cargo install rsepl
```

This adds the `cargo-rspl` binary to your `~/.cargo/bin` directory.

## Usage

```shell
$ cargo rspl
```

Runs either a regular REPL if not in a crate, or a REPL with the crate and its dependencies available if in a crate.

## Features

Access your crate's methods in a REPL - great for quick testing of functions.

Plus, you get to use dependencies like `rand` or `regex` in the REPL.

## Drawbacks

Recompiles all previously run lines on every new line, so it's not very fast.

Not as fleshed out as some other REPLs like `evcxr`.

# License

All code in this repository is dual-licensed under either:

- MIT License (LICENSE-MIT or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)

at your option. This means you can select the license you prefer.

## Your contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you,
as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
