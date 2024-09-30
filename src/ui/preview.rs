use pixels_graphics_lib::buffer_graphics_lib::prelude::*;
use pixels_graphics_lib::prelude::*;
use pixels_graphics_lib::ui::prelude::*;

const COLOR_BUTTON_HEIGHT: usize = 8;

const COLORS: [Color; 4] = [WHITE, BLACK, LIGHT_GRAY, DARK_GRAY];

#[derive(Debug)]
pub struct Preview {
    bounds: Rect,
    image: IndexedImage,
    background: usize,
    show_bg_options: bool,
}

impl Preview {
    pub fn new(bounds: Rect, show_bg_options: bool) -> Self {
        Self {
            bounds,
            image: IndexedImage::new(4, 4, vec![TRANSPARENT], vec![0; 16]).unwrap(),
            background: 0,
            show_bg_options,
        }
    }
}

impl Preview {
    pub fn set_image(&mut self, image: IndexedImage) {
        self.image = image;
    }

    pub fn on_mouse_click(&mut self, xy: Coord) -> Color {
        if self.show_bg_options
            && Rect::new_with_size(
                self.bounds.top_left(),
                self.bounds.width(),
                COLOR_BUTTON_HEIGHT,
            )
            .contains(xy)
        {
            let color_width = self.bounds.width() / COLORS.len();
            self.background =
                (((xy - self.bounds.top_left()).x / color_width as isize) as usize).clamp(0, 3);
        }
        COLORS[self.background]
    }

    pub fn set_background(&mut self, idx: usize) {
        self.background = idx;
    }

    pub fn selected_background(&self) -> usize {
        self.background
    }
}

impl PixelView for Preview {
    fn set_position(&mut self, top_left: Coord) {
        self.bounds = self.bounds.move_to(top_left);
    }

    fn bounds(&self) -> &Rect {
        &self.bounds
    }

    fn render(&self, graphics: &mut Graphics, _: &MouseData) {
        graphics.draw_rect(self.bounds.clone(), fill(COLORS[self.background]));
        let y_offset = if self.show_bg_options {
            let color_width = self.bounds.width() / COLORS.len();
            for (i, color) in COLORS.iter().enumerate() {
                let rect = Rect::new_with_size(
                    self.bounds.top_left() + (i * color_width, 0),
                    color_width,
                    COLOR_BUTTON_HEIGHT,
                );
                graphics.draw_rect(rect, fill(*color));
            }
            COLOR_BUTTON_HEIGHT
        } else {
            0
        };

        let x = if self.image.width() as usize >= self.bounds.width() {
            0
        } else {
            (self.bounds.width() / 2) - (self.image.width() as usize / 2)
        };

        let y = if self.image.height() as usize >= (self.bounds.height() - y_offset) {
            y_offset
        } else {
            ((self.bounds.height() - y_offset) / 2 - (self.image.height() as usize / 2)) + y_offset
        };
        graphics.draw_indexed_image(self.bounds.top_left() + (x, y + 1), &self.image);
    }

    fn update(&mut self, _: &Timing) {}

    fn set_state(&mut self, _: ViewState) {
        unimplemented!("Preview is always normal")
    }

    fn get_state(&self) -> ViewState {
        ViewState::Normal
    }
}
