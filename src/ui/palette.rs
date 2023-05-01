use pixels_graphics_lib::prelude::*;

const SQUARE_SIZE: usize = 8;
const SPACING: usize = 2;
const PER_SQUARE: usize = SPACING + SQUARE_SIZE;

#[derive(Debug)]
pub struct PaletteView {
    bounds: Rect,
    colors: Vec<IciColor>,
    selected: u8,
    cols: usize,
}

impl PaletteView {
    pub fn new(xy: Coord, (width, height): (usize, usize)) -> Self {
        Self {
            bounds: Rect::new_with_size(xy, width, height),
            colors: vec![IciColor::transparent()],
            selected: 0,
            cols: 0,
        }
    }
}

impl PaletteView {
    pub fn set_palette(&mut self, new_colors: &[IciColor]) {
        self.colors = new_colors.to_vec();
        self.cols = self.bounds.width() / PER_SQUARE;
    }

    pub fn set_color_index(&mut self, idx: u8) {
        self.selected = idx;
    }

    pub fn get_selected_color(&self) -> IciColor {
        self.colors[self.selected as usize]
    }

    pub fn get_selected_idx(&self) -> u8 {
        self.selected
    }

    pub fn on_mouse_click(&mut self, mouse_xy: Coord) -> bool {
        if self.bounds.contains(mouse_xy) {
            let xy = mouse_xy - self.bounds.top_left();
            let x = xy.x / PER_SQUARE as isize;
            let y = xy.y / PER_SQUARE as isize;
            //the + y shouldn't be necessary
            //this bug is most like due a difference between rendering and math
            //the line is ending one too early when rendering or something
            let i = x + y * (self.cols as isize) + y;
            if i >= 0 && i < self.colors.len() as isize {
                self.selected = i as u8;
                return true;
            }
        }
        false
    }
}

impl UiElement for PaletteView {
    fn bounds(&self) -> &Rect {
        &self.bounds
    }

    fn render(&self, graphics: &mut Graphics, mouse_xy: Coord) {
        let orig_trans = graphics.get_translate();
        graphics.set_translate(self.bounds.top_left());

        let mut x = 0;
        let mut y = 0;
        for (idx, color) in self.colors.iter().enumerate() {
            let top_left = Coord::from((x, y)) * PER_SQUARE;
            if color.a == 0 {
                graphics.draw_rect(
                    Rect::new_with_size(top_left, SQUARE_SIZE, SQUARE_SIZE),
                    stroke(WHITE),
                );
                graphics.draw_line(top_left, top_left + (SQUARE_SIZE, SQUARE_SIZE), WHITE);
            } else {
                graphics.draw_rect(
                    Rect::new_with_size(top_left, SQUARE_SIZE, SQUARE_SIZE),
                    fill(color.to_color()),
                );
            }

            if idx == self.selected as usize {
                graphics.draw_rect(
                    Rect::new_with_size(top_left - 1, SQUARE_SIZE + 2, SQUARE_SIZE + 2),
                    stroke(WHITE),
                );
            }
            x += 1;
            if x > self.cols {
                x = 0;
                y += 1;
            }
        }

        graphics.set_translate(orig_trans);
    }

    fn update(&mut self, _: &Timing) {}

    fn set_state(&mut self, _: ElementState) {
        unimplemented!("Palette always enabled")
    }

    fn get_state(&self) -> ElementState {
        ElementState::Normal
    }
}
