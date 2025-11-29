# What is this?
A working example of building and running RustGPU projects on nix without Rustup or Cargo-GPU and without being forced onto nightly for spirv-builder.

Also a really good template repo if you want to start writing rust-gpu on nix with wgpu/winnit, its pretty much as minimal as it gets and you can just jump straight into writing shaders.

# How I Got Here?
## Rust-GPU
rust-gpu is an awesome crate that adds a custom compiler target for spirv. As a result, it needs to do some rather ugly things (as far as i can tell) which force us onto a very specific nightly version.

It is really *not* ideal for me to be writing my code in that nightly version, I'd much rather keep just the shaders compiled there and everything else on the latest stable.

A few repos have worked around this, that I explored, however none of them are nix-oriented. It seems they *all* rely on rustup or cargo-gpu. [krnl](https://github.com/charles-r-earp/krnl) is one such example of a repo pointed to in the discussion on the rust-gpu git.

## Cargo-GPU
cargo-GPU is supposed to be the solution to the rust versioning issues presented by rust-gpu.

cargo-gpu has a problem in that it requires rustup. Rustup is very un-nix-like in that it imperatively installs rust versions outside your nix store. I tried to find a declarative way to manage rustup, but I don't believe it exists.

If you want to use nix as intended, you probably don't want rustup, which breaks cargo-gpu!!

# The Solution
So, after several days of exhausting every git issue and existing repo i could find that uses rust-gpu, i had to come up with something new.

## How to use it?
If you enter the devshell provided, `cargo r --bin runner` should just... work. There are some assumptions that your running on wayland, etc, but those are unrelated to what I'm trying to solve here.

## How it works?
We can break this into a few things.
1. flake.nix
  - The flake.nix is actually using two seperate rust toolchains. The usual one for development (what the shell provides) and second, `spirvToolchain` which we generate the path for and store in `SPIRV_PATH`.
2. builder project
  - This is an *excluded* workspace, meaning it is a truly seperate cargo project.
  - If you look in the dir you can see it has a rust-toolchain.toml and expects that nightly version of rust we are trying to avoid.
  - This is a pretty simple application of `spirv-builder`, everything it does can be found in the docs for that crate. A notable difference from most other examples (including the ones in the rust-gpu crate) is that this is not a build.rs file. It is running, in main, and invoking spirv-builder. This shouldn't work on its own, as environment variables we would expect like `OUT_DIR` that are provided to build.rs wont be available (this isnt a build script!!).
3. build.rs
  - build.rs invokes `cargo run` on the `builder` project. Importantly, we override the usual path with `SPIRV_PATH` right before we do that, which lets builder run on the correct nightly version.
  - Because we are running in a build script, it seems like the environment variables carry through to our `builder`, so it is able to get the `OUT_DIR` of *this* project, rather than itself (which is great!).
  - `builder` then builds the shader in `crates/simplest_shader` and sets two rust environment variables.
    - `SIMPLEST_SHADER_ENTRYPOINTS` for the .rs entrypoints module file.
    - `SIMPLEST_SHADER_PATH` for the path to the generated spirv file.
  - Other shader crates can be added by adding additional calls to `build_shader` in `builder/src/main.rs`.

# What Should Probably Change?
This is a pretty clean and good solution. It would be really nice in future if cargo-gpu could ditch rustup and instead take two environment variables, `RUSTC` for the usual rustc install and `SPIRV_RUSTC` for the speicic nightly build needed to compile shaders. This would allow a lot more flexibility in how it is used.
