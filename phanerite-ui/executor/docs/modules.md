# 模块职责

- `app`：应用初始化、executor 和事件循环的运行；`context.rs` 定义共享服务，`shutdown.rs` 实现退出信号。`Application` 与 `Context` 从 crate 根导出。
- `event`：与平台无关的事件机制。`EventHandle` 提交事件和监听器变更，`EventReceiver` 接收队列里的 `QueuedEvent`，`EventDispatcher` 按注册顺序执行监听器。`error.rs` 集中定义提交错误，`registry.rs` 管理监听器，`observer.rs` 适配同步和异步回调，`on.rs` 提供类型化事件参数，`payload.rs` 保存类型擦除后的事件。
- `tao_event`：平台事件适配。`source.rs` 的 `TaaoEventSource` 接收 `&Context`，在独立 executor 任务中轮询 Taao，并将原始事件和类型化事件提交到通用队列。`routing.rs` 负责转换，`window.rs`、`device.rs`、`lifecycle.rs` 定义通知类型。
- `window`：窗口与其独立拥有的 GPU 资源。`builder.rs` 负责初始化，`surface.rs` 定义与窗口尺寸无关的 `SurfaceOptions`，`render.rs` 保留现有渲染扩展位置。
- `macro`：`phanerite-ui-macro` 过程宏 crate，提供 `#[derive(Event)]`。

事件接收端和平台事件源使用不同名称，避免混淆数据流方向：

```text
Taao → TaaoEventSource → EventHandle → EventReceiver → EventDispatcher → On<E>
业务代码 ────────────────────↑
```

`emit` 等待入队，`try_emit` 在队列满时返回错误，`trigger` 直接执行监听器。平台源与 dispatcher 并发运行；监听器仍按注册顺序串行执行。
