use pixels_graphics_lib::prelude::*;
use pixels_graphics_lib::ui::prelude::relative::LayoutContext;
use pixels_graphics_lib::ui::prelude::*;
use pixels_graphics_lib::{layout, px, render};

use crate::scenes::editor::EditorDetails;
use crate::scenes::{file_dialog, import_image, BACKGROUND};
use crate::SceneUpdateResult::{Nothing, Push};
use crate::{DefaultPalette, Scene, SceneName, SceneResult, Settings, HEIGHT, SUR, WIDTH};

const PALETTE_INFO_POS: TextPos = TextPos::Px(10, 200);

pub struct Menu {
    result: SUR,
    title: Label,
    new_button: Button,
    load_button: Button,
    import_button: Button,
    format_label: Label,
    default_palette: DefaultPalette,
    prefs: AppPrefs<Settings>,
    warning: Option<Alert>,
    alert_style: AlertStyle,
}

impl Menu {
    pub fn new(prefs: AppPrefs<Settings>, palette: DefaultPalette, style: &UiStyle) -> Box<Self> {
        let mut title = Label::new(Text::new(
            "ICI IMAGE EDITOR",
            TextPos::Px(0, 0),
            (
                WHITE,
                PixelFont::Standard8x10,
                WrappingStrategy::SpaceBeforeCol(7),
            ),
        ));
        let mut new_button = Button::new((0, 0), "New image", Some(86), &style.button);
        let mut load_button = Button::new((0, 0), "Load ICI", Some(86), &style.button);
        let mut import_button = Button::new((0, 0), "Import", Some(86), &style.button);
        let mut format_label = Label::singleline(
            "PNG, BMP, JPG & TGA",
            (0, 0),
            WHITE,
            PixelFont::Limited3x5,
            WIDTH,
        );

        let context = LayoutContext::new(Rect::new_with_size((0, 0), WIDTH, HEIGHT));

        layout!(context, title, align_top, px!(10));
        layout!(context, title, align_left, px!(10));

        layout!(context, new_button, left_to_left_of title);
        layout!(context, new_button, top_to_bottom_of title, px!(30));

        layout!(context, load_button, left_to_left_of title);
        layout!(context, load_button, top_to_bottom_of new_button, px!(8));

        layout!(context, import_button, left_to_left_of title);
        layout!(context, import_button, top_to_bottom_of load_button, px!(8));

        layout!(context, format_label, left_to_left_of import_button);
        layout!(context, format_label, top_to_bottom_of import_button, px!(4));

        Box::new(Self {
            result: Nothing,
            title,
            new_button,
            load_button,
            import_button,
            prefs,
            default_palette: palette,
            warning: None,
            alert_style: style.alert.clone(),
            format_label,
        })
    }
}

impl Scene<SceneResult, SceneName> for Menu {
    fn render(&self, graphics: &mut Graphics, mouse: &MouseData, _: &FxHashSet<KeyCode>) {
        graphics.clear(BACKGROUND);

        render!(
            graphics,
            mouse,
            self.import_button,
            self.new_button,
            self.title,
            self.load_button,
            self.format_label
        );

        match &self.default_palette {
            DefaultPalette::NoPalette => {}
            DefaultPalette::Error(err) => graphics.draw_text(
                &format!("Error: {err}"),
                PALETTE_INFO_POS,
                (
                    RED,
                    PixelFont::Standard6x7,
                    WrappingStrategy::SpaceBeforeCol(36),
                ),
            ),
            DefaultPalette::Palette(path, colors) => graphics.draw_text(
                &format!("Using palette {path} with {} colors", colors.len()),
                PALETTE_INFO_POS,
                (
                    WHITE,
                    PixelFont::Standard6x7,
                    WrappingStrategy::SpaceBeforeCol(36),
                ),
            ),
        }
    }

    fn on_key_up(&mut self, _: KeyCode, _: &MouseData, _: &FxHashSet<KeyCode>) {}

    fn on_mouse_click(
        &mut self,
        down_at: Coord,
        mouse: &MouseData,
        button: MouseButton,
        _: &FxHashSet<KeyCode>,
    ) {
        if button != MouseButton::Left {
            return;
        }
        if self.new_button.on_mouse_click(down_at, mouse.xy) {
            self.result = Push(false, SceneName::NewImage(None));
        }
        if self.load_button.on_mouse_click(down_at, mouse.xy) {
            if let Some(path) = file_dialog(
                self.prefs.data.last_used_dir.clone(),
                &[("IndexedImage", "ici"), ("AnimatedIndexedImage", "ica")],
            )
            .pick_file()
            {
                self.result = Push(true, SceneName::Editor(EditorDetails::Open(path)));
            }
        }
        if self.import_button.on_mouse_click(down_at, mouse.xy) {
            if let Some(result) = import_image(&self.alert_style, &mut self.prefs) {
                match result {
                    Ok(img) => {
                        self.result = Push(true, SceneName::Editor(EditorDetails::OpenImage(img)))
                    }
                    Err(alert) => self.warning = Some(alert),
                }
            }
        }
    }

    fn update(&mut self, _: &Timing, _: &MouseData, _: &FxHashSet<KeyCode>, _: &Window) -> SUR {
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
