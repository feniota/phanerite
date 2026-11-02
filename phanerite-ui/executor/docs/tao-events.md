# Tao 事件使用说明

本文对应当前 `phanerite-executor` 的 `tao_event` 模块，基于 Tao 0.37 和 Taao 0.1.3。

`tao_event` 将平台输入转换成可通过 `On<E>` 监听的类型。所有类型位于 `phanerite_executor::tao_event`，不需要逐个调用
`register_event`，注册监听器时会自动注册事件类型。

## 快速开始

通过 `Application::event_handle()` 获取统一的 `EventHandle`，使用它注册监听器：

```rust
use phanerite_executor::{
    Application,
    event::On,
    tao_event::window::{MouseInput, Resized},
};
use tao::event::{ElementState, MouseButton};

fn main() {
    let app = Application::new();
    let events = app.event_handle();

    events.observe(|event: On<MouseInput>| {
        if event.button == MouseButton::Left
            && event.state == ElementState::Pressed
        {
            println!("窗口 {:?} 按下鼠标左键", event.window_id);
        }
    }).unwrap();

    events.observe(async |event: On<Resized>| {
        println!("窗口 {:?} 尺寸变为 {:?}", event.window_id, event.size);
        // 可以在这里 await 异步操作。
    }).unwrap();

    app.run().unwrap();
}
```

`observe` 返回 `Result<ObserverId, ObserveError>`。句柄可以克隆，在其他任务或线程中继续注册和移除监听器。框架内部的
`Context.events` 也是同一种句柄。

当前监听范围是整个 dispatcher。窗口事件都带有 `window_id`；只处理某个窗口时，需要在回调中比较该字段。目前没有按窗口或 UI
节点注册的接口，也没有捕获、冒泡和停止传播机制。

## 输入与分发过程

```text
Tao 平台事件
    ↓ Taao 转为异步事件流
tao_event::TaaoEventSource::run
    ↓ 依次 emit 原始 EventKind 和对应类型化事件
EventHandle → EventReceiver → EventDispatcher::pump → On<E> 监听器
```

`event` 模块只处理事件队列和监听器，不依赖 Taao/Tao。`Application` 并发驱动 `TaaoEventSource::run()` 和 dispatcher 的
`pump()`；前者等待平台输入，通过 `emit` 转发，队列满时等待容量。

对于同一条平台输入，原始 `EventKind` 先入队，对应的类型化事件随后入队。它们是两条独立事件，其他生产者的事件可以插入其间，监听器增删也可以在两次分发之间生效。没有对应类型的事件仍可通过原始
`EventKind` 监听。

同一事件的监听器按实际注册顺序依次执行，每个异步回调完成后才执行下一个。回调长时间等待时，后续事件和监听器变更也会等待。

平台输入只有 Taao 事件流这一个来源。自定义事件通过同一个 `EventHandle` 的 `emit` 或 `try_emit` 提交；自定义事件直接按自身类型分发，不经过
Tao 路由。

## 窗口事件：`tao_event::window`

本节所有类型都包含 `window_id: tao::window::WindowId`。表中只列出其余字段；`—` 表示没有其他字段。类型名通常对应 Tao
`WindowEvent` 的同名变体。

### 窗口状态与文件

| 类型                     | 其他字段                                                          | 含义                                      |
|------------------------|---------------------------------------------------------------|-----------------------------------------|
| `Resized`              | `size: PhysicalSize<u32>`                                     | 窗口客户区尺寸变化，单位为物理像素                       |
| `Moved`                | `position: PhysicalPosition<i32>`                             | 窗口位置变化，保留 Tao 的物理坐标                     |
| `CloseRequested`       | —                                                             | 收到关闭请求；路由本身不关闭窗口或退出应用                   |
| `Destroyed`            | —                                                             | 窗口已销毁                                   |
| `Started`              | —                                                             | 窗口生命周期进入 started 状态，Tao 主要在 Android 上提供 |
| `Suspended`            | —                                                             | 窗口生命周期暂停，Tao 在 Android/iOS 上提供          |
| `Resumed`              | —                                                             | 窗口生命周期恢复，Tao 在 Android/iOS 上提供          |
| `Stopped`              | —                                                             | 窗口生命周期停止，Tao 主要在 Android 上提供            |
| `Focused`              | `focused: bool`                                               | 获得或失去焦点                                 |
| `DroppedFile`          | `path: PathBuf`                                               | 文件被拖放到窗口；多个文件可产生多条事件                    |
| `HoveredFile`          | `path: PathBuf`                                               | 文件拖动到窗口上方                               |
| `HoveredFileCancelled` | —                                                             | 文件拖动离开或取消                               |
| `ScaleFactorChanged`   | `scale_factor: f64`、`suggested_inner_size: PhysicalSize<u32>` | DPI 缩放比例及建议客户区尺寸变化                      |
| `ThemeChanged`         | `theme: tao::window::Theme`                                   | 系统主题变化                                  |
| `DecorationsClick`     | —                                                             | 窗口装饰区域被点击，Tao 当前仅在 Windows 上提供          |

`PhysicalSize` 和 `PhysicalPosition` 来自 `tao::dpi`，`PathBuf` 来自 `std::path`。事件保留物理单位；需要逻辑单位时，使用对应窗口的缩放比例转换。

`ScaleFactorChanged` 是拥有数据所有权的通知。`suggested_inner_size` 是值拷贝，不能像同步 Tao 回调中的 `&mut PhysicalSize`
那样修改平台回调的尺寸。Taao 将此输入包装为 `EventKind::TaoScaleFactorChanged`，本模块再转换为上述类型。

### 键盘、鼠标与触摸

| 类型                 | 其他字段                                                                                            | 含义                  |
|--------------------|-------------------------------------------------------------------------------------------------|---------------------|
| `ReceivedImeText`  | `text: String`                                                                                  | 收到文本输入              |
| `KeyboardInput`    | `device_id: DeviceId`、`event: KeyEvent`、`is_synthetic: bool`                                    | 窗口键盘输入，包括是否为合成事件    |
| `ModifiersChanged` | `modifiers: ModifiersState`                                                                     | Shift、Ctrl 等修饰键状态变化 |
| `CursorMoved`      | `device_id: DeviceId`、`position: PhysicalPosition<f64>`、`modifiers: ModifiersState`             | 光标在客户区中的物理位置变化      |
| `CursorEntered`    | `device_id: DeviceId`                                                                           | 光标进入窗口              |
| `CursorLeft`       | `device_id: DeviceId`                                                                           | 光标离开窗口              |
| `MouseWheel`       | `device_id: DeviceId`、`delta: MouseScrollDelta`、`phase: TouchPhase`、`modifiers: ModifiersState` | 鼠标滚轮或触控板滚动          |
| `MouseInput`       | `device_id: DeviceId`、`state: ElementState`、`button: MouseButton`、`modifiers: ModifiersState`   | 鼠标按钮按下或释放           |
| `TouchpadPressure` | `device_id: DeviceId`、`pressure: f32`、`stage: i64`                                              | 触控板压力变化             |
| `AxisMotion`       | `device_id: DeviceId`、`axis: AxisId`、`VALUE: f64`                                               | 窗口收到设备轴输入           |
| `Touch`            | `touch: tao::event::Touch`                                                                      | 触摸点状态、位置等数据         |

`ModifiersState` 来自 `tao::keyboard`，其余表内的非基础输入类型来自 `tao::event`。

`MouseScrollDelta` 可能是按行滚动的 `LineDelta`，也可能是按物理像素滚动的 `PixelDelta`，处理时需要区分。部分鼠标事件中的
`modifiers` 字段继承 Tao 的旧接口；跟踪修饰键状态可监听 `ModifiersChanged`。

`MouseInput` 只包含按钮状态，不包含光标位置。需要点击位置时，可结合相同窗口的 `CursorMoved` 保存的位置。这里尚未生成
`Click`、`PointerDown`、拖拽或双击等 UI 语义事件；这些需要上层结合命中测试和交互状态生成。

## 原始设备事件：`tao_event::device`

本节所有类型都包含 `device_id: tao::event::DeviceId`，不包含 `window_id`。

| 类型            | 其他字段                                     | 含义                   |
|---------------|------------------------------------------|----------------------|
| `Added`       | —                                        | 输入设备加入               |
| `Removed`     | —                                        | 输入设备移除               |
| `MouseMotion` | `delta: (f64, f64)`                      | 原始鼠标相对移动量，不是窗口内的光标位置 |
| `MouseWheel`  | `delta: MouseScrollDelta`                | 原始滚轮输入               |
| `Motion`      | `axis: AxisId`、`VALUE: f64`              | 原始设备轴输入              |
| `Button`      | `button: ButtonId`、`state: ElementState` | 原始设备按钮输入             |
| `Key`         | `event: RawKeyEvent`                     | 原始键盘输入               |
| `Text`        | `codepoint: char`                        | 原始文本字符               |

表内非基础类型来自 `tao::event`。窗口输入和设备输入是不同的事件类型，例如 `window::MouseWheel` 与 `device::MouseWheel`
需要分别监听。避免使用通配导入导致名称冲突：

```rust
use phanerite_executor::tao_event::{device, window};

// events 是 EventHandle。
events.observe( | event: On<window::MouseWheel>| {
println ! ("窗口滚动: {:?}", event.delta);
}) ?;
events.observe( | event: On<device::MouseMotion>| {
println ! ("设备移动: {:?}", event.delta);
}) ?;
```

Tao 将设备事件定义为独立于窗口焦点的原始输入，实际投递能力取决于平台。同一物理操作可能同时产生窗口事件和设备事件，而且两者的
`device_id` 不保证相同；不要将它们直接当作两次 UI 操作。

## 事件循环事件：`tao_event::lifecycle`

这些类型不共享统一字段，完整字段如下。

| 类型                    | 字段                                 | 当前 Taao 来源                 |
|-----------------------|------------------------------------|----------------------------|
| `NewEvents`           | `cause: tao::event::StartCause`    | Taao 0.1.3 过滤，不会自然到达       |
| `UserEvent`           | 无                                  | Taao 内部唤醒事件被过滤，不会自然到达      |
| `MainEventsCleared`   | 无                                  | Taao 0.1.3 过滤，不会自然到达       |
| `RedrawRequested`     | `window_id: tao::window::WindowId` | 窗口重绘请求                     |
| `RedrawEventsCleared` | 无                                  | Taao 0.1.3 过滤，不会自然到达       |
| `LoopDestroyed`       | 无                                  | Taao 0.1.3 过滤，不会自然到达       |
| `Opened`              | `urls: Vec<url::Url>`              | 外部资源打开应用，取决于平台             |
| `Reopen`              | `has_visible_windows: bool`        | 应用被重新打开，Tao 当前仅在 macOS 上提供 |

被过滤的类型仍然可以手动 `emit` 或 `trigger`，但定义了类型不代表平台源会投递它。尤其不能依赖这里的 `LoopDestroyed` 执行退出清理。

## 原始事件监听

需要查看 Taao 输入，或处理尚未类型化的平台变体时，监听 `EventKind`：

```rust
use phanerite_executor::event::On;
use taao::event::EventKind;

events.observe( | event: On<EventKind>| {
match & * event {
EventKind::Tao(tao_event) => println ! ("{tao_event:?}"),
EventKind::TaoScaleFactorChanged { scale_factor, .. } => {
println ! ("DPI 比例: {scale_factor}");
}
}
}) ?;
```

同时监听原始类型和对应的类型化事件，会收到同一条输入的两种表示。平台特有或未来新增的变体，只有在 Taao 实际保留并转发时，才能从这里收到。

## 运行期间增删监听器

```rust
let id = events.observe( | event: On<window::Focused>| {
println ! ("焦点状态: {}", event.focused);
}) ?;

// 可以在另一个任务中使用 events.clone() 后提交移除。
events.remove_observer(id) ?;
```

注册立即返回预留的 ID，实际增删由 `pump` 消费命令后应用。移除成功表示命令已入队；已经移除的 ID 再次移除是空操作。ID 属于对应
dispatcher，不能跨 dispatcher 使用。

回调内可用 `event.observer()` 获取自己的 ID，再通过捕获的句柄移除自己。当前输入剩余的监听器仍会正常执行，修改在后续分发之间生效。

命令按队列顺序处理，事件输入与命令之间没有统一的全局顺序。`pump` 每轮最多处理 64
条命令再检查事件；大量命令积压时，不能假定所有已提交变更都先于下一条事件生效。空闲时，新命令会唤醒 `pump`，没有忙等。

通过句柄注册的回调捕获需要满足 `Send + 'static`，异步回调返回的 Future 可以是 `!Send`。`On<E>` 通过 `Deref` 只读访问数据，也可以
`clone()` 或 `into_inner()` 保留共享数据，不要求 `E: Clone`。

## 自定义事件与使用限制

自定义事件使用相同句柄：

```rust
use phanerite_executor::event::Event;

#[derive(Event)]
struct RefreshRequested;

events.observe( | _: On<RefreshRequested>| {
println ! ("刷新请求");
}) ?;
events.emit(RefreshRequested).await?;
```

`emit` 会等待自定义事件队列容量，`try_emit` 立即返回，队列满时返回 `EmitError::Full`。在同一 dispatcher 的回调内提交后续事件应使用
`try_emit`，避免队列满时等待自身消费而死锁。即便手动提交的是 Tao 类型或原始 `EventKind`，也只按提交的类型分发，不会再次路由。

`EventHandle::close()` 关闭统一事件队列，包含 Taao 转发在内的所有生产者都无法再提交；已排队事件排空后 `pump`
返回。转发器下一次提交时返回 `EmitError::Closed`，不会主动关闭 Tao 窗口。`Application` 在 `pump` 返回时结束运行，也会取消仍在等待平台输入的转发器。

Taao 事件流自然结束只释放转发器持有的句柄，不关闭其他生产者。丢弃全部句柄也会使事件队列结束；回调捕获的句柄和转发器持有的句柄都属于生产者。

Taao 0.1.3 的平台事件缓冲容量为 256，满时新事件会覆盖最旧事件；这与自定义队列的背压行为不同。因此耗时监听器可能导致平台输入丢失。事件在平台上的可用性和顺序遵循
Tao；本模块只做数据路由，不额外补齐平台缺失的事件。

## 对应源码

- [路由实现](../src/tao_event/routing.rs)
- [Taao 事件源](../src/tao_event/source.rs)
- [窗口事件类型](../src/tao_event/window.rs)
- [设备事件类型](../src/tao_event/device.rs)
- [事件循环类型](../src/tao_event/lifecycle.rs)
- [统一句柄](../src/event/handle.rs)
- [pump 实现](../src/event/dispatcher.rs)
- [完整 Tao 监听示例](../examples/tao_observe.rs)
