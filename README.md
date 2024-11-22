# Raphael XIV [<img src="https://img.shields.io/discord/1244140502643904522?logo=discord&logoColor=white"/>](https://discord.com/invite/m2aCy3y8he)

:link: [www.raphael-xiv.com](https://www.raphael-xiv.com/)

Raphael is a crafting rotation solver for the online game Final Fantasy XIV.
* Produces optimal solutions.
* Short solve time (20-60 seconds) and reasonable memory usage (300-1000 MB) for most configurations.

## Contents <!-- omit in toc -->

* [How does it work?](#how-does-it-work)
* [Building from source](#building-from-source)
  * [Native app](#native-app)
  * [Web app (WASM)](#web-app-wasm)
  * [Native CLI](#native-cli)

## How does it work?

* Short answer: [A* search](https://en.wikipedia.org/wiki/A*_search_algorithm) + [Pareto optimization](https://en.wikipedia.org/wiki/Multi-objective_optimization) + [Dynamic programming](https://en.wikipedia.org/wiki/Dynamic_programming).
* Long answer: coming soon<sup>tm</sup>

## Building from source

The [Rust](https://www.rust-lang.org/) toolchain is required to build the solver.

### Native app

To build and run the application:

```
cargo run --release
```

### Web app (WASM)

[Trunk](https://trunkrs.dev/) is required to bundle and host the website and can be installed via the Rust toolchain:

```
cargo install --locked trunk
```

To build and host the application locally:

```
export RANDOM_SUFFIX=""
export RUSTFLAGS="--cfg=web_sys_unstable_apis"
trunk serve --release --dist distrib
```

### Native CLI

To build and run the command-line interface (CLI):

```
cargo run --release --package raphael-cli -- <cli-args>
```

The CLI currently supports searching for items and solving for crafting rotations. Run the following to see the relevant help messages:
```
cargo run --release --package raphael-cli -- --help
cargo run --release --package raphael-cli -- search --help
cargo run --release --package raphael-cli -- solve --help
```

Some examples:
```
cargo run --release --package raphael-cli -- search 'Archeo Fending'
cargo run --release --package raphael-cli -- solve --item-id 8548 --stats 5000 4000 500
```

The CLI can also be installed so that it can be called from anywhere:

```
cargo install --path raphael-cli
```
