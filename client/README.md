# ME2 Game Client

## Building

This project uses Rust's build system Cargo. Install Rust [here](https://rustup.rs/).

You will need a 32 bit Windows target installed. On Windows, you probably want to run `rustup target add i686-pc-windows-msvc` On Linux, you probably want to run `rustup target add i686-pc-windows-gnu`.

```
cd client
```

Then:

```
cargo build --release --target i686-pc-windows-msvc
```

or

```
cargo build --release --target i686-pc-windows-gnu
```

The output be in `target/me2_game/` and there will be a zip file `target/me2_game.zip`.
