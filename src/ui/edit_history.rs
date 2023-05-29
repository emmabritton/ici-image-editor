use crate::ui::image_fill::fill_pixels;
use fnv::FnvHashSet;
#[cfg(not(test))]
use log::debug;
use pixels_graphics_lib::prelude::{
    IciColor, IndexedImage, IndexedImageError, Line, Shape, ToColor,
};
use std::mem::swap;
#[cfg(test)]
use std::println as debug;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum EditEvent {
    PixelsChange {
        pixel_idxs: Vec<usize>,
        color_idx: u8,
    },
    FrameAdd {
        idx: usize,
        content: Vec<u8>,
    },
    FrameRemove(usize),
    FrameSelect(usize),
    PaletteChange(Vec<IciColor>),
}

#[derive(Debug)]
pub struct EditHistory {
    base_images: Vec<IndexedImage>,
    edited_images: Vec<IndexedImage>,
    current_image: IndexedImage,
    events: Vec<EditEvent>,
    /// current position in events, should be events.len() unless undo is used
    index: usize,
    active_frame: usize,
}

impl EditHistory {
    pub fn new(base_images: Vec<IndexedImage>) -> Self {
        let current_image = base_images[0].clone();
        let edited_images = base_images.clone();
        Self {
            base_images,
            current_image,
            edited_images,
            events: vec![],
            index: 0,
            active_frame: 0,
        }
    }
}

impl EditHistory {
    pub fn undo(&mut self) -> Result<(), IndexedImageError> {
        if self.index >= 1 {
            self.index -= 1;
            return self.rebuild_current_image();
        }
        Ok(())
    }

    pub fn redo(&mut self) -> Result<(), IndexedImageError> {
        if self.index < self.events.len() {
            let event = self.events[self.index].clone();
            self.index += 1;
            debug!(
                "Redoing {event:?}, index: {}, total: {}",
                self.index,
                self.events.len()
            );
            return self.handle_edit_event(&event);
        }
        Ok(())
    }

    pub fn get_current_image(&self) -> &IndexedImage {
        &self.current_image
    }

    pub fn get_image(&self, idx: usize) -> &IndexedImage {
        &self.edited_images[idx]
    }

    pub fn get_images(&self) -> Vec<IndexedImage> {
        self.edited_images.clone()
    }

    pub fn active_frame(&self) -> usize {
        self.active_frame
    }

    pub fn frame_count(&self) -> usize {
        self.edited_images.len()
    }

    pub fn is_first_event_light_pixel(&self) -> bool {
        if self.events.len() == 1 {
            if let EditEvent::PixelsChange {
                pixel_idxs,
                color_idx,
            } = &self.events[0]
            {
                let color = self.current_image.get_color(*color_idx).unwrap();
                if pixel_idxs.len() == 1 && color.to_color().brightness() > 0.95 {
                    return true;
                }
            }
        }
        false
    }
}

impl EditHistory {
    pub fn add_line(
        &mut self,
        start: (u8, u8),
        end: (u8, u8),
        color: u8,
    ) -> Result<(), IndexedImageError> {
        let points = Line::new(start, end).outline_pixels();
        let mut pixels = vec![];

        for point in points {
            let i = self
                .current_image
                .get_pixel_index(point.x as u8, point.y as u8)?;
            pixels.push(i);
        }

        let event = EditEvent::PixelsChange {
            pixel_idxs: pixels,
            color_idx: color,
        };
        self.add_event(event)
    }

    pub fn add_rect(
        &mut self,
        start: (u8, u8),
        end: (u8, u8),
        color: u8,
    ) -> Result<(), IndexedImageError> {
        let mut pixels = FnvHashSet::default();
        let top_left = ((start.0).min(end.0), (start.1).min(end.1));
        let bottom_right = ((start.0).max(end.0), (start.1).max(end.1));

        for x in top_left.0..bottom_right.0 {
            let i = self.current_image.get_pixel_index(x, top_left.1)?;
            pixels.insert(i);
            let i = self.current_image.get_pixel_index(x, bottom_right.1)?;
            pixels.insert(i);
        }

        for y in top_left.1..=bottom_right.1 {
            let i = self.current_image.get_pixel_index(top_left.0, y)?;
            pixels.insert(i);
            let i = self.current_image.get_pixel_index(bottom_right.0, y)?;
            pixels.insert(i);
        }

        let event = EditEvent::PixelsChange {
            pixel_idxs: pixels.into_iter().collect(),
            color_idx: color,
        };
        self.add_event(event)
    }

    pub fn add_fill(&mut self, xy: (u8, u8), color: u8) -> Result<(), IndexedImageError> {
        let pixels = fill_pixels(&self.current_image, xy)?;
        let event = EditEvent::PixelsChange {
            pixel_idxs: pixels,
            color_idx: color,
        };
        self.add_event(event)
    }

    pub fn add_pencil(&mut self, xy: (u8, u8), color: u8) -> Result<(), IndexedImageError> {
        let i = self.current_image.get_pixel_index(xy.0, xy.1)?;
        if self.current_image.get_pixel(i).unwrap() != color {
            let event = EditEvent::PixelsChange {
                pixel_idxs: vec![i],
                color_idx: color,
            };
            self.add_event(event)
        } else {
            Ok(())
        }
    }

    pub fn add_clear(&mut self) -> Result<(), IndexedImageError> {
        let size = self.current_image.width() as usize * self.current_image.height() as usize;
        let event = EditEvent::PixelsChange {
            pixel_idxs: (0..size).collect(),
            color_idx: 0,
        };
        self.add_event(event)
    }

    pub fn add_palette_change(&mut self, colors: &[IciColor]) -> Result<(), IndexedImageError> {
        if self.current_image.get_palette() == colors {
            return Ok(());
        }
        let event = EditEvent::PaletteChange(colors.to_vec());
        self.add_event(event)
    }

    pub fn add_blank_frame(&mut self) -> Result<(), IndexedImageError> {
        self.add_event(EditEvent::FrameAdd {
            idx: self.active_frame,
            content: vec![
                0;
                self.current_image.width() as usize * self.current_image.height() as usize
            ],
        })
    }

    pub fn add_duplicate_frame(&mut self) -> Result<(), IndexedImageError> {
        let pixels = self.current_image.get_pixels().to_vec();
        self.add_event(EditEvent::FrameAdd {
            idx: self.active_frame,
            content: pixels,
        })
    }

    pub fn remove_frame(&mut self) -> Result<(), IndexedImageError> {
        self.add_event(EditEvent::FrameRemove(self.active_frame))
    }

    pub fn add_frame_select(&mut self, idx: usize) -> Result<(), IndexedImageError> {
        if self.active_frame == idx {
            return Ok(());
        }
        self.add_event(EditEvent::FrameSelect(idx))
    }
}

impl EditHistory {
    fn add_event(&mut self, event: EditEvent) -> Result<(), IndexedImageError> {
        debug!("Adding {event:?}");
        if (self.index as isize) < self.events.len() as isize {
            debug!("Index before end, rewriting");
            let mut temp = vec![];
            swap(&mut self.events, &mut temp);
            self.events = temp.into_iter().take(self.index).collect();
        }
        self.edited_images[self.active_frame] = self.current_image.clone();
        self.handle_edit_event(&event)?;
        self.events.push(event.clone());
        self.condense_pencil_events();
        self.index += 1;
        Ok(())
    }

    fn condense_pencil_events(&mut self) {
        if self.events.len() >= 5 {
            let last_five_events = self.events.iter().rev().take(5);
            let events: Vec<(usize, u8)> = last_five_events
                .filter_map(|ev| {
                    if let EditEvent::PixelsChange {
                        pixel_idxs,
                        color_idx,
                    } = ev
                    {
                        if pixel_idxs.len() == 1 {
                            Some((pixel_idxs[0], *color_idx))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();
            //if the last five events are single pixels and all use the same color
            if events.len() == 5 && events.iter().all(|ev| ev.1 == events[0].1) {
                //then combine into one
                let pixels = events.iter().map(|ev| ev.0).collect();
                let event = EditEvent::PixelsChange {
                    pixel_idxs: pixels,
                    color_idx: events[0].1,
                };

                let mut new_events: Vec<EditEvent> = self
                    .events
                    .iter()
                    .take(self.events.len() - 5)
                    .cloned()
                    .collect();
                debug!(
                    "Condensed the last 5 pixel edits to {event:?}, history is now {} long",
                    new_events.len() + 1
                );
                new_events.push(event);
                self.index -= 4;
                self.events = new_events;
            }
        }
    }

    fn handle_edit_event(&mut self, event: &EditEvent) -> Result<(), IndexedImageError> {
        match event {
            EditEvent::PixelsChange {
                pixel_idxs,
                color_idx,
            } => {
                for idx in pixel_idxs {
                    self.current_image.set_pixel(*idx, *color_idx)?;
                }
            }
            EditEvent::PaletteChange(colors) => {
                for image in &mut self.edited_images {
                    image.set_palette(colors)?;
                }
                self.current_image = self.edited_images[self.active_frame].clone();
            }
            EditEvent::FrameAdd { idx, content } => {
                self.active_frame = idx + 1;
                let image = IndexedImage::new(
                    self.current_image.width(),
                    self.current_image.height(),
                    self.current_image.get_palette().to_vec(),
                    content.clone(),
                )?;
                self.edited_images.insert(self.active_frame, image.clone());
                self.current_image = image;
            }
            EditEvent::FrameRemove(idx) => {
                self.edited_images.remove(*idx);
                if self.active_frame >= self.edited_images.len() {
                    self.active_frame = self.edited_images.len() - 1;
                }
                self.current_image = self.edited_images[self.active_frame].clone();
            }
            EditEvent::FrameSelect(idx) => {
                self.current_image = self.edited_images[*idx].clone();
                self.active_frame = *idx;
            }
        }
        Ok(())
    }

    fn rebuild_current_image(&mut self) -> Result<(), IndexedImageError> {
        debug!("Rebuilding image");
        self.edited_images = self.base_images.clone();
        self.current_image = self.base_images[0].clone();
        self.active_frame = 0;
        debug!(
            "Replaying {} events, history has {} in total",
            self.index,
            self.events.len()
        );
        let events: Vec<EditEvent> = self.events.iter().take(self.index).cloned().collect();
        for event in &events {
            debug!("Replaying {event:?}");
            self.handle_edit_event(event)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ui::edit_history::EditEvent::*;
    use pixels_graphics_lib::prelude::*;

    fn q_pc(idx: usize, color: u8) -> EditEvent {
        PixelsChange {
            pixel_idxs: vec![idx],
            color_idx: color,
        }
    }

    fn q_mpc(idx: &[usize], color: u8) -> EditEvent {
        PixelsChange {
            pixel_idxs: idx.to_vec(),
            color_idx: color,
        }
    }

    fn q_ai(idx: usize, image: &IndexedImage) -> EditEvent {
        FrameAdd {
            idx,
            content: image.get_pixels().to_vec(),
        }
    }

    #[test]
    fn init_state() {
        let original_image = IndexedImage::new(
            3,
            3,
            vec![TRANSPARENT.to_ici(), BLUE.to_ici()],
            vec![0; 3 * 3],
        )
        .unwrap();
        let history = EditHistory::new(vec![original_image.clone()]);
        assert_eq!(history.active_frame, 0);
        assert_eq!(history.events, vec![]);
        assert_eq!(history.base_images, history.edited_images);
        assert_eq!(history.base_images[0], history.current_image);
        assert_eq!(original_image, history.current_image);
        assert_eq!(history.index, 0);
    }

    #[test]
    fn condensing() {
        let original_image = IndexedImage::new(
            3,
            3,
            vec![TRANSPARENT.to_ici(), BLUE.to_ici()],
            vec![0; 3 * 3],
        )
        .unwrap();
        let mut history = EditHistory::new(vec![original_image]);
        history.add_pencil((0, 0), 1).unwrap();
        assert_eq!(history.events, vec![q_pc(0, 1)]);
        assert_eq!(history.index, 1);
        history.add_pencil((1, 0), 1).unwrap();
        assert_eq!(history.events, vec![q_pc(0, 1), q_pc(1, 1)]);
        assert_eq!(history.index, 2);
        history.add_pencil((2, 0), 1).unwrap();
        assert_eq!(history.events, vec![q_pc(0, 1), q_pc(1, 1), q_pc(2, 1)]);
        assert_eq!(history.index, 3);
        history.add_pencil((0, 1), 1).unwrap();
        assert_eq!(
            history.events,
            vec![q_pc(0, 1), q_pc(1, 1), q_pc(2, 1), q_pc(3, 1)]
        );
        assert_eq!(history.index, 4);
        history.add_pencil((1, 1), 1).unwrap();
        assert_eq!(history.events, vec![q_mpc(&[4, 3, 2, 1, 0], 1)]);
        assert_eq!(history.index, 1);
        history.add_pencil((2, 1), 1).unwrap();
        assert_eq!(history.events, vec![q_mpc(&[4, 3, 2, 1, 0], 1), q_pc(5, 1)]);
        assert_eq!(history.index, 2);
    }

    #[test]
    fn undo_redo_single_frame() {
        let original_image = IndexedImage::new(
            3,
            3,
            vec![TRANSPARENT.to_ici(), BLUE.to_ici()],
            vec![0; 3 * 3],
        )
        .unwrap();
        let mut history = EditHistory::new(vec![original_image]);
        assert_eq!(history.get_current_image().get_pixel(0).unwrap(), 0);
        assert_eq!(history.get_current_image().get_pixel(4).unwrap(), 0);
        history.add_pencil((0, 0), 1).unwrap();
        history.add_pencil((1, 1), 1).unwrap();
        assert_eq!(history.events, vec![q_pc(0, 1), q_pc(4, 1)]);
        assert_eq!(history.index, 2);
        assert_eq!(history.get_current_image().get_pixel(0).unwrap(), 1);
        assert_eq!(history.get_current_image().get_pixel(4).unwrap(), 1);
        history.undo().unwrap();
        assert_eq!(history.events, vec![q_pc(0, 1), q_pc(4, 1)]);
        assert_eq!(history.index, 1);
        assert_eq!(history.get_current_image().get_pixel(0).unwrap(), 1);
        assert_eq!(history.get_current_image().get_pixel(4).unwrap(), 0);
        history.redo().unwrap();
        assert_eq!(history.events, vec![q_pc(0, 1), q_pc(4, 1)]);
        assert_eq!(history.index, 2);
        assert_eq!(history.get_current_image().get_pixel(0).unwrap(), 1);
        assert_eq!(history.get_current_image().get_pixel(4).unwrap(), 1);
    }

    #[test]
    fn remove_first_of_three_frames() {
        let palette = vec![
            TRANSPARENT.to_ici(),
            BLUE.to_ici(),
            RED.to_ici(),
            GREEN.to_ici(),
        ];
        let image1 = IndexedImage::new(3, 3, palette.clone(), vec![1; 9]).unwrap();
        let image2 = IndexedImage::new(3, 3, palette.clone(), vec![2; 9]).unwrap();
        let image3 = IndexedImage::new(3, 3, palette, vec![3; 9]).unwrap();
        let mut history = EditHistory::new(vec![image1.clone(), image2.clone(), image3.clone()]);
        assert_eq!(history.active_frame, 0);
        assert_eq!(
            history.edited_images,
            vec![image1.clone(), image2.clone(), image3.clone()]
        );
        assert_eq!(
            history.base_images,
            vec![image1.clone(), image2.clone(), image3.clone()]
        );
        assert_eq!(history.events, vec![]);
        assert_eq!(history.index, 0);
        history.remove_frame().unwrap();
        assert_eq!(history.active_frame, 0);
        assert_eq!(history.edited_images, vec![image2.clone(), image3.clone()]);
        assert_eq!(history.base_images, vec![image1, image2, image3]);
        assert_eq!(history.events, vec![FrameRemove(0)]);
        assert_eq!(history.index, 1);
    }

    #[test]
    fn remove_second_of_three_frames() {
        let palette = vec![
            TRANSPARENT.to_ici(),
            BLUE.to_ici(),
            RED.to_ici(),
            GREEN.to_ici(),
        ];
        let image1 = IndexedImage::new(3, 3, palette.clone(), vec![1; 9]).unwrap();
        let image2 = IndexedImage::new(3, 3, palette.clone(), vec![2; 9]).unwrap();
        let image3 = IndexedImage::new(3, 3, palette, vec![3; 9]).unwrap();
        let mut history = EditHistory::new(vec![image1.clone(), image2.clone(), image3.clone()]);
        assert_eq!(history.active_frame, 0);
        assert_eq!(
            history.edited_images,
            vec![image1.clone(), image2.clone(), image3.clone()]
        );
        assert_eq!(
            history.base_images,
            vec![image1.clone(), image2.clone(), image3.clone()]
        );
        assert_eq!(history.events, vec![]);
        assert_eq!(history.index, 0);
        history.add_frame_select(1).unwrap();
        assert_eq!(history.active_frame, 1);
        assert_eq!(
            history.edited_images,
            vec![image1.clone(), image2.clone(), image3.clone()]
        );
        assert_eq!(
            history.base_images,
            vec![image1.clone(), image2.clone(), image3.clone()]
        );
        assert_eq!(history.events, vec![FrameSelect(1)]);
        assert_eq!(history.index, 1);
        history.remove_frame().unwrap();
        assert_eq!(history.active_frame, 1);
        assert_eq!(history.edited_images, vec![image1.clone(), image3.clone()]);
        assert_eq!(history.base_images, vec![image1, image2, image3]);
        assert_eq!(history.events, vec![FrameSelect(1), FrameRemove(1)]);
        assert_eq!(history.index, 2);
    }

    #[test]
    fn remove_third_of_three_frames() {
        let palette = vec![
            TRANSPARENT.to_ici(),
            BLUE.to_ici(),
            RED.to_ici(),
            GREEN.to_ici(),
        ];
        let image1 = IndexedImage::new(3, 3, palette.clone(), vec![1; 9]).unwrap();
        let image2 = IndexedImage::new(3, 3, palette.clone(), vec![2; 9]).unwrap();
        let image3 = IndexedImage::new(3, 3, palette, vec![3; 9]).unwrap();
        let mut history = EditHistory::new(vec![image1.clone(), image2.clone(), image3.clone()]);
        assert_eq!(history.active_frame, 0);
        assert_eq!(
            history.edited_images,
            vec![image1.clone(), image2.clone(), image3.clone()]
        );
        assert_eq!(
            history.base_images,
            vec![image1.clone(), image2.clone(), image3.clone()]
        );
        assert_eq!(history.events, vec![]);
        assert_eq!(history.index, 0);
        history.add_frame_select(2).unwrap();
        assert_eq!(history.active_frame, 2);
        assert_eq!(
            history.edited_images,
            vec![image1.clone(), image2.clone(), image3.clone()]
        );
        assert_eq!(
            history.base_images,
            vec![image1.clone(), image2.clone(), image3.clone()]
        );
        assert_eq!(history.events, vec![FrameSelect(2)]);
        assert_eq!(history.index, 1);
        history.remove_frame().unwrap();
        assert_eq!(history.active_frame, 1);
        assert_eq!(history.edited_images, vec![image1.clone(), image2.clone()]);
        assert_eq!(history.base_images, vec![image1, image2, image3]);
        assert_eq!(history.events, vec![FrameSelect(2), FrameRemove(2)]);
        assert_eq!(history.index, 2);
    }

    #[test]
    fn add_blank_frame() {
        let palette = vec![
            TRANSPARENT.to_ici(),
            BLUE.to_ici(),
            RED.to_ici(),
            GREEN.to_ici(),
        ];
        let image1 = IndexedImage::new(3, 3, palette.clone(), vec![1; 9]).unwrap();
        let image2 = IndexedImage::new(3, 3, palette, vec![0; 9]).unwrap();
        let mut history = EditHistory::new(vec![image1.clone()]);
        history.add_blank_frame().unwrap();
        assert_eq!(history.active_frame, 1);
        assert_eq!(history.edited_images, vec![image1.clone(), image2.clone()]);
        assert_eq!(history.base_images, vec![image1]);
        assert_eq!(history.events, vec![q_ai(0, &image2)]);
        assert_eq!(history.index, 1);
        assert_eq!(history.current_image, image2)
    }

    #[test]
    fn add_duplicate_frame() {
        let palette = vec![
            TRANSPARENT.to_ici(),
            BLUE.to_ici(),
            RED.to_ici(),
            GREEN.to_ici(),
        ];
        let image1 = IndexedImage::new(3, 3, palette, vec![1; 9]).unwrap();
        let mut history = EditHistory::new(vec![image1.clone()]);
        history.add_duplicate_frame().unwrap();
        assert_eq!(history.active_frame, 1);
        assert_eq!(history.edited_images, vec![image1.clone(), image1.clone()]);
        assert_eq!(history.base_images, vec![image1.clone()]);
        assert_eq!(history.events, vec![q_ai(0, &image1)]);
        assert_eq!(history.index, 1);
        assert_eq!(history.current_image, image1)
    }

    #[test]
    fn palette_swap() {
        let orig_palette = vec![TRANSPARENT.to_ici(), BLUE.to_ici()];
        let new_palette = vec![TRANSPARENT.to_ici(), RED.to_ici()];
        let image1 = IndexedImage::new(3, 3, orig_palette, vec![1; 9]).unwrap();
        let mut history = EditHistory::new(vec![image1.clone(), image1]);
        assert_eq!(history.current_image.get_pixel(0).unwrap(), 1);
        assert_eq!(history.current_image.get_color(1).unwrap(), BLUE.to_ici());
        assert_eq!(
            history.edited_images[0].get_color(1).unwrap(),
            BLUE.to_ici()
        );
        assert_eq!(
            history.edited_images[1].get_color(1).unwrap(),
            BLUE.to_ici()
        );
        history.add_palette_change(&new_palette).unwrap();
        assert_eq!(history.current_image.get_pixel(0).unwrap(), 1);
        assert_eq!(history.current_image.get_color(1).unwrap(), RED.to_ici());
        assert_eq!(history.edited_images[0].get_color(1).unwrap(), RED.to_ici());
        assert_eq!(history.edited_images[1].get_color(1).unwrap(), RED.to_ici());
    }
}
