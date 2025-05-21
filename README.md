# Raphael XIV
[<img src="https://img.shields.io/discord/1244140502643904522?label=Discord&logo=discord&logoColor=white"/>](https://discord.com/invite/m2aCy3y8he)
[<img src="https://img.shields.io/github/downloads/KonaeAkira/raphael-rs/total?label=Downloads&color=%23dedede"/>](https://github.com/KonaeAkira/raphael-rs/releases)

:link: [www.raphael-xiv.com](https://www.raphael-xiv.com/)

Raphael is a crafting rotation solver for the online game Final Fantasy XIV.
It produces crafting macros that are tailored to your stats.

## Contents <!-- omit in toc -->

* [Optimal macro selection](#optimal-macro-selection)
* [How does it work?](#how-does-it-work)
* [Building from source](#building-from-source)
  * [Native GUI](#native-gui)
  * [Native CLI](#native-cli)
* [Contributing](#contributing)

## Optimal macro selection

The following is the specification of how the optimal macro is selected:

* The generated macro must be able to finish the synthesis, i.e. reach 100% progress.
* Valid macros are then ranked based on these criteria, in order:
  * Quality reached, capped at the target quality defined in the solver configuration. (Higher is better)
  * Number of macro steps. (Lower is better)
  * Total macro duration, in seconds. (Lower is better)

Anything not mentioned in the above specification is not guaranteed to be taken into account.
If you would like to change/amend the specification, please submit a feature request.

If you find a macro that beats the generated macro according to the specification above, please submit a bug report.

## How does it work?

* Short answer: [A* search](https://en.wikipedia.org/wiki/A*_search_algorithm) + [Pareto optimization](https://en.wikipedia.org/wiki/Multi-objective_optimization) + [Dynamic programming](https://en.wikipedia.org/wiki/Dynamic_programming).
* Long answer: coming soon<sup>tm</sup>

## Building from source

The [Rust](https://www.rust-lang.org/) toolchain is required to build the solver.
The minimal supported Rust version (MSRV) is 1.88.0.

> [!NOTE]  
> At the moment of writing, Rust version 1.88.0 is in beta and hasn't been released as stable yet.
> To use the Rust's beta version:
>
> ```
> rustup update beta && rustup default beta
> ```
>
> Rust 1.88.0 is expected to release on 26.06.2025, after which you don't need the above command anymore.

### Native GUI

To build and run the application:

```
cargo run --release
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

Some basic examples:
```
cargo run --release --package raphael-cli -- search --pattern "Fiberboard"
cargo run --release --package raphael-cli -- solve --recipe-id 36183 --stats 5400 4900 600
```

The CLI can also be installed so that it can be called from anywhere:

```
cargo install --path raphael-cli
```

## Contributing

First of all, thank you for your interest in contributing to the project!

If you are looking for things to help out on, the [Open Issues](https://github.com/KonaeAkira/raphael-rs/issues) are a good place to start.

If you already have something in mind, feel free to open a pull request.
Although ideally, you would discuss your idea on [Discord](https://discord.com/invite/m2aCy3y8he) beforehand to make sure it fits the general direction of the project and that no one else is already working on it.

Before submitting a pull request, make sure all tests are ok by running:
```
cargo test --workspace
```

> [!IMPORTANT]  
> Pull requests should be opened against the `preview` branch. The `main` branch is for releasing.
