use crate::graphics_shapes::coord;
use crate::ui::edit_history::EditHistory;
use log::error;
use pixels_graphics_lib::prelude::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Tool {
    Pencil,
    Line,
    Rect,
    Fill,
}

#[derive(Debug)]
pub struct Canvas {
    bounds: Rect,
    inner_bounds: Rect,
    image: IndexedImage,
    screen_px_per_image_px: usize,
    trans_background_colors: (Color, Color),
    cursor_color: Color,
    selected_color_idx: u8,
    tool: Tool,
    first_click_at: Option<(u8, u8)>,
    state: ElementState,
}

impl Canvas {
    pub fn new(xy: Coord, (width, height): (usize, usize)) -> Self {
        Self {
            bounds: Rect::new_with_size(xy, width, height),
            inner_bounds: Rect::new_with_size(xy, 0, 0),
            image: IndexedImage::new(1, 1, vec![IciColor::transparent()], vec![0]).unwrap(),
            screen_px_per_image_px: 1,
            trans_background_colors: (LIGHT_GRAY, DARK_GRAY),
            cursor_color: RED,
            selected_color_idx: 1,
            tool: Tool::Pencil,
            first_click_at: None,
            state: ElementState::Normal,
        }
    }
}

impl Canvas {
    pub fn set_image(&mut self, image: IndexedImage) {
        let width = image.width() as usize;
        let height = image.height() as usize;
        self.image = image;
        let side = width.max(height);
        let length = self.bounds.width().min(self.bounds.height());
        self.screen_px_per_image_px = length / side;
        self.inner_bounds = Rect::new_with_size(
            (0, 0),
            width * self.screen_px_per_image_px - 1,
            height * self.screen_px_per_image_px - 1,
        );
        let max_width = self.bounds.width() / 2;
        let max_height = self.bounds.height() / 2;
        let image_width = self.inner_bounds.width() / 2;
        let image_height = self.inner_bounds.height() / 2;
        self.inner_bounds = self.inner_bounds.move_to(coord!(
            self.bounds.top_left().x + ((max_width - image_width) as isize),
            self.bounds.top_left().y + ((max_height - image_height) as isize),
        ));
    }

    pub fn get_image(&self) -> &IndexedImage {
        &self.image
    }

    pub fn on_mouse_down(&mut self, mouse_xy: Coord, edit_history: &mut EditHistory) -> bool {
        if self.inner_bounds.contains(mouse_xy) && self.state == ElementState::Normal {
            let (x, y) = self.mouse_to_image(mouse_xy);
            if self.tool == Tool::Pencil {
                edit_history
                    .add_pencil((x, y), self.selected_color_idx)
                    .unwrap();
                return true;
            } else if self.first_click_at.is_none() {
                self.first_click_at = Some((x, y));
            }
        }
        false
    }

    pub fn on_mouse_up(&mut self, mouse_xy: Coord, edit_history: &mut EditHistory) {
        if self.inner_bounds.contains(mouse_xy) && self.state == ElementState::Normal {
            let (x, y) = self.mouse_to_image(mouse_xy);
            let result = match (self.tool, self.first_click_at) {
                (Tool::Line, Some(start)) => {
                    edit_history.add_line(start, (x, y), self.selected_color_idx)
                }
                (Tool::Rect, Some(start)) => {
                    edit_history.add_rect(start, (x, y), self.selected_color_idx)
                }
                (Tool::Fill, Some(start)) => edit_history.add_fill(start, self.selected_color_idx),
                _ => Ok(()),
            };
            if let Err(e) = result {
                error!("Error drawing {:?} at {x},{y}: {e:?}", self.tool);
            }
        }
        self.first_click_at = None;
    }

    #[allow(unused)] //will be one day
    pub fn trans_background_colors(&self) -> (Color, Color) {
        self.trans_background_colors
    }

    #[allow(unused)] //will be one day
    pub fn set_trans_background_colors(&mut self, trans_background_colors: (Color, Color)) {
        self.trans_background_colors = trans_background_colors;
    }

    pub fn set_color_index(&mut self, idx: u8) {
        if let Ok(color) = self.image.get_color(idx) {
            self.cursor_color = color.to_color();
            self.selected_color_idx = idx;
        }
    }

    fn mouse_to_image(&self, mouse_xy: Coord) -> (u8, u8) {
        let offset_xy = mouse_xy - self.inner_bounds.top_left();
        let img_coord = offset_xy / self.screen_px_per_image_px;
        let x = img_coord.x.min(255).max(0) as u8;
        let y = img_coord.y.min(255).max(0) as u8;
        (x, y)
    }

    pub fn set_tool(&mut self, tool: Tool) {
        self.tool = tool;
    }

    pub fn get_palette(&self) -> &[IciColor] {
        self.image.get_palette()
    }

    pub fn get_usage_state(&self) -> (Tool, Color, u8) {
        (self.tool, self.cursor_color, self.selected_color_idx)
    }

    pub fn set_usage_state(&mut self, (tool, cursor, selected): (Tool, Color, u8)) {
        self.tool = tool;
        self.cursor_color = cursor;
        self.selected_color_idx = selected;
    }
}

impl Canvas {
    fn draw_mouse_highlight(&self, graphics: &mut Graphics, mouse_xy: Coord) {
        if self.inner_bounds.contains(mouse_xy) {
            let xy = self.mouse_to_image(mouse_xy);
            self.draw_cursor_on_image(graphics, xy);
        }
    }

    fn draw_cursor_on_image(&self, graphics: &mut Graphics, xy: (u8, u8)) {
        let top_left =
            (Coord::from(xy) * self.screen_px_per_image_px) + self.inner_bounds.top_left();
        if self.cursor_color.is_transparent() {
            let mut color = BLACK;
            color.a = 125;
            graphics.draw_line(
                top_left,
                top_left
                    + (
                        self.screen_px_per_image_px - 1,
                        self.screen_px_per_image_px - 1,
                    ),
                color,
            );
            graphics.draw_line(
                top_left + (self.screen_px_per_image_px - 1, 0),
                top_left + (0, self.screen_px_per_image_px - 1),
                color,
            );
        } else {
            graphics.draw_rect(
                Rect::new_with_size(
                    top_left,
                    self.screen_px_per_image_px - 1,
                    self.screen_px_per_image_px - 1,
                ),
                stroke(self.cursor_color),
            );
        }
    }

    fn draw_img_px(
        &self,
        graphics: &mut Graphics,
        image: &IndexedImage,
        img_x: u8,
        img_y: u8,
        trans_color: Color,
    ) {
        let img_i = image.get_pixel_index(img_x, img_y).unwrap();
        let color_idx = image.get_pixel(img_i).unwrap();
        let color = image.get_color(color_idx).unwrap();
        let px_size = self.screen_px_per_image_px as isize;
        let scr_x = img_x as isize * px_size;
        let scr_y = img_y as isize * px_size;
        if color.is_transparent() {
            match self.screen_px_per_image_px {
                1 => graphics.set_pixel(scr_x, scr_y, trans_color),
                _ => graphics.draw_rect(
                    Rect::new_with_size(
                        (scr_x, scr_y),
                        self.screen_px_per_image_px - 1,
                        self.screen_px_per_image_px - 1,
                    ),
                    fill(trans_color),
                ),
            }
        } else {
            match self.screen_px_per_image_px {
                1 => graphics.set_pixel(scr_x, scr_y, color.to_color()),
                _ => graphics.draw_rect(
                    Rect::new_with_size(
                        (scr_x, scr_y),
                        self.screen_px_per_image_px - 1,
                        self.screen_px_per_image_px - 1,
                    ),
                    fill(color.to_color()),
                ),
            }
        }
    }

    fn temp_line(&self, graphics: &mut Graphics, start: (u8, u8), mouse_xy: Coord) {
        let end = self.mouse_to_image(mouse_xy);

        let points = Line::new(start, end).outline_pixels();
        for point in points {
            self.draw_cursor_on_image(graphics, (point.x as u8, point.y as u8));
        }
    }

    fn temp_rect(&self, graphics: &mut Graphics, start: (u8, u8), mouse_xy: Coord) {
        let end = self.mouse_to_image(mouse_xy);
        let top_left = ((start.0).min(end.0), (start.1).min(end.1));
        let bottom_right = ((start.0).max(end.0), (start.1).max(end.1));

        for x in top_left.0..bottom_right.0 {
            self.draw_cursor_on_image(graphics, (x, top_left.1));
            self.draw_cursor_on_image(graphics, (x, bottom_right.1));
        }

        for y in top_left.1..=bottom_right.1 {
            self.draw_cursor_on_image(graphics, (top_left.0, y));
            self.draw_cursor_on_image(graphics, (bottom_right.0, y));
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

        let orig_trans = graphics.set_translate(self.inner_bounds.top_left());

        for img_y in 0..self.image.height() {
            for img_x in 0..self.image.width() {
                self.draw_img_px(graphics, &self.image, img_x, img_y, trans_color);
                swap_color(&mut trans_color);
            }
            if self.image.width() % 2 == 0 {
                swap_color(&mut trans_color);
            }
        }

        graphics.set_translate(orig_trans);
        if self.inner_bounds.contains(mouse_xy) && self.state == ElementState::Normal {
            match (self.tool, self.first_click_at) {
                (Tool::Line, Some(start)) => self.temp_line(graphics, start, mouse_xy),
                (Tool::Rect, Some(start)) => self.temp_rect(graphics, start, mouse_xy),
                _ => self.draw_mouse_highlight(graphics, mouse_xy),
            }
        }
    }

    fn update(&mut self, _: &Timing) {}

    fn set_state(&mut self, state: ElementState) {
        self.state = state;
    }

    fn get_state(&self) -> ElementState {
        self.state
    }
}
