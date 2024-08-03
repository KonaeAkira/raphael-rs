# Raphael XIV [<img src="https://img.shields.io/discord/1244140502643904522?logo=discord&logoColor=white"/>](https://discord.com/invite/m2aCy3y8he)

:link: [www.raphael-xiv.com](https://www.raphael-xiv.com/)

Raphael is a crafting rotation solver for the online game Final Fantasy XIV.
* Produces optimal solutions. Achieving higher quality than the solver is impossible.
* Short solve time (5-20 seconds) and reasonable memory usage (300-500 MB).

## How does it work?

* Short answer: [A* search](https://en.wikipedia.org/wiki/A*_search_algorithm) + [Pareto optimization](https://en.wikipedia.org/wiki/Multi-objective_optimization) + [Dynamic programming](https://en.wikipedia.org/wiki/Dynamic_programming).
* Long answer: coming soon<sup>tm</sup>

## Building from source (wasm)

The [Rust](https://www.rust-lang.org/) toolchain is required to build the solver.
[Trunk](https://trunkrs.dev/) is required to bundle and deploy the WASM and can be installed via the Rust toolchain:

```
cargo install --locked trunk
```

To build and host the application locally):

```
export RANDOM_SUFFIX=""
export RUSTFLAGS="--cfg=web_sys_unstable_apis"
trunk serve --release --dist docs
```


## Running/building from source (native)

```
cargo run -r --bin raphael-xiv
```