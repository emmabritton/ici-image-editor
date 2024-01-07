use crate::SceneResult::SavePaletteData;
use crate::{SceneName, SceneResult, SUR};
use pixels_graphics_lib::buffer_graphics_lib::prelude::Positioning::CenterTop;
use pixels_graphics_lib::buffer_graphics_lib::prelude::*;
use pixels_graphics_lib::prelude::SceneUpdateResult::*;
use pixels_graphics_lib::prelude::*;
use pixels_graphics_lib::ui::prelude::*;

const WARN_ID: &[&str] = &["ID must be between", "0 and 65535"];
const WARN_NAME: &[&str] = &["Name must not be empty"];

pub struct SavePaletteDataDialog {
    result: SUR,
    background: ShapeCollection,
    cancel: Button,
    save_no_data: Button,
    save_id: Button,
    save_name: Button,
    save_colors: Button,
    id: TextField,
    name: TextField,
    alert: Alert,
    title: Text,
    show_alert: bool,
}

impl SavePaletteDataDialog {
    pub fn new(
        width: usize,
        height: usize,
        alert_style: &AlertStyle,
        style: &DialogStyle,
    ) -> Box<Self> {
        let dialog_pos = style.bounds.top_left();
        let background = dialog_background(width, height, style);
        let alert = Alert::new_warning(&[""], width, height, alert_style);
        let button_width = Some(80);
        let title = Text::new(
            "Palette data?",
            TextPos::px(dialog_pos + (style.bounds.width() / 2, 8)),
            (style.text, TextSize::Normal, CenterTop),
        );
        let save_no_data = Button::new(
            dialog_pos + (8, 25),
            "Don't include",
            button_width,
            &style.button,
        );
        let save_id = Button::new(
            dialog_pos + (8, 50),
            "Save as ID",
            button_width,
            &style.button,
        );
        let id = TextField::new(
            dialog_pos + (100, 53),
            5,
            TextSize::Normal,
            (None, None),
            "",
            &[TextFilter::Numbers],
            &style.text_field,
        );
        let save_name = Button::new(
            dialog_pos + (8, 75),
            "Save as name",
            button_width,
            &style.button,
        );
        let name = TextField::new(
            dialog_pos + (8, 96),
            36,
            TextSize::Small,
            (None, None),
            "",
            &[TextFilter::All],
            &style.text_field,
        );
        let save_colors = Button::new(
            dialog_pos + (8, 112),
            "Include color list",
            button_width,
            &style.button,
        );
        let cancel = Button::new(
            dialog_pos + (55, 146),
            "Cancel",
            button_width,
            &style.button,
        );
        let result = Nothing;
        Box::new(SavePaletteDataDialog {
            result,
            background,
            cancel,
            save_no_data,
            save_id,
            save_name,
            save_colors,
            id,
            name,
            alert,
            title,
            show_alert: false,
        })
    }
}

impl Scene<SceneResult, SceneName> for SavePaletteDataDialog {
    fn render(&self, graphics: &mut Graphics, mouse: &MouseData, _: &[KeyCode]) {
        self.background.render(graphics);
        self.title.render(graphics);
        self.id.render(graphics, mouse);
        self.name.render(graphics, mouse);
        self.cancel.render(graphics, mouse);
        self.save_id.render(graphics, mouse);
        self.save_no_data.render(graphics, mouse);
        self.save_name.render(graphics, mouse);
        self.save_colors.render(graphics, mouse);

        if self.show_alert {
            self.alert.render(graphics, mouse);
        }
    }

    fn on_key_up(&mut self, key: KeyCode, _: &MouseData, held: &[KeyCode]) {
        self.id.on_key_press(key, held);
        self.name.on_key_press(key, held);
    }

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
        if self.show_alert {
            if self.alert.on_mouse_click(down_at, mouse.xy) == Some(AlertResult::Positive) {
                self.show_alert = false;
            }
            return;
        }
        self.id.on_mouse_click(down_at, mouse.xy);
        self.name.on_mouse_click(down_at, mouse.xy);

        if self.save_no_data.on_mouse_click(down_at, mouse.xy) {
            self.result = Pop(Some(SavePaletteData(FilePalette::NoData)));
        }
        if self.save_id.on_mouse_click(down_at, mouse.xy) {
            if self.name.content().is_empty() {
                self.alert.change_text(WARN_ID);
                self.show_alert = true;
            } else {
            }
        }
        if self.save_name.on_mouse_click(down_at, mouse.xy) {
            if self.name.content().is_empty() {
                self.alert.change_text(WARN_NAME);
                self.show_alert = true;
            } else {
            }
        }
        if self.save_colors.on_mouse_click(down_at, mouse.xy) {
            self.result = Pop(Some(SavePaletteData(FilePalette::Colors)));
        }
        if self.cancel.on_mouse_click(down_at, mouse.xy) {
            self.result = Pop(None);
        }
    }

    fn update(&mut self, timing: &Timing, _: &MouseData, _: &[KeyCode]) -> SUR {
        self.id.update(timing);
        self.name.update(timing);

        self.result.clone()
    }

    fn resuming(&mut self, _: Option<SceneResult>) {
        self.result = Nothing;
    }

    fn is_dialog(&self) -> bool {
        true
    }
}
