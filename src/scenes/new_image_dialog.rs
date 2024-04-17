use crate::scenes::editor::EditorDetails;
use crate::SceneName::Editor;
use crate::SceneUpdateResult::*;
use crate::{Scene, SceneName, SceneResult, SUR};
use pixels_graphics_lib::buffer_graphics_lib::prelude::*;
use pixels_graphics_lib::prelude::*;
use pixels_graphics_lib::ui::layout::relative::LayoutContext;
use pixels_graphics_lib::ui::prelude::TextFilter::Numbers;
use pixels_graphics_lib::ui::prelude::*;
use pixels_graphics_lib::{layout, px, render};
use std::str::FromStr;

#[derive(Debug)]
pub struct NewImageDialog {
    result: SUR,
    width_field: TextField,
    width_label: Label,
    height_field: TextField,
    height_label: Label,
    x_label: Label,
    size_label: Label,
    palette_checkbox: Checkbox,
    submit_button: Button,
    cancel_button: Button,
    background: ShapeCollection,
    alert: Alert,
    alert_visible: bool,
    quick_8: Button,
    quick_12: Button,
    quick_16: Button,
    quick_24: Button,
    quick_32: Button,
    quick_48: Button,
    quick_64: Button,
    quick_8_16: Button,
    quick_16_32: Button,
    label: Text,
    palette: Option<Vec<Color>>,
}

impl NewImageDialog {
    pub fn new(
        width: usize,
        height: usize,
        palette: Option<Vec<Color>>,
        style: &UiStyle,
    ) -> Box<Self> {
        let background = dialog_background(width, height, &style.dialog);
        let context = LayoutContext::new(style.dialog.bounds.clone());
        let mut width_label = Label::new(Text::new(
            "Width",
            TextPos::Px(0, 0),
            (WHITE, PixelFont::Standard6x7),
        ));
        let mut width_field = TextField::new(
            (0, 0),
            2,
            PixelFont::Standard6x7,
            (None, None),
            "",
            &[Numbers],
            &style.text_field,
        );
        let mut height_label = Label::new(Text::new(
            "Height",
            TextPos::Px(0, 0),
            (WHITE, PixelFont::Standard6x7),
        ));
        let mut x_label = Label::new(Text::new(
            "x",
            TextPos::Px(0, 0),
            (WHITE, PixelFont::Standard6x7),
        ));
        let mut height_field = TextField::new(
            (0, 0),
            2,
            PixelFont::Standard6x7,
            (None, None),
            "",
            &[Numbers],
            &style.text_field,
        );
        let mut size_label =
            Label::singleline("Between 0..=64", (0, 0), WHITE, PixelFont::Standard4x5, 100);
        let mut palette_checkbox = Checkbox::new((0, 0), "Copy palette", false, &style.checkbox);

        layout!(context, width_label, align_top, px!(8));
        layout!(context, width_label, align_left, px!(8));

        layout!(context, width_field, left_to_left_of width_label);
        layout!(context, grow width_field, right_to_right_of width_label);
        layout!(context, width_field, top_to_bottom_of  width_label);

        layout!(context, x_label, left_to_right_of  width_field, px!(4));
        layout!(context, x_label, bottom_to_bottom_of  width_field);

        layout!(context, height_field, left_to_right_of  x_label, px!(4));
        layout!(context, height_field, bottom_to_bottom_of  x_label);

        layout!(context, height_label, left_to_left_of  height_field);
        layout!(context, grow height_field, right_to_right_of height_label);
        layout!(context, height_label, top_to_top_of  width_label );

        layout!(context, size_label, bottom_to_bottom_of   height_field );
        layout!(context, size_label, left_to_right_of   height_label, px!(6) );

        layout!(context, palette_checkbox, left_to_left_of width_label);
        layout!(context, palette_checkbox, top_to_bottom_of  width_field, px!(8));

        let submit_button = Button::new(
            style.dialog.bounds.top_left() + (138, 144),
            "Create",
            None,
            &style.button,
        );
        let cancel_button = Button::new(
            style.dialog.bounds.top_left() + (8, 144),
            "Cancel",
            None,
            &style.button,
        );
        let row1_y = 70;
        let row2_y = 90;
        let row3_y = 110;
        let quick_8 = Button::new(
            style.dialog.bounds.top_left() + (8, row1_y),
            "8x8",
            Some(50),
            &style.button,
        );
        let quick_12 = Button::new(
            (
                quick_8.bounds().bottom_right().x + 8,
                style.dialog.bounds.top_left().y + row1_y,
            ),
            "12x12",
            Some(50),
            &style.button,
        );
        let quick_16 = Button::new(
            (
                quick_12.bounds().bottom_right().x + 8,
                style.dialog.bounds.top_left().y + row1_y,
            ),
            "16x16",
            Some(50),
            &style.button,
        );
        let quick_24 = Button::new(
            style.dialog.bounds.top_left() + (8, row2_y),
            "24x24",
            Some(50),
            &style.button,
        );
        let quick_32 = Button::new(
            (
                quick_24.bounds().bottom_right().x + 8,
                style.dialog.bounds.top_left().y + row2_y,
            ),
            "32x32",
            Some(50),
            &style.button,
        );
        let quick_48 = Button::new(
            (
                quick_32.bounds().bottom_right().x + 8,
                style.dialog.bounds.top_left().y + row2_y,
            ),
            "48x48",
            Some(50),
            &style.button,
        );
        let quick_64 = Button::new(
            style.dialog.bounds.top_left() + (8, row3_y),
            "64x64",
            Some(50),
            &style.button,
        );
        let quick_8_16 = Button::new(
            (
                quick_64.bounds().bottom_right().x + 8,
                style.dialog.bounds.top_left().y + row3_y,
            ),
            "8x16",
            Some(50),
            &style.button,
        );
        let quick_16_32 = Button::new(
            (
                quick_8_16.bounds().bottom_right().x + 8,
                style.dialog.bounds.top_left().y + row3_y,
            ),
            "16x32",
            Some(50),
            &style.button,
        );
        let label = Text::new(
            "Quick create",
            TextPos::px(style.dialog.bounds.top_left() + (8, 60)),
            (WHITE, PixelFont::Standard6x7),
        );
        let alert = Alert::new_warning(&[""], width, height, &style.alert);
        width_field.focus();
        Box::new(Self {
            x_label,
            size_label,
            result: Nothing,
            width_field,
            width_label,
            height_field,
            height_label,
            submit_button,
            cancel_button,
            background,
            alert,
            alert_visible: false,
            quick_8,
            palette,
            quick_12,
            quick_16,
            quick_24,
            quick_32,
            quick_48,
            quick_64,
            quick_8_16,
            quick_16_32,
            label,
            palette_checkbox,
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
                Ok((width, height))
            }
        }
    }

    fn set_success(&mut self, width: u8, height: u8) {
        let palette = if self.palette_checkbox.is_checked() {
            self.palette.clone()
        } else {
            None
        };
        self.result = Push(true, Editor(EditorDetails::New(width, height, palette)));
    }

    fn submit(&mut self) {
        match self.verify() {
            Ok((w, h)) => self.set_success(w, h),
            Err(err) => {
                self.alert.change_text(&[&err]);
                self.alert_visible = true;
            }
        }
    }
}

impl Scene<SceneResult, SceneName> for NewImageDialog {
    fn render(&self, graphics: &mut Graphics, mouse: &MouseData, _: &FxHashSet<KeyCode>) {
        graphics.draw(&self.background);
        self.label.render(graphics);
        render!(
            graphics,
            mouse,
            self.submit_button,
            self.cancel_button,
            self.quick_8,
            self.quick_12,
            self.quick_16,
            self.quick_32,
            self.quick_24,
            self.quick_48,
            self.quick_64,
            self.quick_8_16,
            self.quick_16_32,
            self.size_label,
            self.x_label,
            self.width_label,
            self.height_label,
            self.width_field,
            self.height_field
        );

        if self.palette.is_some() {
            self.palette_checkbox.render(graphics, mouse);
        }

        if self.alert_visible {
            self.alert.render(graphics, mouse);
        }
    }

    fn on_key_up(&mut self, key: KeyCode, _: &MouseData, held: &FxHashSet<KeyCode>) {
        if self.alert_visible {
            return;
        }
        if key == KeyCode::Tab && self.width_field.is_focused() {
            self.width_field.unfocus();
            self.height_field.focus();
        } else if self.height_field.is_focused() {
            if key == KeyCode::Tab
                && (held.contains(&KeyCode::ShiftLeft) || held.contains(&KeyCode::ShiftRight))
            {
                self.height_field.unfocus();
                self.width_field.focus();
            } else if key == KeyCode::Enter {
                self.submit();
            }
        }
        self.width_field.on_key_press(key, held);
        self.height_field.on_key_press(key, held);
    }

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
        if self.alert_visible {
            if self.alert.on_mouse_click(down_at, mouse.xy).is_some() {
                self.alert_visible = false;
            }
            return;
        }
        let _ = self.palette_checkbox.on_mouse_click(down_at, mouse.xy);
        self.width_field.on_mouse_click(down_at, mouse.xy);
        self.height_field.on_mouse_click(down_at, mouse.xy);
        if self.submit_button.on_mouse_click(down_at, mouse.xy) {
            self.submit();
        }
        if self.cancel_button.on_mouse_click(down_at, mouse.xy) {
            self.result = Pop(None);
        }
        if self.quick_8.on_mouse_click(down_at, mouse.xy) {
            self.set_success(8, 8);
        }
        if self.quick_12.on_mouse_click(down_at, mouse.xy) {
            self.set_success(12, 12);
        }
        if self.quick_16.on_mouse_click(down_at, mouse.xy) {
            self.set_success(16, 16);
        }
        if self.quick_24.on_mouse_click(down_at, mouse.xy) {
            self.set_success(24, 24);
        }
        if self.quick_32.on_mouse_click(down_at, mouse.xy) {
            self.set_success(32, 32);
        }
        if self.quick_48.on_mouse_click(down_at, mouse.xy) {
            self.set_success(48, 48);
        }
        if self.quick_64.on_mouse_click(down_at, mouse.xy) {
            self.set_success(64, 64);
        }
        if self.quick_8_16.on_mouse_click(down_at, mouse.xy) {
            self.set_success(8, 16);
        }
        if self.quick_16_32.on_mouse_click(down_at, mouse.xy) {
            self.set_success(16, 32);
        }
    }

    fn update(&mut self, timing: &Timing, _: &MouseData, _: &FxHashSet<KeyCode>) -> SUR {
        self.width_field.update(timing);
        self.height_field.update(timing);
        self.result.clone()
    }

    fn resuming(&mut self, _: Option<SceneResult>) {}

    fn is_dialog(&self) -> bool {
        true
    }
}
