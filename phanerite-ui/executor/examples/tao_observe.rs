use phanerite_executor::{
    Application,
    event::On,
    tao_event::window::{MouseInput, Resized},
};

fn main() {
    let app = Application::new();
    let event = app.event_handle();

    event
        .observe(|event: On<MouseInput>| {
            println!(
                "window {:?}: {:?} {:?}",
                event.window_id, event.button, event.state
            );
        })
        .unwrap();
    event
        .observe(async |event: On<Resized>| {
            println!("window {:?}: {:?}", event.window_id, event.size);
        })
        .unwrap();

    app.run().unwrap();
}
