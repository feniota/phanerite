use phanerite_executor::tao_event::window::CloseRequested;
use phanerite_executor::window::Window;
use phanerite_executor::{
    Application, Shutdown,
    event::On,
    tao_event::window::{MouseInput, Resized},
};

fn main() {
    let app = Application::new();

    app.run(|cx, exit| async move {
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

        exit.await;
    })
    .unwrap();
}
