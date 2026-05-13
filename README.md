# typr

CLI typing test in Rust. monkeytype-style: text and code, time / word-count / snippet limits, live WPM, syntax-highlighted code mode.

## Run

```
typr                       # text, 30s default
typr -s code -l rust       # rust code, snippet or time
typr -s text -d e1k        # 1k word difficulty
typr --help
```

Keys inside the TUI: Tab cycles limit, Shift+Tab back, Up/Down switches source, Left/Right cycles difficulty (text) or language (code). Tab while typing code auto-advances through indentation. Enter on results retries.

## Build

Requires Rust (`rustup`).

### macOS arm64 (Apple Silicon, native)

```
cargo build --release
./target/release/typr
```

### Linux x86_64 (cross from macOS)

```
rustup target add x86_64-unknown-linux-musl
CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=rust-lld \
  cargo build --release --target x86_64-unknown-linux-musl
```

Output: `target/x86_64-unknown-linux-musl/release/typr` — static, runs on any Linux x86_64.

### Linux aarch64 (cross from macOS)

```
rustup target add aarch64-unknown-linux-musl
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=rust-lld \
  cargo build --release --target aarch64-unknown-linux-musl
```

Output: `target/aarch64-unknown-linux-musl/release/typr` — static, runs on any Linux arm64 (Raspberry Pi, Graviton, etc).

### Linux native (any arch)

```
cargo build --release
```

## Release via GitHub Actions

Push a tag matching `v*` and `.github/workflows/release.yml` builds all three targets and attaches binaries to a GitHub Release.

```
git tag v0.1.0
git push --tags
```

Trigger manually from the Actions tab via `workflow_dispatch` to just produce build artifacts without a release.

## Deploy

```
scp target/<target>/release/typr user@host:~/
ssh user@host 'chmod +x ~/typr && ~/typr'
```

Binaries are stripped and LTO'd via the `[profile.release]` config — ~3-4MB.
