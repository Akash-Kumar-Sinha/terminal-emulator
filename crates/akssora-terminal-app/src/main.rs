use akssora_terminal_app::App;
use winit::event_loop::EventLoop;

fn main() {
    let event_loop = EventLoop::new().expect("failed to create winit event loop");
    let mut app = App::new();

    let _ = event_loop.run_app(&mut app);
}
