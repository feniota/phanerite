# Phanerite Core

一个使用 Rust 编写的 Minecraft 启动器核心库。

[English](README.md)

`phanerite-core` 是 [Phanerite](https://github.com/feniota/phanerite) 的后端核心，提供 Minecraft
启动器所需的基础设施：解析版本清单、下载与校验文件、安装 Java 运行时和模组加载器、处理登录、生成启动命令行，以及读写游戏产生的文件和日志。

它不关心界面框架、异步运行时或进度展示方式。

## 设计

**显式编排，自由装配。** Core 提供可组合的组件，而不是一套固定的执行管线。下载器、缓存、镜像、任务组和进度监视器均可由调用方自由组合；Core
不要求特定的异步运行时，也不会隐藏自身的执行策略。

**没有全局状态。** Core 不使用单例。`Storage`、`Downloader`、`JavaManager`
等对象均由调用方创建并显式传入。这样虽然使装配过程略显繁琐，但允许同一进程中并存多套互不干扰的配置，也无需在测试中替换全局变量。确实需要全局共享的对象，例如登录凭据和存储目录，则放入
`utils::container::Container`，由其提供并发安全的访问和快照。

**安装是「生成任务」，而不是「执行下载」。** `Instance::install` 返回 `DownloadTask`
的迭代器，而不直接执行下载。任务由调用方选择合适的下载器执行，并自行决定并发度、是否使用镜像或缓存，以及如何监视进度。不同的执行策略通过组合下载器表达，而不是通过传递大量配置选项实现。

**类型驱动的流程引导。** 对象的构建状态通过类型参数记录，不同状态仅暴露当前合法的操作。因此，IDE 的 LSP
补全本身也是流程的一部分：开发者可以直接从当前值看到当前能够执行的操作以及下一步可以进行的操作，而无需记忆完整的
API。比如，只有在 Java 和文件均准备完成后，`launch()` 才会出现在补全列表中。非法状态不仅无法通过编译，也不会出现在正常的 API
使用路径中。

**不绑定异步运行时。** Core 仅依赖 `futures` 与 `async-*` 系列库，示例使用 `smol`。对于解压等可能长时间占用 CPU
的计算密集型操作，则使用独立线程执行，以避免阻塞异步执行器。

## 模块

| 模块            | 回答的问题           |
|---------------|-----------------|
| `storage`     | 数据存储在哪里         |
| `download`    | 数据如何下载到本地       |
| `instance`    | 一个可启动的游戏版本是什么   |
| `auth`        | 以什么身份启动游戏       |
| `runtime`     | 使用哪个 Java 运行时启动 |
| `mod_loader`  | 如何为实例添加模组加载器    |
| `mod_project` | 从哪里查找模组         |
| `parsers`     | 如何读写游戏侧的文件和日志   |

每个模块都包含介绍设计意图和注意事项的 `//!` 文档注释，建议从这些文档开始阅读，而不是直接从模块列表入手。

## 快速开始

```rust
use phanerite_core::download::downloader::RawDownloader;
use phanerite_core::download::vanilla::VersionIndex;
use phanerite_core::download::DownloaderExt;
use phanerite_core::error::Error;
use phanerite_core::instance::Instance;
use phanerite_core::storage::Storage;
use std::collections::HashSet;

fn main() -> Result<(), Error> {
    smol::block_on(async {
        // 数据存储位置
        let storage = Storage::new(".minecraft").await?;

        // 基础下载器。通常创建一次后持续复用
        let raw = RawDownloader::builder().build().await?;

        // 任务组只负责跟踪这一批任务的执行状态
        let downloader = raw.with_group();

        // 获取要安装的版本
        let manifest = VersionIndex::sync(&downloader)
            .await?
            .latest_release()?
            .get_manifest(&downloader)
            .await?;

        let instance = Instance::create(manifest, Some("latest"), &storage, &downloader).await?;

        // install 只生成任务，实际执行由下载器负责
        downloader
            .join(instance.install(HashSet::new()).await?)
            .await
            .iter()
            .for_each(|e| eprintln!("{e}"));

        Ok::<(), Error>(())
    })
}
```

`examples/fullflow.rs` 展示了完整的工作流程：使用 Authlib-Injector 通过 Yggdrasil 登录、安装 Java 运行时、安装实例、启动游戏，并监视执行进度。

运行：

```sh
cargo run --example fullflow
```

示例从环境变量读取 `USERNAME` 和 `PASSWORD`，也可以使用 `.env` 文件提供这些变量。

## 实例之间共享文件

`{root}/share` 下的文件以内容的 Blake3 值命名。多个实例引用相同的库或资源时，只需在磁盘上保存一份。

`Storage::new` 会探测整棵目录树，根据实际文件系统能力决定文件共享策略：优先使用硬链接，其次使用符号链接，最后退化为直接移动。由于目录树可能跨越多个文件系统，因此该策略基于实际探测结果，而不是预先假定文件系统行为。

共享文件不会自动回收。删除实例后，需要调用 `Storage::clean_hardlink` 清理引用计数已经归零的文件；该操作本身非常快速。

## 环境要求

需要 **nightly** 工具链（见 `rust-toolchain.toml`）。使用到的不稳定特性列于 `src/lib.rs` 顶部。

## 当前状态

项目目前处于早期阶段，API 尚未稳定。以下模块仍在早期开发中：

* 模组加载器：Fabric 和 NeoForge 可用，Forge 尚未支持。
* 模组仓库：早期开发阶段。
