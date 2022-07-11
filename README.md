# Chip-8 emulator in Rust

This is a small Chip-8 emulator written in Rust that passes all the tests
I know of at the very least.

The Chip-8 console was a fantasy console / interpreted language / virtual machine that never really had an actual device to go with it, though it makes for a great introduction into emulating regardless.

It is my first foray into emulation development and it was quite fun. The timings between instructions can vary between systems as they aren't really specified and I use `spin-sleep` for more accurate waiting which is not great for performance but made it easier to implement.

## Usage

You will need to provide your own ROMs, though they are quite easy to find.

The path to the ROM is passed as the first (and only) parameter either with the executable directly like so:

```bash
$ ./chip-8rs <ROM_PATH>
```

or via `cargo`:

```bash
$ cargo run --release -- <ROM_PATH>
```

This emulator has to be built in release mode as there is an integer overflow that happens in debug mode which would then panic. (Of course that is an intended feature and not a bug!)