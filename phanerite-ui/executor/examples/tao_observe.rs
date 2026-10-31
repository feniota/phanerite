use phanerite_executor::{
    Application,
    event::On,
    tao_event::window::{MouseInput, Resized},
};

fn main() {
    let mut app = Application::new();
    app.event_dispatcher_mut()
        .observe(|event: On<MouseInput>| {
            println!(
                "window {:?}: {:?} {:?}",
                event.window_id, event.button, event.state
            );
        })
        .observe(async |event: On<Resized>| {
            println!("window {:?}: {:?}", event.window_id, event.size);
        });
    app.run().unwrap();
}
