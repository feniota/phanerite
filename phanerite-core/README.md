# Phanerite Core

[![Crates.io](https://img.shields.io/crates/v/phanerite-core.svg)](https://crates.io/crates/your-crate)
[![Documentation](https://docs.rs/phanerite-core/badge.svg)](https://docs.rs/phanerite-core)
[![License](https://img.shields.io/crates/l/phanerite-core.svg)](../LICENSE)

A Minecraft launcher core library written in Rust.

[简体中文](README.zh-CN.md)

`phanerite-core` is the backend core of [Phanerite](https://github.com/feniota/phanerite), providing the
infrastructure a Minecraft launcher needs: resolving version manifests, downloading and verifying files, installing Java
runtimes and mod loaders, handling login, producing the launch command line, and reading and writing the files and logs
the game produces.

It has no opinion about your UI framework, your async runtime, or how you present progress.

## Design

**Explicit orchestration, free composition.** Core provides composable components rather than one fixed pipeline.
Downloaders, caches, mirrors, task groups and progress monitors can all be combined however the caller sees fit; Core
neither requires a particular async runtime nor hides its own execution strategy.

**No global state.** Core uses no singletons. `Storage`, `Downloader`, `JavaManager` and the like are all created by the
caller and passed in explicitly. This makes wiring somewhat more verbose, but it lets several independent configurations
coexist within one process, and removes any need to swap out globals in tests. The objects that genuinely do need to be
shared globally, such as login credentials and storage directories, go into `utils::container::Container`, which
provides concurrent access and snapshots.

**Installing produces tasks, it does not download.** `Instance::install` returns an iterator of `DownloadTask` instead
of
performing the downloads itself. The caller picks a suitable downloader to run them and decides the concurrency, whether
to use a mirror or a cache, and how to monitor progress. Different execution strategies are expressed by composing
downloaders, rather than by passing a large number of configuration options.

**Type-driven guidance through the workflow.** The build state of an object is recorded in its type parameters, and each
state exposes only the operations that are legal at that point. Your IDE's LSP completion therefore becomes part of the
workflow itself: from the value in front of you, you can see what can be done right now and what the next step may be,
without having to memorise the whole API. `launch()`, for instance, only appears in the completion list once both the
Java runtime and the files are ready. Illegal states not only fail to compile, they never show up on the normal path
through the API.

**No async runtime baked in.** Core depends only on `futures` and the `async-*` family of crates; the examples use
`smol`. Compute-bound work that may occupy the CPU for a long time, such as extraction, runs on a dedicated thread so
that it does not block the async executor.

## Modules

| Module        | Answers                                         |
|---------------|-------------------------------------------------|
| `storage`     | Where the data is stored                        |
| `download`    | How the data is downloaded                      |
| `instance`    | What a launchable game version is               |
| `auth`        | Which identity the game is launched with        |
| `runtime`     | Which Java runtime launches it                  |
| `mod_loader`  | How a mod loader is added to an instance        |
| `mod_project` | Where to look for mods                          |
| `parsers`     | How to read and write the game's files and logs |

Every module carries a `//!` doc comment covering its design intent and the things to watch out for. Start there rather
than with the module listing.

## Getting started

```rust
use phanerite_core::download::DownloaderExt;
use phanerite_core::download::downloader::RawDownloader;
use phanerite_core::download::vanilla::VersionIndex;
use phanerite_core::error::Error;
use phanerite_core::instance::Instance;
use phanerite_core::storage::Storage;

fn main() -> Result<(), Error> {
    smol::block_on(async {
        // Where to store the data
        let storage = Storage::new(".minecraft").await?;

        // The base downloader. Usually created once and reused
        let raw = RawDownloader::builder().build().await?;

        // A task group for tracking the execution of this batch of tasks
        let downloader = raw.with_group();

        // Get the version to install
        let manifest = VersionIndex::sync(&downloader)
            .await?
            .latest_release()?
            .get_manifest(&downloader)
            .await?;

        let instance = Instance::create(manifest, Some("latest"), &storage, &downloader).await?;

        // install only generates tasks; the downloader is responsible for executing them
        downloader
            .join(instance.install(std::collections::HashSet::new()).await?)
            .await
            .iter()
            .for_each(|e| eprintln!("{e}"));

        Ok::<(), Error>(())
    })
}
```

`examples/fullflow.rs` demonstrates the complete workflow: logging in through Yggdrasil with Authlib-Injector,
installing a Java runtime, installing the instance, launching the game, and monitoring progress throughout.

Run it with:

```sh
cargo run --example fullflow
```

The example reads `USERNAME` and `PASSWORD` from the environment; a `.env` file can supply them as well.

## Sharing files between instances

Files under `{root}/share` are named after the Blake3 hash of their contents. When several instances reference the same
library or asset, only a single copy is kept on disk.

`Storage::new` probes the whole directory tree and decides the sharing strategy from what the filesystem actually
supports: hard links first, then symlinks, falling back to plain moves. Because the tree may span several filesystems,
that strategy is based on what the probe measures rather than on assumptions about how the filesystem behaves.

Shared files are not reclaimed automatically. After deleting an instance, call `Storage::clean_hardlink` to clear out
files whose reference count has dropped to zero; the operation itself is very fast.

## Requirements

A **nightly** toolchain is required (see `rust-toolchain.toml`). The unstable features in use are listed at the top of
`src/lib.rs`.

## Current status

The project is at an early stage and the API is not yet stable. The following are still under early development:

* Mod loaders: Fabric and NeoForge are usable; Forge is not yet supported.
* Mod repositories: early development.
