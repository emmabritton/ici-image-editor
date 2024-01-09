use crate::scenes::new_editor::EditorDetails;
use crate::scenes::{file_dialog, BACKGROUND};
use crate::SceneUpdateResult::{Nothing, Push};
use crate::{Scene, SceneName, SceneResult, Settings, SUR};
use color_eyre::Result;
use pixels_graphics_lib::prelude::TextSize::Large;
use pixels_graphics_lib::prelude::*;
use pixels_graphics_lib::ui::prelude::*;

const LOGO_POS: Coord = Coord::new(10, 10);
const NEW_POS: Coord = Coord::new(10, 50);
const LOAD_POS: Coord = Coord::new(10, 70);

pub struct Menu {
    result: SUR,
    logo: Image,
    new_button: Button,
    load_button: Button,
    prefs: AppPrefs<Settings>,
}

fn make_image(width: usize, height: usize, method: fn(&mut Graphics)) -> Result<Image> {
    let mut buffer = vec![0_u8; width * height * 4];
    let mut graphics = Graphics::new(&mut buffer, width, height)?;
    method(&mut graphics);
    Ok(graphics.copy_to_image())
}

impl Menu {
    pub fn new(prefs: AppPrefs<Settings>, button_style: &ButtonStyle) -> Box<Self> {
        let logo = make_image(60, 40, |graphics| {
            graphics.draw_text(
                "ici Image Editor",
                TextPos::Px(0, 0),
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
            prefs,
        })
    }
}

impl Scene<SceneResult, SceneName> for Menu {
    fn render(&self, graphics: &mut Graphics, mouse: &MouseData, _: &[KeyCode]) {
        graphics.clear(BACKGROUND);

        graphics.draw_image(LOGO_POS, &self.logo);

        self.new_button.render(graphics, mouse);
        self.load_button.render(graphics, mouse);
    }

    fn on_key_up(&mut self, _: KeyCode, _: &MouseData, _: &[KeyCode]) {}

    fn on_mouse_click(
        &mut self,
        down_at: Coord,
        mouse: &MouseData,
        button: MouseButton,
        _: &[KeyCode],
    ) {
        if button != MouseButton::Left {
            return;
        }
        if self.new_button.on_mouse_click(down_at, mouse.xy) {
            self.result = Push(false, SceneName::NewImage);
        }
        if self.load_button.on_mouse_click(down_at, mouse.xy) {
            if let Some(path) = file_dialog(
                self.prefs.data.last_used_dir.clone(),
                &[("IndexedImage", "ici"), ("AnimatedIndexedImage", "ica")],
            )
            .pick_file()
            {
                self.result = Push(false, SceneName::Editor(EditorDetails::Open(path)));
            }
        }
    }

    fn update(&mut self, _: &Timing, _: &MouseData, _: &[KeyCode]) -> SUR {
        self.result.clone()
    }

    fn resuming(&mut self, _result: Option<SceneResult>) {
        self.prefs.reload();
        self.result = Nothing;
    }

    fn is_dialog(&self) -> bool {
        false
    }
}
