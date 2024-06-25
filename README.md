# Raphael XIV

[<img src="https://img.shields.io/discord/1244140502643904522?logo=discord&logoColor=white"/>](https://discord.gg/Qd9u9CtaYj)

Raphael is a crafting rotation solver for the online game Final Fantasy XIV.

**Key Features:**
* Produces optimal solutions. Achieving higher quality than the solver is impossible.
* Fast solve-time (5-10 seconds) and reasonable memory usage (300 - 500 MB).

**How does it work?**

* Short answer: A* search, pareto optimization, dynamic programming.
* Long answer: coming soon :tm:

## Building from source

The [Rust](https://www.rust-lang.org/) toolchain is required to build the solver.
[Trunk](https://trunkrs.dev/) is required to bundle and deploy the WASM and can be installed via the Rust toolchain:

```
cargo install --locked trunk
```

To build and host the application locally (optionally use the `--release` flag):

```
RUSTFLAGS='--cfg=web_sys_unstable_apis' trunk serve
```
