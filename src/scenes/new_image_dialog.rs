use crate::scenes::dialog_background;
use crate::scenes::new_editor::EditorDetails;
use crate::SceneName::Editor;
use crate::SceneUpdateResult::*;
use crate::{Scene, SceneName, SceneResult, SUR};
use pixels_graphics_lib::prelude::Positioning::LeftCenter;
use pixels_graphics_lib::prelude::WrappingStrategy::SpaceBeforeCol;
use pixels_graphics_lib::prelude::*;
use pixels_graphics_lib::ui::prelude::TextFilter::*;
use pixels_graphics_lib::ui::prelude::*;
use std::str::FromStr;

#[derive(Debug)]
pub struct NewImageDialog {
    result: SUR,
    width_field: TextField,
    width_label: Text,
    height_field: TextField,
    height_label: Text,
    submit_button: Button,
    cancel_button: Button,
    background: ShapeCollection,
    error_outline: Polyline,
    error_message: Option<String>,
    quick_8: Button,
    quick_12: Button,
    quick_16: Button,
    quick_24: Button,
    quick_32: Button,
    quick_48: Button,
    quick_64: Button,
    quick_8_16: Button,
    error_pos: Coord,
    label: Text,
}

impl NewImageDialog {
    pub fn new(width: usize, height: usize, style: &DialogStyle) -> Box<Self> {
        let background = dialog_background(width, height, style);
        let width_label = Text::new(
            "Width (1..=64)",
            TextPos::px(style.bounds.top_left() + (8, 8)),
            (WHITE, Normal),
        );
        let width_field = TextField::new(
            style.bounds.top_left() + (8, 18),
            6,
            Normal,
            (None, None),
            "",
            &[Numbers],
            &style.text_field,
        );
        let height_label = Text::new(
            "Height (1..=64)",
            TextPos::px(style.bounds.top_left() + (8, 40)),
            (WHITE, Normal),
        );
        let height_field = TextField::new(
            style.bounds.top_left() + (8, 50),
            6,
            Normal,
            (None, None),
            "",
            &[Numbers],
            &style.text_field,
        );
        let submit_button = Button::new(
            style.bounds.top_left() + (138, 144),
            "Create",
            None,
            &style.button,
        );
        let cancel_button = Button::new(
            style.bounds.top_left() + (8, 144),
            "Cancel",
            None,
            &style.button,
        );
        let error_pos = style.bounds.top_left() + (16, 80);
        let error_outline = Polyline::rounded_rect(
            error_pos.x,
            error_pos.y,
            style.bounds.top_left().x + 180,
            style.bounds.top_left().y + 120,
            8,
            RED,
        )
        .unwrap();
        let row1_y = 80;
        let row2_y = 100;
        let row3_y = 120;
        let quick_8 = Button::new(
            style.bounds.top_left() + (8, row1_y),
            "8x8",
            Some(50),
            &style.button,
        );
        let quick_12 = Button::new(
            (
                quick_8.bounds().bottom_right().x + 8,
                style.bounds.top_left().y + row1_y,
            ),
            "12x12",
            Some(50),
            &style.button,
        );
        let quick_16 = Button::new(
            (
                quick_12.bounds().bottom_right().x + 8,
                style.bounds.top_left().y + row1_y,
            ),
            "16x16",
            Some(50),
            &style.button,
        );
        let quick_24 = Button::new(
            style.bounds.top_left() + (8, row2_y),
            "24x24",
            Some(50),
            &style.button,
        );
        let quick_32 = Button::new(
            (
                quick_24.bounds().bottom_right().x + 8,
                style.bounds.top_left().y + row2_y,
            ),
            "32x32",
            Some(50),
            &style.button,
        );
        let quick_48 = Button::new(
            (
                quick_32.bounds().bottom_right().x + 8,
                style.bounds.top_left().y + row2_y,
            ),
            "48x48",
            Some(50),
            &style.button,
        );
        let quick_64 = Button::new(
            style.bounds.top_left() + (8, row3_y),
            "64x64",
            Some(50),
            &style.button,
        );
        let quick_8_16 = Button::new(
            (
                quick_64.bounds().bottom_right().x + 8,
                style.bounds.top_left().y + row3_y,
            ),
            "8x16",
            Some(50),
            &style.button,
        );
        let label = Text::new(
            "Quick create",
            TextPos::px(style.bounds.top_left() + (8, 68)),
            (WHITE, Normal),
        );
        Box::new(Self {
            result: Nothing,
            width_field,
            width_label,
            height_field,
            height_label,
            submit_button,
            cancel_button,
            background,
            error_outline,
            error_message: None,
            quick_8,
            quick_12,
            quick_16,
            quick_24,
            quick_32,
            quick_48,
            quick_64,
            quick_8_16,
            error_pos,
            label,
        })
    }
}

#[allow(clippy::unnecessary_unwrap)] //for readability, it is necessary
impl NewImageDialog {
    fn verify(&self) -> Result<(u8, u8), String> {
        if self.width_field.content().is_empty() {
            Err(String::from("Width must be provided"))
        } else if self.height_field.content().is_empty() {
            Err(String::from("Height must be provided"))
        } else {
            let width = u8::from_str(self.width_field.content());
            let height = u8::from_str(self.height_field.content());
            if width.is_err() {
                Err(String::from("Width is invalid"))
            } else if height.is_err() {
                Err(String::from("Height is invalid"))
            } else {
                let width = width.unwrap();
                let height = height.unwrap();
                // if width > 64 || height > 64 {
                //     Err(String::from("Too big, max 64"))
                // } else {
                Ok((width, height))
                // }
            }
        }
    }

    fn set_success(&mut self, width: u8, height: u8) {
        self.result = Push(true, Editor(EditorDetails::New(width, height)));
    }
}

impl Scene<SceneResult, SceneName> for NewImageDialog {
    fn render(&self, graphics: &mut Graphics, mouse_xy: Coord) {
        graphics.draw(&self.background);
        graphics.draw(&self.width_label);
        graphics.draw(&self.height_label);
        self.submit_button.render(graphics, mouse_xy);
        self.cancel_button.render(graphics, mouse_xy);
        self.label.render(graphics);
        self.quick_8.render(graphics, mouse_xy);
        self.quick_12.render(graphics, mouse_xy);
        self.quick_16.render(graphics, mouse_xy);
        self.quick_32.render(graphics, mouse_xy);
        self.quick_24.render(graphics, mouse_xy);
        self.quick_48.render(graphics, mouse_xy);
        self.quick_64.render(graphics, mouse_xy);
        self.quick_8_16.render(graphics, mouse_xy);
        self.width_field.render(graphics, mouse_xy);
        self.height_field.render(graphics, mouse_xy);

        if let Some(text) = self.error_message.as_ref() {
            self.error_outline.render(graphics);
            // graphics.draw_image(self.error_pos + (6, 12), &self.error_icon);
            graphics.draw_text(
                text,
                TextPos::px(self.error_pos + (26, 20)),
                (RED, Normal, SpaceBeforeCol(18), LeftCenter),
            )
        }
    }

    fn on_key_up(&mut self, key: VirtualKeyCode, _: Coord, _: &Vec<&VirtualKeyCode>) {
        if key == VirtualKeyCode::Tab && self.width_field.is_focused() {
            self.width_field.unfocus();
            self.height_field.focus();
        }
        self.width_field.on_key_press(key);
        self.height_field.on_key_press(key);
    }

    fn on_mouse_up(&mut self, xy: Coord, button: MouseButton, _: &Vec<&VirtualKeyCode>) {
        if button != MouseButton::Left {
            return;
        }
        self.width_field.on_mouse_click(xy);
        self.height_field.on_mouse_click(xy);
        if self.submit_button.on_mouse_click(xy) {
            match self.verify() {
                Ok((w, h)) => self.set_success(w, h),
                Err(err) => self.error_message = Some(err),
            }
        }
        if self.cancel_button.on_mouse_click(xy) {
            self.result = Pop(None);
        }
        if self.quick_8.on_mouse_click(xy) {
            self.set_success(8, 8);
        }
        if self.quick_12.on_mouse_click(xy) {
            self.set_success(12, 12);
        }
        if self.quick_16.on_mouse_click(xy) {
            self.set_success(16, 16);
        }
        if self.quick_24.on_mouse_click(xy) {
            self.set_success(24, 24);
        }
        if self.quick_32.on_mouse_click(xy) {
            self.set_success(32, 32);
        }
        if self.quick_48.on_mouse_click(xy) {
            self.set_success(48, 48);
        }
        if self.quick_64.on_mouse_click(xy) {
            self.set_success(64, 64);
        }
        if self.quick_8_16.on_mouse_click(xy) {
            self.set_success(8, 16);
        }
    }

    fn update(&mut self, timing: &Timing, _: Coord, _: &Vec<&VirtualKeyCode>) -> SUR {
        self.width_field.update(timing);
        self.height_field.update(timing);
        self.result.clone()
    }

    fn resuming(&mut self, _: Option<SceneResult>) {}

    fn is_dialog(&self) -> bool {
        true
    }
}
