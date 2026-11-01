# 模块职责

- `app`：应用初始化、executor 和事件循环的运行；`context.rs` 定义共享服务，`shutdown.rs` 实现退出信号。`Application` 与 `Context` 从 crate 根导出。
- `event`：与平台无关的事件机制。`EventHandle` 提交事件和监听器变更，`EventReceiver` 接收队列里的 `QueuedEvent`，`EventDispatcher` 按注册顺序执行监听器。`error.rs` 集中定义提交错误，`registry.rs` 管理监听器，`observer.rs` 适配同步和异步回调，`on.rs` 提供类型化事件参数，`payload.rs` 保存类型擦除后的事件。
- `tao_event`：平台事件适配。`source.rs` 的 `TaaoEventSource` 接收 `&Context`，在独立 executor 任务中轮询 Taao，并将原始事件和类型化事件提交到通用队列。`routing.rs` 负责转换，`window.rs`、`device.rs`、`lifecycle.rs` 定义通知类型。
- `window`：`Window` 只持有原生窗口，`builder.rs` 只负责原生窗口初始化。`render/mod.rs` 的 `WindowRender` 管理 surface、adapter、device、queue、surface 配置，以及获取帧、清屏、提交和呈现。`render/options.rs` 定义 GPU、surface 和绘制选项，`render/content.rs` 定义窗口绘制工厂与空内容，`render/error.rs` 定义初始化错误，`render/observer.rs` 负责重绘注册及关闭时释放监听器。
- `macro`：`phanerite-ui-macro` 过程宏 crate，提供 `#[derive(Event)]`。

共享绘制接口 `RenderContent` 和 `RenderViewport` 定义在独立的 `phanerite-render` crate，
从 `window` 重新导出。布局快照转 GPU 实例、shader 和 `LayoutDebugRenderer` 也位于
该 crate，通过默认启用的 `layout-debug` feature 提供；executor 的库依赖关闭这一
feature，因此不依赖布局树或 Taffy。布局示例单独启用它来组合窗口、布局和绘制。
`phanerite-render` 不依赖 executor 或原生窗口库，可直接对离屏纹理绘制。

事件接收端和平台事件源使用不同名称，避免混淆数据流方向：

```text
Taao → TaaoEventSource → EventHandle → EventReceiver → EventDispatcher → On<E>
业务代码 ────────────────────↑
```

`emit` 等待入队，`try_emit` 在队列满时返回错误，`trigger` 直接执行监听器。平台源与 dispatcher 并发运行；监听器仍按注册顺序串行执行。

自定义绘制通过 `WindowRenderOptions::default().with_content(|device, queue, config| ...)` 提供工厂。在 GPU 初始化完成、surface 格式确定后调用一次，返回实现 `RenderContent` 的对象，持续持有 pipeline、buffer、纹理等资源。每帧清屏后调用对象的 `render`，向同一个 encoder 添加命令；自定义 pass 使用 `LoadOp::Load` 保留背景。surface 重新配置时调用 `configure`，可更新依赖尺寸或格式的资源。

`WindowRender::new_with_options(&window, &cx, options).await` 创建渲染器，`register(&cx)` 注册该窗口的重绘和关闭监听器。首次绘制由调用方 `window.window.request_redraw()` 发起。完整示例见 `examples/custom_render.rs`，默认选项仅绘制白色背景。
