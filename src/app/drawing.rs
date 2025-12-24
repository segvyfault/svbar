use super::BarWindow;

use smithay_client_toolkit::{
    shell::WaylandSurface,
    shm::{slot::{Buffer, SlotPool}, Shm}
};
use wayland_client::{QueueHandle, globals::GlobalList, protocol::wl_shm};
use ab_glyph::{Font, FontRef, ScaleFont, point};


pub struct GraphicsState {
    pub width: u32,
    pub height: u32,

    pub shm: Shm,
    pub pool: SlotPool,
    pub buffer: Option<Buffer>,
}

impl GraphicsState {
    pub fn new(width: u32, height: u32, globals: &GlobalList, qh: &QueueHandle<BarWindow>) -> Self {
        let shm = Shm::bind(globals, qh).expect("wl_shm not available");
        let pool = SlotPool::new((width * height * 4) as usize, &shm).expect("Failed to create pool");

        Self {
            width, height,
            shm, pool,
            buffer: None
        }
    }
}

impl BarWindow {
    pub (super) fn draw(&mut self, qh: &QueueHandle<Self>) {
        let width  = self.graphics.width;
        let height = self.graphics.height;
        let stride = self.graphics.width as i32 * 4;

        let buffer = self.graphics.buffer.get_or_insert_with(|| {
            self.graphics.pool
                .create_buffer(width as i32, height as i32, stride, wl_shm::Format::Argb8888)
                .expect("create buffer")
                .0
        });

        let canvas = match self.graphics.pool.canvas(buffer) {
            Some(canvas) => canvas,
            None => {
                // This should be rare, but if the compositor has not released the previous
                // buffer, we need double-buffering.
                let (second_buffer, canvas) = self
                    .graphics
                    .pool
                    .create_buffer(
                        width as i32,
                        height as i32,
                        stride,
                        wl_shm::Format::Argb8888,
                    )
                    .expect("create buffer");
                *buffer = second_buffer;
                canvas
            }
        };

        {
            canvas.chunks_exact_mut(4).enumerate().for_each(|(index, chunk)| {
                let color: i32 = if index % (height + 1) as usize <= 30 &&
                    index % width as usize <= self.state.bar_width as usize
                {
                    let hex = self.config.bar_color.as_hex();
                    let alpha = 0xFF << 24;

                    alpha + hex
                }
                else {
                    0x00 << 24
                };

                let array: &mut [u8; 4] = chunk.try_into().unwrap();
                *array = color.to_le_bytes();
            });

            let font_data = std::fs::read(
                "/usr/share/fonts/urw-fonts/C059-Roman.otf"
            ).unwrap();
            let font = FontRef::try_from_slice(&font_data).unwrap();

            let text = self.state.get_modules_display();
            let text_width: f32 = text
                .chars()
                .map(|c| font.as_scaled(20.0).h_advance(font.glyph_id(c)))
                .sum();

            let mut pen_x = self.state.bar_width as f32 - text_width - 8.0;

            for ch in text.chars() {
                let glyph = font
                    .glyph_id(ch)
                    .with_scale_and_position(20.0, point(pen_x, 18.0));

                if let Some(glyph) = font.outline_glyph(glyph) {
                    let bb = glyph.px_bounds();

                    glyph.draw(|gx, gy, v| {
                        let x = bb.min.x + gx as f32;
                        let y = (bb.min.y + gy as f32).floor();

                        if x < 0.0 || y < 0.0 {
                            return;
                        }

                        let x = x as u32;
                        let y = y as u32;

                        if x >= width || y >= height {
                            return;
                        }

                        let idx = ((y * width + x) * 4) as usize;

                        let (r,g,b) = {
                            let color = &self.config.text_color;
                            (
                                color.r as f32,
                                color.g as f32,
                                color.b as f32,
                            )
                        };

                        canvas[idx + 0] = (b * v) as u8; // B
                        canvas[idx + 1] = (g * v) as u8; // G
                        canvas[idx + 2] = (r * v) as u8; // R
                        canvas[idx + 3] = 0xff;          // A
                    });
                }

                pen_x += font.as_scaled(20.0).h_advance(font.glyph_id(ch));
            }
        }

        self.wayland.surface.wl_surface().damage_buffer(0, 0, width as i32, height as i32);
        self.wayland.surface.wl_surface().frame(qh, self.wayland.surface.wl_surface().clone());
        buffer.attach_to(self.wayland.surface.wl_surface()).expect("buffer attach");
        self.wayland.surface.commit();
    }
}
