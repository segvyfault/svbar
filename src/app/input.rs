use super::BarWindow;

use smithay_client_toolkit::{
    seat::pointer::{PointerEvent, PointerEventKind::*}, 
    shell::WaylandSurface
};

impl BarWindow {
    pub (super) fn handle_input_event(&mut self, events: &[PointerEvent]) {
        for event in events {
            if &event.surface != self.wayland.surface.wl_surface() {
                continue;
            }

            match event.kind {
                Press { button, .. } => if button == 273 {
                    self.state.no_disappearing = !self.state.no_disappearing;

                    if event.position.1 == 0.0 && event.position.0 > 0.0 {
                        self.state.bar_width = event.position.0 as u32;
                    }
                    else if event.position.0 >= 0.0 || event.position.1 >= 0.0 {
                        self.state.bar_width = 0;
                    }
                }

                Leave { .. } => if !self.state.no_disappearing {
                    self.state.bar_width = 0;
                    self.state.track_x = false;

                    let x = event.position.0;
                    let y = event.position.1;

                    if x == 0.0 && self.state.track_y && y > self.graphics.height as f64 - 1.5 {
                        self.state.execute_command("niri msg action open-overview");
                    }

                    self.state.track_y = false;
                }

                Motion { .. } => if !self.state.no_disappearing {
                    let x = event.position.0;
                    let y = event.position.1;

                    if event.position == (0.0, 0.0) {
                        self.state.track_x = true;
                        self.state.track_y = true;
                    }
                    else if x > 0.0 && y > 0.0 {
                        self.state.track_x = false;
                        self.state.track_y = false;
                    }

                    if y == 0.0 && x > 0.0 && self.state.track_x {
                        self.state.bar_width = x as u32;
                    }
                    else if x == 0.0 && self.state.track_y && y > self.graphics.height as f64 - 1.5{
                        self.state.execute_command("niri msg action open-overview");
                    }
                    else if x > 0.0 {
                        self.state.bar_width = 0;
                    }
                }
                _ => {}
            }
        }
    }
}
