use phanerite_executor::event::On;
use phanerite_executor::tao_event::window::CloseRequested;
use phanerite_executor::window::Window;
use phanerite_executor::{Application, Shutdown};

fn main() {
    Application::new()
        .run(|cx, exit| async move {
            let window = Window::builder().build(&cx).await.unwrap();
            let window_id = window.window.id();
            let handler = cx.events.clone();
            cx.events
                .observe(async move |event: On<CloseRequested>| {
                    if event.window_id == window_id {
                        handler.emit(Shutdown).await.unwrap();
                    }
                })
                .unwrap();
            exit.await;
        })
        .unwrap();
}
