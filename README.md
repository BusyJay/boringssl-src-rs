# boringssl-src
A crate for building boringssl.

This crate is intended to integrate with other build script to build boringssl.

1. To use the crate, just include it as build-dependency:
```toml
[build-dependencies]
boringssl-src = "0.1"
```

2. And then build it in build script:
```rust
let artifact = boringssl_src::Build::new().build();
```

3. If you just need to link it to your library, then let it setup directly:
```rust
artifacts.print_cargo_metadata();
```

If you want to make it available to existing build system, take CMake as an example,
you can setup by using `OPENSSL_ROOT_PATH`:
```rust
let config = cmake::Config::new("native project");
config.define("OPENSSL_ROOT_DIR", format!("{}", boringssl_artifact.root_dir().display()));
```

Then cmake should be able to find the library by `find_package(OpenSSL)`.

# How and When is boringssl updated?

It's updated periodically. It for now serves as a build dependency for tikv/grpc-rs, so
whenever grpc updates boringssl, this crate also updates the native dependency.
