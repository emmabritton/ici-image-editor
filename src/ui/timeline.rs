use pixels_graphics_lib::buffer_graphics_lib::prelude::*;
use pixels_graphics_lib::prelude::*;
use pixels_graphics_lib::ui::prelude::*;

const PADDING: isize = 2;

#[derive(Debug)]
pub struct Timeline {
    bounds: Rect,
    frames: Vec<IndexedImage>,
    offset: usize,
    selected: usize,
    frame_size: (usize, usize),
    background: Color,
    state: ViewState,
}

impl Timeline {
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            frames: vec![],
            offset: 0,
            selected: 0,
            frame_size: (0, 0),
            background: WHITE,
            state: ViewState::Normal,
        }
    }
}

impl Timeline {
    pub fn on_mouse_click(&mut self, mouse_xy: Coord) -> Option<usize> {
        if self.bounds.contains(mouse_xy) && self.state == ViewState::Normal {
            let x = mouse_xy.x - self.bounds.left() + self.offset as isize - 2;
            let selected = (x / (self.frame_size.0 as isize + 2)) as usize;
            if selected < self.frames.len() {
                self.selected = selected;
                Some(self.selected)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn on_scroll(&mut self, mouse_xy: Coord, x_diff: isize) {
        if self.bounds.contains(mouse_xy) && self.state == ViewState::Normal {
            let max_visible_count = self.bounds.width() / (self.frame_size.0 + 2);
            let last_frame = self.frames.len() as isize - max_visible_count as isize + 2;
            let maximum = self.frame_size.0 as isize * last_frame.max(0);
            self.offset = ((self.offset as isize) + x_diff).max(0).min(maximum) as usize
        }
    }

    pub fn set_frames(&mut self, frames: Vec<IndexedImage>, active: usize) {
        assert!(!frames.is_empty());
        self.frame_size = (frames[0].width() as usize, frames[0].height() as usize);
        self.frames = frames;
        self.selected = active;
        self.center_on_frame(active);
    }

    pub fn update_frame(&mut self, frame: IndexedImage) {
        self.frames.remove(self.selected);
        self.frames.insert(self.selected, frame);
    }

    pub fn set_background(&mut self, background: Color) {
        self.background = background;
    }

    pub fn set_active(&mut self, idx: usize) {
        self.selected = idx;
        self.center_on_frame(idx);
    }
}

impl Timeline {
    fn is_frame_visible(&self, idx: usize) -> bool {
        let x = (idx * self.frame_size.0) as isize;
        let frame_rect = Rect::new_with_size(
            (x - self.offset as isize, 0),
            self.frame_size.0,
            self.frame_size.1,
        );
        let visible_rect = Rect::new_with_size((0, 0), self.bounds.width(), self.frame_size.1);
        frame_rect.intersects_rect(&visible_rect)
    }

    fn center_on_frame(&mut self, idx: usize) {
        let max_visible_count = self.bounds.width() / (self.frame_size.0 + 2);
        self.offset = if self.frames.len() <= max_visible_count || self.bounds.width() == 0 {
            0
        } else {
            let mid_view = (self.bounds.width() / 2) - (self.frame_size.0 / 2);
            let frame_start = (self.frame_size.0 + 2) * idx;
            let offset = mid_view + frame_start;
            let limit = (self.frames.len() - max_visible_count) * (self.frame_size.0 + 2);
            offset.max(0).min(limit)
        };
    }
}

impl PixelView for Timeline {
    fn set_position(&mut self, _top_left: Coord) {
        unimplemented!("Does not support moving")
    }

    fn bounds(&self) -> &Rect {
        &self.bounds
    }

    fn render(&self, graphics: &mut Graphics, _: &MouseData) {
        graphics.clip_mut().set_valid_rect(self.bounds.clone());
        let y = self.bounds.top() + PADDING;
        let start_x = self.bounds.left() + PADDING;
        for (i, frame) in self.frames.iter().enumerate() {
            if self.is_frame_visible(i) {
                let x = (i as isize) * (self.frame_size.0 as isize + PADDING) + start_x
                    - (self.offset as isize);
                graphics.draw_rect(
                    Rect::new_with_size(
                        Coord::new(x, y),
                        self.frame_size.0 - 1,
                        self.frame_size.1 - 1,
                    ),
                    fill(self.background),
                );
                graphics.draw_indexed_image((x, y), frame);
                if self.selected == i {
                    graphics.draw_rect(
                        Rect::new_with_size(
                            Coord::new(x, y) - (1, 1),
                            self.frame_size.0 + 1,
                            self.frame_size.1 + 1,
                        ),
                        stroke(CYAN),
                    );
                }
            }
        }
        graphics.clip_mut().set_all_valid();
    }

    fn update(&mut self, _: &Timing) {}

    fn set_state(&mut self, state: ViewState) {
        self.state = state;
    }

    fn get_state(&self) -> ViewState {
        self.state
    }
}
