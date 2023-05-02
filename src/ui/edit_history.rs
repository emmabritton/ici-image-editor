use std::mem::swap;
use fnv::FnvHashSet;
use pixels_graphics_lib::prelude::{IciColor, IndexedImage, IndexedImageError, Line, Shape};
use crate::ui::image_fill::fill_pixels;

#[derive(Debug, Clone)]
pub enum EditEvent {
    /// Pixels([idx], u8])
    Pixels(Vec<usize>, u8),
    /// Palette(new)
    Palette(Vec<IciColor>),
}

#[derive(Debug)]
pub struct FrameHistory {
    base_image: IndexedImage,
    current_image: IndexedImage,
    history: Vec<EditEvent>,
    index: usize,
}

impl FrameHistory {
    pub fn new(base_image: IndexedImage) -> Self {
        let current_image = base_image.clone();
        Self { base_image, current_image, history: vec![], index: 0 }
    }
}

impl FrameHistory {
    pub fn undo(&mut self)  -> Result<(), IndexedImageError >{
        if self.index >= 1 {
            self.index -= 1;
        }
        self.rebuild_current_image()
    }

    pub fn redo(&mut self) -> Result<(), IndexedImageError >{
        if self.index < self.history.len() - 1 {
            self.index += 1;
        }
        self.update_current_image(self.index - 1)
    }

    pub fn get_current_image(&self) -> &IndexedImage {
        &self.current_image
    }

    pub fn get_color(&self, idx: u8) -> Option<IciColor> {
        self.base_image.get_palette().get(idx as usize).cloned()
    }
}

impl FrameHistory {
    pub fn add_line(&mut self, start: (u8, u8), end: (u8, u8), color: u8) -> Result<(), IndexedImageError> {
        let points = Line::new(start, end).outline_pixels();
        let mut pixels = vec![];

        for point in points {
            let i = self.base_image.get_pixel_index(point.x as u8, point.y as u8)?;
            pixels.push(i);
        }

        let event = EditEvent::Pixels(pixels, color);
        self.add_event(event)
    }

    pub fn add_rect(&mut self, start: (u8, u8), end: (u8, u8), color: u8) -> Result<(), IndexedImageError> {
        let mut pixels = FnvHashSet::default();
        let top_left = ((start.0).min(end.0), (start.1).min(end.1));
        let bottom_right = ((start.0).max(end.0), (start.1).max(end.1));

        for x in top_left.0..bottom_right.0 {
            let i = self.base_image.get_pixel_index(x, top_left.1)?;
            pixels.insert(i);
            let i = self.base_image.get_pixel_index(x, bottom_right.1)?;
            pixels.insert(i);
        }

        for y in top_left.1..=bottom_right.1 {
            let i = self.base_image.get_pixel_index(top_left.0, y)?;
            pixels.insert(i);
            let i = self.base_image.get_pixel_index(bottom_right.0, y)?;
            pixels.insert(i);
        }

        let event = EditEvent::Pixels(pixels.into_iter().collect(), color);
        self.add_event(event)
    }

    pub fn add_fill(&mut self, xy: (u8, u8), color: u8) -> Result<(), IndexedImageError> {
        let pixels = fill_pixels(&self.current_image, xy)?;
        let event = EditEvent::Pixels(pixels, color);
        self.add_event(event)
    }

    pub fn add_pencil(&mut self, xy: (u8, u8), color: u8) -> Result<(), IndexedImageError> {
        let i = self.base_image.get_pixel_index(xy.0, xy.1)?;
        if self.current_image.get_pixel(i).unwrap() != color {
            let event = EditEvent::Pixels(vec![i], color);
            self.add_event(event)
        } else {
            Ok(())
        }
    }

    pub fn add_clear(&mut self) -> Result<(), IndexedImageError> {
        let size = self.base_image.width() as usize * self.base_image.height() as usize;
        let event = EditEvent::Pixels(vec![0; size], 0);
        self.add_event(event)
    }

    pub fn add_palette(&mut self, colors: &[IciColor]) -> Result<(), IndexedImageError> {
        let event = EditEvent::Palette(colors.to_vec());
        self.add_event(event)
    }
}

impl FrameHistory {
    fn add_event(&mut self, event: EditEvent) -> Result<(), IndexedImageError> {
        if (self.index as isize) < self.history.len() as isize {
            let mut temp = vec![];
            swap(&mut self.history, &mut temp);
            self.history = temp.into_iter().take(self.index).collect();
        }
        self.history.push(event);
        self.index += 1;
        self.update_current_image(self.index - 1)
    }

    fn update_current_image(&mut self, event_idx: usize) -> Result<(), IndexedImageError> {
        match &self.history[event_idx] {
            EditEvent::Pixels(pixels, color) => {
                for idx in pixels {
                    self.current_image.set_pixel(*idx, *color)?;
                }
            }
            EditEvent::Palette(colors) => self.current_image.set_palette(colors)?,
        }
        Ok(())
    }

    fn rebuild_current_image(&mut self) -> Result<(), IndexedImageError> {
        self.current_image = self.base_image.clone();
        for i in 0..self.index {
            self.update_current_image(i)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use pixels_graphics_lib::prelude::{IciColor, IndexedImage};
    use crate::ui::edit_history::FrameHistory;

    fn compare_images(lhs: &IndexedImage, rhs: &IndexedImage) {
        if lhs.width() != rhs.width() || lhs.height() != rhs.height() {
            panic!("Images are different sizes: {},{} != {},{}", lhs.width(), lhs.height(), rhs.width(), rhs.height());
        }
        if lhs.get_palette() != rhs.get_palette() {
            panic!("Images have different palettes: {:?} != {:?}", lhs.get_palette(), rhs.get_palette());
        }
        let mut output = String::new();
        let len = lhs.width() as usize * lhs.height() as usize;
        for i in 0..len {
            let lhs = lhs.get_pixel(i).unwrap();
            let rhs = rhs.get_pixel(i).unwrap();
            if lhs != rhs {
                output.push_str(&format!("\n{i} {lhs} != {rhs}"));
            }
        }
        if !output.is_empty() {
            panic!("Pixels are different: {output}");
        }
    }

    #[test]
    fn after_one_fill() {
        let base = IndexedImage::new(4, 4, vec![IciColor::transparent(), IciColor::new(255, 0, 0, 255)], vec![0; 4 * 4]).unwrap();
        let red = IndexedImage::new(4, 4, vec![IciColor::transparent(), IciColor::new(255, 0, 0, 255)], vec![1; 4 * 4]).unwrap();
        let mut history = FrameHistory::new(base.clone());
        compare_images(&base, &history.current_image);
        history.add_fill((0, 0), 1).unwrap();
        compare_images(&red, &history.current_image);
    }

    #[test]
    fn undo_redo() {
        let palette = vec![IciColor::transparent(), IciColor::new(255, 0, 0, 255), IciColor::new(255, 0, 0, 255)];
        let base = IndexedImage::new(4, 4, palette.clone(), vec![0; 4 * 4]).unwrap();
        let red = IndexedImage::new(4, 4, palette.clone(), vec![1; 4 * 4]).unwrap();
        let blue = IndexedImage::new(4, 4, palette.clone(), vec![2; 4 * 4]).unwrap();
        let mut history = FrameHistory::new(base.clone());
        compare_images(&base, &history.current_image);
        history.add_fill((0, 0), 1).unwrap();
        compare_images(&red, &history.current_image);
        history.add_fill((0, 0), 2).unwrap();
        compare_images(&blue, &history.current_image);
        history.undo().unwrap();
        compare_images(&red, &history.current_image);
        history.undo().unwrap();
        compare_images(&base, &history.current_image);
        history.redo().unwrap();
        compare_images(&red, &history.current_image);
    }

    #[test]
    fn undo_event_redo() {
        let palette = vec![IciColor::transparent(), IciColor::new(255, 0, 0, 255), IciColor::new(255, 0, 0, 255)];
        let base = IndexedImage::new(4, 4, palette.clone(), vec![0; 4 * 4]).unwrap();
        let red = IndexedImage::new(4, 4, palette.clone(), vec![1; 4 * 4]).unwrap();
        let blue = IndexedImage::new(4, 4, palette.clone(), vec![2; 4 * 4]).unwrap();
        let mut history = FrameHistory::new(base.clone());
        compare_images(&base, &history.current_image);
        history.add_fill((0, 0), 1).unwrap();
        compare_images(&red, &history.current_image);
        history.undo().unwrap();
        compare_images(&base, &history.current_image);
        history.add_fill((0, 0), 2).unwrap();
        compare_images(&blue, &history.current_image);
        history.redo().unwrap();
        compare_images(&blue, &history.current_image);
    }
}