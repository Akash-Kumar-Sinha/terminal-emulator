use crate::app::App;
use winit::event_loop::EventLoop;

mod app;
mod state;
mod fonts;
mod pty;
mod renderer;
mod terminal;
mod theme;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::default();

    let _ = event_loop.run_app(&mut app);
}
