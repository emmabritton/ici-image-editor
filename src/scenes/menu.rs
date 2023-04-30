use crate::scenes::new_editor::EditorDetails;
use crate::scenes::BACKGROUND;
use crate::SceneName::Editor;
use crate::SceneUpdateResult::{Nothing, Push};
use crate::{Scene, SceneName, SceneResult, SUR};
use pixels_graphics_lib::buffer_graphics_lib::image::Image;
use pixels_graphics_lib::buffer_graphics_lib::prelude::*;
use pixels_graphics_lib::buffer_graphics_lib::text::wrapping::WrappingStrategy;
use pixels_graphics_lib::buffer_graphics_lib::text::TextSize::Large;
use pixels_graphics_lib::buffer_graphics_lib::Graphics;
use pixels_graphics_lib::prelude::button::Button;
use pixels_graphics_lib::prelude::*;
use pixels_graphics_lib::ui::styles::ButtonStyle;
use pixels_graphics_lib::Timing;

const LOGO_POS: Coord = Coord::new(10, 10);
const NEW_POS: Coord = Coord::new(10, 50);
const LOAD_POS: Coord = Coord::new(10, 70);

#[derive(Debug)]
pub struct Menu {
    result: SUR,
    logo: Image,
    new_button: Button,
    load_button: Button,
}

fn make_image(
    width: usize,
    height: usize,
    method: fn(&mut Graphics),
) -> Result<Image, GraphicsError> {
    let mut buffer = vec![0_u8; width * height * 4];
    let mut graphics = Graphics::new(&mut buffer, width, height)?;
    method(&mut graphics);
    Ok(graphics.copy_to_image())
}

impl Menu {
    pub fn new(button_style: &ButtonStyle) -> Box<Self> {
        let logo = make_image(60, 40, |graphics| {
            graphics.draw_text(
                "ici Image Editor",
                Px(0, 0),
                (WHITE, Large, WrappingStrategy::SpaceBeforeCol(7)),
            );
        })
        .unwrap();
        let new_button = Button::new(NEW_POS, "New image", Some(86), button_style);
        let load_button = Button::new(LOAD_POS, "Load image", Some(86), button_style);
        Box::new(Self {
            result: Nothing,
            logo,
            new_button,
            load_button,
        })
    }
}

impl Scene<SceneResult, SceneName> for Menu {
    fn render(&self, graphics: &mut Graphics, mouse_xy: Coord) {
        graphics.clear(BACKGROUND);

        graphics.draw_image(LOGO_POS, &self.logo);

        self.new_button.render(graphics, mouse_xy);
        self.load_button.render(graphics, mouse_xy);
    }

    fn on_key_up(&mut self, _: VirtualKeyCode, _: &Vec<&VirtualKeyCode>) {}

    fn on_mouse_up(&mut self, xy: Coord, button: MouseButton, _: &Vec<&VirtualKeyCode>) {
        if button != MouseButton::Left {
            return;
        }
        if self.new_button.on_mouse_click(xy) {
            self.result = Push(false, SceneName::NewImage);
        }
        if self.load_button.on_mouse_click(xy) {
            self.result = Push(false, SceneName::LoadFile(String::from("ici")));
        }
    }

    fn update(&mut self, _: &Timing, _: Coord, _: &Vec<&VirtualKeyCode>) -> SUR {
        self.result.clone()
    }

    fn resuming(&mut self, result: Option<SceneResult>) {
        if let Some(result) = result {
            self.result = match result {
                SceneResult::LoadFilePath(path) => Push(false, Editor(EditorDetails::Open(path))),
                _ => Nothing,
            }
        } else {
            self.result = Nothing;
        }
    }

    fn is_dialog(&self) -> bool {
        false
    }
}
