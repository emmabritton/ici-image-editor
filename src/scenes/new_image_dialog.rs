use crate::scenes::dialog_background;
use crate::ui::prelude::*;
use crate::ui::text_field::TextFilter::Numbers;
use crate::SceneName::Editor;
use crate::SceneUpdateResult::*;
use crate::{Scene, SceneName, SceneResult, SUR};
use pixels_graphics_lib::buffer_graphics_lib::image::Image;
use pixels_graphics_lib::buffer_graphics_lib::image_loading::load_image;
use pixels_graphics_lib::prelude::ImageFormat::Png;
use pixels_graphics_lib::prelude::Positioning::LeftCenter;
use pixels_graphics_lib::prelude::WrappingStrategy::SpaceBeforeCol;
use pixels_graphics_lib::prelude::*;
use std::io::{BufReader, Cursor};
use std::str::FromStr;
use crate::scenes::editor::EditorDetails;

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
    error_icon: Image,
    quick_8: Button,
    quick_12: Button,
    quick_16: Button,
    error_pos: Coord
}

impl NewImageDialog {
    pub fn new(width: usize,height: usize,style: &DialogStyle) -> Box<Self> {
        let background = dialog_background(width, height, style);
        let width_label = Text::new(
            "Width (1..=20)",
            TextPos::px(style.bounds.top_left() + (8, 8)),
            (WHITE, Normal),
        );
        let width_field = TextField::new(style.bounds.top_left() + (8, 18), 6, Normal, None,"", &[Numbers], &style.text_field);
        let height_label = Text::new(
            "Height (1..=20)",
            TextPos::px(style.bounds.top_left() + (8, 40)),
            (WHITE, Normal),
        );
        let height_field = TextField::new(style.bounds.top_left() + (8, 50), 6, Normal,None, "", &[Numbers], &style.text_field);
        let submit_button = Button::new(style.bounds.top_left() + (138, 144), "Create", None, &style.button);
        let cancel_button = Button::new(style.bounds.top_left() + (8, 144), "Cancel", None, &style.button);
        let error_pos = style.bounds.top_left()+(16,  80);
        let error_outline = Polyline::rounded_rect(
            error_pos.x,
            error_pos.y,
            style.bounds.top_left().x + 180,
            style.bounds.top_left().y + 120,
            8,
            RED,
        )
        .unwrap();
        let warning_icon_data = include_bytes!("../../assets/icons/warning.png");
        let cursor = Cursor::new(warning_icon_data);
        let reader = BufReader::new(cursor);
        let warning_icon = load_image(reader, Png).unwrap();
        let quick_8 = Button::new(style.bounds.top_left() + (138, 8), "8x8", Some(50), &style.button);
        let quick_12 = Button::new(style.bounds.top_left() + (138, 30), "12x12", Some(50), &style.button);
        let quick_16 = Button::new(style.bounds.top_left() + (138, 52), "16x16", Some(50), &style.button);
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
            error_icon: warning_icon,
            quick_8,
            quick_12,
            quick_16,
            error_pos
        })
    }
}

impl NewImageDialog {
    fn verify(&self) -> Result<(usize, usize), String> {
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
                let width = width.unwrap() as usize;
                let height = height.unwrap() as usize;
                Ok((width, height))
            }
        }
    }

    fn set_success(&mut self, width: usize, height: usize) {
        self.result = Push(
            true,
            Editor(EditorDetails::New(width, height)),
        );
    }
}

impl Scene<SceneResult, SceneName> for NewImageDialog {
    fn render(&self, graphics: &mut Graphics, mouse_xy: Coord) {
        graphics.draw(&self.background);
        graphics.draw(&self.width_label);
        graphics.draw(&self.height_label);
        self.submit_button.render(graphics, mouse_xy);
        self.cancel_button.render(graphics, mouse_xy);
        self.quick_8.render(graphics, mouse_xy);
        self.quick_12.render(graphics, mouse_xy);
        self.quick_16.render(graphics, mouse_xy);
        self.width_field.render(graphics, mouse_xy);
        self.height_field.render(graphics, mouse_xy);

        if let Some(text) = self.error_message.as_ref() {
            self.error_outline.render(graphics);
            graphics.draw_image(self.error_pos + (6, 12), &self.error_icon);
            graphics.draw_text(
                text,
                TextPos::px(self.error_pos + (26, 20)),
                (RED, Normal, SpaceBeforeCol(18), LeftCenter),
            )
        }
    }

    fn on_key_press(&mut self, key: VirtualKeyCode, _: &Vec<&VirtualKeyCode>) {
        if key == VirtualKeyCode::Tab && self.width_field.is_focused() {
            self.width_field.unfocus();
            self.height_field.focus();
        }
        self.width_field.on_key_press(key);
        self.height_field.on_key_press(key);
    }

    fn on_mouse_click(&mut self, xy: Coord, _: &Vec<&VirtualKeyCode>) {
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
