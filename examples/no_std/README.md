# `no_std` example

A minimal `#![no_std]` library that drives a `Reactor` with `bevy_flurx` and Bevy built without `std`.
See [src/lib.rs](src/lib.rs).

## Run it

The test builds an `App` and runs the reactor to completion on the host.
`bevy_flurx` and `bevy` are still compiled without their `std` feature, so this exercises the `no_std` code path.

```sh
cargo test
```

## Check it compiles without `std`

Compiling for a target that has no standard library ensures that no dependency links `std`:

```sh
rustup target add x86_64-unknown-none
cargo check --target x86_64-unknown-none
```

Targets without native atomics also need the `critical-section` feature:

```sh
rustup target add thumbv6m-none-eabi
cargo check --features critical-section --target thumbv6m-none-eabi
```
