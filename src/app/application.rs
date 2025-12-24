use std::process::Command;
use std::sync::{Arc, RwLock};

use smithay_client_toolkit::{
    delegate_compositor, delegate_layer, delegate_output,
    delegate_pointer, delegate_registry, delegate_seat, delegate_shm, 

    compositor::CompositorHandler, 
    output::{OutputHandler, OutputState}, 

    registry::{ProvidesRegistryState, RegistryState}, 
    registry_handlers,

    seat::{
        Capability, SeatHandler, SeatState, 
        pointer::{PointerEvent, PointerHandler}
    },
    shell::wlr_layer::{
        LayerShellHandler, LayerSurface, LayerSurfaceConfigure
    },
    shm::{Shm, ShmHandler}
};
use wayland_client::{
    protocol::{wl_output, wl_pointer, wl_seat, wl_surface},
    Connection, QueueHandle,
    globals::GlobalList,
};

use crate::modules::{
    ClockModule, AudioModule,
    ModuleInfo,
};

use super::config::ConfigState;
use super::drawing::GraphicsState;

pub struct WaylandState {
    pub registry_state: RegistryState,
    pub seat_state: SeatState,
    pub output_state: OutputState,

    pub surface: LayerSurface,
    pub pointer: Option<wl_pointer::WlPointer>,
}

pub struct AppState {
    pub first_configure: bool,
    pub exiting: Arc<RwLock<bool>>,

    pub track_x: bool, 
    pub track_y: bool,
    pub no_disappearing: bool,

    pub bar_width: u32,
    pub modules: Vec<Box<dyn ModuleInfo>>
}

impl AppState {
    pub fn new(exiting: Arc<RwLock<bool>>) -> Self {
        let modules: Vec<Box<dyn ModuleInfo>> = vec![
            Box::new(ClockModule::new()),
            Box::new(AudioModule::new()),
        ];

        Self { 
            first_configure: true,
            exiting,

            track_x: false,
            track_y: false,
            no_disappearing: false,

            bar_width: 0,
            modules
        }
    }

    pub fn module_cleanup(&mut self) {
        self.modules.iter_mut()
            .for_each(|m| m.clean_up());
    }

    pub fn get_modules_display(&mut self) -> String {
        self.modules.iter_mut()
            .map(|m| m.display())
            .rev()
            .collect::<Vec<String>>()
            .join(" ")
    }

    pub fn execute_command(&self, command: &str) {
        let mut argv = command.split(' ');
        let mut command = Command::new(argv.nth(0).unwrap());
        let argv = argv.skip(0);
        command.args(argv).output().unwrap();
    }
}

pub struct BarWindow {
    pub wayland: WaylandState,
    pub graphics: GraphicsState,

    pub state: AppState,
    pub config: ConfigState
}

impl BarWindow {
    pub fn new(
        width: u32, height: u32,
        globals: &GlobalList, 
        qh: &QueueHandle<Self>,
        surface: LayerSurface,
        exiting: Arc<RwLock<bool>>,
    ) -> Self {
        Self {
            wayland: WaylandState { 
                registry_state: RegistryState::new(globals), 
                seat_state: SeatState::new(globals, qh), 
                output_state: OutputState::new(globals, qh), 
                surface, 
                pointer: None
            },
            graphics: GraphicsState::new(width, height, globals, qh),
            state: AppState::new(exiting),
            config: ConfigState::new()
        }
    }

}

impl PointerHandler for BarWindow {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        self.handle_input_event(events);
    }
}

impl CompositorHandler for BarWindow {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {}

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {}

    fn frame(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        self.draw(qh);
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {}

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {}
}

impl LayerShellHandler for BarWindow {
    fn closed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &LayerSurface) {
        *self.state.exiting.write().unwrap() = true;
    }

    fn configure(
            &mut self,
            _: &Connection,
            qh: &QueueHandle<Self>,
            _: &LayerSurface,
            configure: LayerSurfaceConfigure,
            _: u32,
    ) {
        self.graphics.width = configure.new_size.0;
        self.graphics.height = configure.new_size.1;

        if self.state.first_configure {
            self.draw(qh);
            self.state.first_configure = false;
        }
    }
}

impl OutputHandler for BarWindow {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.wayland.output_state
    }

    fn new_output(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: wl_output::WlOutput,
    ) {}

    fn update_output(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: wl_output::WlOutput,
    ) {}

    fn output_destroyed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: wl_output::WlOutput,
    ) {}
}

impl SeatHandler for BarWindow {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.wayland.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Pointer && self.wayland.pointer.is_none() {
            let pointer = self.wayland.seat_state.get_pointer(qh, &seat).expect("Failed to create pointer");
            self.wayland.pointer = Some(pointer);
        }
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Pointer && self.wayland.pointer.is_some() {
            self.wayland.pointer.take().unwrap().release();
        }
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl ShmHandler for BarWindow {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.graphics.shm
    }
}

delegate_compositor!(BarWindow);
delegate_output!(BarWindow);
delegate_shm!(BarWindow);

delegate_seat!(BarWindow);
delegate_pointer!(BarWindow);

delegate_layer!(BarWindow);
delegate_registry!(BarWindow);

impl ProvidesRegistryState for BarWindow {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.wayland.registry_state
    }
    registry_handlers![OutputState, SeatState,];
}
