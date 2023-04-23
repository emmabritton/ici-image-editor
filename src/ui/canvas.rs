use log::error;
use pixels_graphics_lib::prelude::*;

#[derive(Debug)]
pub struct Canvas {
    bounds: Rect,
    image: IndexedImage,
    screen_px_per_image_px: usize,
    trans_background_colors: (Color, Color),
    cursor_color: Color,
    selected_color_idx: u8,
}

impl Canvas {
    pub fn new(xy: Coord, (width, height): (usize, usize)) -> Self {
        Self {
            bounds: Rect::new_with_size(xy, width, height),
            image: IndexedImage::new(1, 1, vec![IciColor::transparent()], vec![0]).unwrap(),
            screen_px_per_image_px: 1,
            trans_background_colors: (LIGHT_GRAY, DARK_GRAY),
            cursor_color: RED,
            selected_color_idx: 1
        }
    }
}

impl Canvas {
    pub fn set_image(&mut self, image: IndexedImage) {
        self.image = image;
        let side = self.image.width().max(self.image.height()) as usize;
        let length = self.bounds.width().max(self.bounds.height());
        self.screen_px_per_image_px = length / side;
    }

    pub fn get_image(&self) -> &IndexedImage {
        &self.image
    }

    pub fn on_mouse_click(&mut self, mouse_xy: Coord) {
        if self.bounds.contains(mouse_xy) {
            let offset_xy = mouse_xy - self.bounds.top_left();
            let img_coord = offset_xy / self.screen_px_per_image_px;
            let x = img_coord.x.min(255).max(0) as u8;
            let y = img_coord.y.min(255).max(0) as u8;
            match self.image.get_pixel_index(x,y) {
                Ok(px_idx) => self.image.set_pixel(px_idx, self.selected_color_idx).expect("Canvas palette data corrupt?"),
                Err(e) =>
                    error!("Attempted to draw outside canvas: {e:?} at mouse {mouse_xy:?}, px {img_coord:?}, bounds: {},{}", self.bounds.width(), self.bounds.height())
            }
        }
    }

    pub fn trans_background_colors(&self) -> (Color, Color) {
        self.trans_background_colors
    }

    pub fn set_trans_background_colors(&mut self, trans_background_colors: (Color, Color)) {
        self.trans_background_colors = trans_background_colors;
    }

    pub fn set_cursor_color(&mut self, cursor_color: Color) {
        self.cursor_color = cursor_color;
    }

    fn draw_cursor(&self, graphics: &mut Graphics, mouse_xy: Coord) {
        if self.bounds.contains(mouse_xy) {
            let offset_xy = mouse_xy - self.bounds.top_left();
            let img_coord = offset_xy / self.screen_px_per_image_px;
            let px_size = self.screen_px_per_image_px as isize;
            graphics.draw_rect(
                Rect::new_with_size(
                    self.bounds.top_left()+(
                        img_coord.x * px_size,
                        img_coord.y * px_size,
                    ),
                    self.screen_px_per_image_px-1,
                    self.screen_px_per_image_px-1,
                ),
                stroke(self.cursor_color),
            );
        }
    }
    fn draw_img_px(&self, graphics: &mut Graphics, img_x: u8, img_y: u8, trans_color: Color) {
        let img_i = self.image.get_pixel_index(img_x, img_y).unwrap();
        let color_idx = self.image.get_pixel(img_i).unwrap();
        let color = self.image.get_color(color_idx).unwrap();
        let px_size = self.screen_px_per_image_px as isize;
        let scr_x = img_x as isize * px_size;
        let scr_y = img_y as isize * px_size;
        if color.is_transparent() {
            match self.screen_px_per_image_px {
                1 => graphics.set_pixel(scr_x, scr_y, trans_color),
                _ => graphics.draw_rect(
                    Rect::new_with_size(
                        (
                            scr_x,
                            scr_y,
                        ),
                        self.screen_px_per_image_px-1,
                        self.screen_px_per_image_px-1,
                    ),
                    fill(trans_color),
                ),
            }
        } else {
            match self.screen_px_per_image_px {
                1 => graphics.set_pixel(scr_x, scr_y, color.to_color()),
                _ => graphics.draw_rect(
                    Rect::new_with_size(
                        (
                            scr_x,
                            scr_y,
                        ),
                        self.screen_px_per_image_px-1,
                        self.screen_px_per_image_px-1,
                    ),
                    fill(color.to_color()),
                ),
            }
        }
    }
}

impl UiElement for Canvas {
    fn bounds(&self) -> &Rect {
        &self.bounds
    }

    fn render(&self, graphics: &mut Graphics, mouse_xy: Coord) {
        let mut trans_color = self.trans_background_colors.0;
        let swap_color = |color: &mut Color| {
            if color == &self.trans_background_colors.0 {
                *color = self.trans_background_colors.1;
            } else {
                *color = self.trans_background_colors.0;
            }
        };

        let orig_trans = graphics.get_translate();
        graphics.set_translate(self.bounds.top_left());

        for img_y in 0..self.image.height() {
            for img_x in 0..self.image.width() {
                self.draw_img_px(graphics, img_x, img_y, trans_color);
                swap_color(&mut trans_color);
            }
            swap_color(&mut trans_color);
        }

        graphics.set_translate(orig_trans);
        self.draw_cursor(graphics, mouse_xy);
        // graphics.draw_rect(self.bounds, stroke(LIGHT_GRAY));
    }

    fn update(&mut self, _: &Timing) {}

    fn set_state(&mut self, _: ElementState) {
        unimplemented!("not supported for canvas")
    }

    fn get_state(&self) -> ElementState {
        ElementState::Normal
    }
}
