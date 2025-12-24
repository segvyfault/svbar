mod modules;

mod app;
use app::BarWindow;

use std::time::Duration;
use std::sync::{Arc, RwLock};

use smithay_client_toolkit::{
    compositor::CompositorState,

    reexports::calloop::EventLoop,
    reexports::calloop_wayland_source::WaylandSource,

    shell::{
        wlr_layer::{
            Layer, LayerShell,
            Anchor, KeyboardInteractivity
        },
        WaylandSurface,
    },
};
use wayland_client::{
    globals::registry_queue_init,
    Connection
};

const WINDOW_WIDTH:  u32 = 1920;
const WINDOW_HEIGHT: u32 = 24;

fn main() {
    let conn = Connection::connect_to_env().unwrap();

    let (globals, event_queue) = registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();
    let mut event_loop: EventLoop<BarWindow> =
        EventLoop::try_new().expect("Failed to initialize the event loop!");
    let loop_handle = event_loop.handle();
    WaylandSource::new(conn.clone(), event_queue).insert(loop_handle).unwrap();

    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor not available");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("zwlr_layer_shell_v1 not available");

    let surface = compositor.create_surface(&qh);
    let surface = layer_shell.create_layer_surface(&qh, surface, Layer::Top, None::<String>, None);
    surface.set_anchor(Anchor::TOP);
    surface.set_keyboard_interactivity(KeyboardInteractivity::None);
    surface.set_size(WINDOW_WIDTH, WINDOW_HEIGHT);
    surface.commit();

    let exiting = Arc::new(RwLock::new(false));

    let mut window = BarWindow::new(
        WINDOW_WIDTH, WINDOW_HEIGHT, 
        &globals, &qh, 
        surface, exiting.clone()
    );

    ctrlc::set_handler(move || {
        let mut exiting = exiting.write().expect("Failed to handle ctrlc, not able to write");
        *exiting = true;
    }).expect("failed to set handler");

    loop {
        event_loop.dispatch(Duration::from_millis(15), &mut window).unwrap();

        if let Ok(exiting) = window.state.exiting.clone().read() && *exiting {
            window.state.module_cleanup();
            println!("exiting");
            break;
        }
    }
}
