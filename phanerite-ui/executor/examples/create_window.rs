use phanerite_executor::event::On;
use phanerite_executor::tao_event::window::CloseRequested;
use phanerite_executor::window::{Window, WindowRender};
use phanerite_executor::{Application, Shutdown};

fn main() {
    Application::new()
        .run(async move |cx, exit| {
            let window = Window::builder().build(&cx).await.unwrap();
            let window_id = window.window.id();
            let handler = cx.events.clone();
            let render = WindowRender::new(&window, &cx)
                .await
                .expect("Failed to init GPU");
            render.register(&cx).unwrap();
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
