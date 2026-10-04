# raphael-dioxus

A rewrite of the Raphael GUI using [Dioxus](https://dioxuslabs.com) 0.7. It is meant to replace the
egui app in the root crate (`raphael-xiv`). Both apps live side by side until this one reaches parity.

The solver, simulator, game data and translations are shared with the egui app. This crate only
contains the UI, which is plain HTML/CSS rendered in the browser (web) or in a system webview (native).

## Running

### Native

```sh
cargo run --release --package raphael-dioxus
```

On Linux, Dioxus desktop needs WebKitGTK and libxdo:

- Arch: `pacman -S webkit2gtk-4.1 xdotool`
- Debian/Ubuntu: `apt install libwebkit2gtk-4.1-dev libgtk-3-dev libxdo-dev`

Windows uses WebView2 and macOS uses WKWebView, so they need no extra dependencies.

### Web

The web version is built with [Trunk](https://trunkrs.dev), like the egui app. It reuses
`.cargo/config_wasm.toml` (atomics, shared memory and `build-std`) so that the solver can run on a
`wasm-bindgen-rayon` thread pool.

```sh
./raphael-dioxus/serve-web.sh   # dev server on http://localhost:8080
./raphael-dioxus/build-web.sh   # production build into raphael-dioxus/distrib
```

Multithreading needs `SharedArrayBuffer`, so the page must be served with these headers:

```
Cross-Origin-Embedder-Policy: require-corp
Cross-Origin-Opener-Policy: same-origin
```

`Trunk.toml` sets them for `trunk serve`, and `assets/_headers` sets them for Cloudflare Pages.

The `dx` CLI is not used. Trunk already supports the custom wasm build configuration needed for threads.

## Persistence

Settings, crafter stats and saved rotations are stored as RON under the same keys the egui app uses:

- **Web:** in `localStorage`. On the same origin, existing data carries over automatically.
- **Native:** in `settings.ron` in the egui app's data directory (e.g. `~/.local/share/raphaelxiv`).
  On first launch the data is imported from the egui app's `app.ron`.

## Layout

| Path                   | Contents                                                          |
| ---------------------- | ----------------------------------------------------------------- |
| `src/config.rs`        | Recipe, crafter and quality-target configuration (from egui app)  |
| `src/context.rs`       | `AppContext`: all persisted app state                             |
| `src/solve.rs`         | Starts the solver on the thread pool and polls its progress       |
| `src/macros.rs`        | Rotation → in-game macro text                                     |
| `src/saved_rotations.rs` | Solve history & saved macros                                    |
| `src/storage.rs`       | `localStorage` / file persistence                                 |
| `src/thread_pool.rs`   | Lazy rayon pool initialization (native & `wasm-bindgen-rayon`)    |
| `src/native.rs`        | Desktop-only: bundled asset serving and self-update               |
| `src/ui/`              | Dioxus components                                                 |
| `src/style.css`        | All styles (light & dark themes)                                  |
