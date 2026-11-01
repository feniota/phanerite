use phanerite_executor::tao_event::window::CloseRequested;
use phanerite_executor::window::{Window, WindowRender};
use phanerite_executor::{
    Application, Shutdown,
    event::On,
    tao_event::window::{MouseInput, Resized},
};

fn main() {
    let app = Application::new();

    app.run(async move |cx, exit| {
        let window = Window::builder().build(&cx).await.unwrap();
        let window_id = window.window.id();

        let render = WindowRender::new(&window, &cx)
            .await
            .expect("Failed to init GPU");
        render.register(&cx).unwrap();

        cx.events
            .observe(|event: On<MouseInput>| {
                println!(
                    "window {:?}: {:?} {:?}",
                    event.window_id, event.button, event.state
                );
            })
            .unwrap();
        cx.events
            .observe(async |event: On<Resized>| {
                println!("window {:?}: {:?}", event.window_id, event.size);
            })
            .unwrap();

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
