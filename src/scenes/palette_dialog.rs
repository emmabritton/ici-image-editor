use crate::SceneUpdateResult::{Nothing, Pop};
use crate::{Scene, SceneName, SceneResult, Settings, SUR};

use pixels_graphics_lib::buffer_graphics_lib::prelude::*;
use pixels_graphics_lib::prelude::*;
use pixels_graphics_lib::ui::prelude::TextFilter::Numbers;
use pixels_graphics_lib::ui::prelude::*;

use std::fs;
use std::path::PathBuf;

use crate::palettes::*;
use crate::scenes::file_dialog;
use std::str::FromStr;

const PAL_POS: Coord = Coord::new(5, 48);
const PAL_TILE_SIZE: usize = 6;
const PAL_SPACED: usize = 9;
const PAL_PER_ROW: usize = 21;
const PAL_WIDTH: usize = PAL_SPACED * PAL_PER_ROW;
const PAL_HEIGHT: usize = PAL_SPACED * 2;

pub struct PaletteDialog {
    result: SUR,
    dos: Button,
    gb: Button,
    pico: Button,
    vic: Button,
    zx: Button,
    def: Button,
    ok: Button,
    cancel: Button,
    replace: Button,
    delete: Button,
    add: Button,
    save: Button,
    load: Button,
    background: ShapeCollection,
    selected_color: usize,
    colors: Vec<Color>,
    red: TextField,
    green: TextField,
    blue: TextField,
    alpha: TextField,
    current_color: Color,
    dialog_pos: Coord,
    file_path: Option<PathBuf>,
    prefs: AppPrefs<Settings>,
}

impl PaletteDialog {
    pub fn new(
        colors: Vec<Color>,
        width: usize,
        height: usize,
        selected: usize,
        prefs: AppPrefs<Settings>,
        style: &UiStyle,
    ) -> Box<Self> {
        let dialog_pos = style.dialog.bounds.top_left();
        let background = dialog_background(width, height, &style.dialog);
        let button_start_pos = dialog_pos + (4, 4);
        let button_offset_x = 64_isize;
        let button_offset_y = 20_isize;

        let vic = Button::new(button_start_pos, "Vic-20", Some(60), &style.button);
        let dos = Button::new(
            button_start_pos + (button_offset_x, 0),
            "DOS",
            Some(60),
            &style.button,
        );
        let gb = Button::new(
            button_start_pos + (button_offset_x * 2, 0),
            "GB",
            Some(60),
            &style.button,
        );
        let pico = Button::new(
            button_start_pos + (0, button_offset_y),
            "Pico-8",
            Some(60),
            &style.button,
        );
        let zx = Button::new(
            button_start_pos + (button_offset_x, button_offset_y),
            "ZX",
            Some(60),
            &style.button,
        );
        let def = Button::new(
            button_start_pos + (button_offset_x * 2, button_offset_y),
            "Default",
            Some(60),
            &style.button,
        );

        let pal_but_y = 70;
        let edit = Button::new(
            (button_start_pos.x, dialog_pos.y + pal_but_y),
            "Replace",
            Some(60),
            &style.button,
        );
        let delete = Button::new(
            (
                button_start_pos.x + button_offset_x,
                dialog_pos.y + pal_but_y,
            ),
            "Delete",
            Some(60),
            &style.button,
        );
        let add = Button::new(
            (
                button_start_pos.x + button_offset_x * 2,
                dialog_pos.y + pal_but_y,
            ),
            "Add",
            Some(60),
            &style.button,
        );

        let cancel = Button::new(dialog_pos + (4, 148), "Cancel", Some(60), &style.button);
        let ok = Button::new(dialog_pos + (132, 148), "OK", Some(60), &style.button);

        let red = TextField::new(
            dialog_pos + (70, 94),
            3,
            PixelFont::Standard6x7,
            (None, None),
            "",
            &[Numbers],
            &style.text_field,
        );
        let green = TextField::new(
            dialog_pos + (70, 106),
            3,
            PixelFont::Standard6x7,
            (None, None),
            "",
            &[Numbers],
            &style.text_field,
        );
        let blue = TextField::new(
            dialog_pos + (70, 118),
            3,
            PixelFont::Standard6x7,
            (None, None),
            "",
            &[Numbers],
            &style.text_field,
        );
        let alpha = TextField::new(
            dialog_pos + (70, 130),
            3,
            PixelFont::Standard6x7,
            (None, None),
            "",
            &[Numbers],
            &style.text_field,
        );

        let save = Button::new(dialog_pos + (132, 98), "Save", Some(60), &style.button);
        let load = Button::new(dialog_pos + (132, 118), "Load", Some(60), &style.button);

        let mut dialog = Self {
            file_path: None,
            save,
            load,
            dialog_pos,
            result: Nothing,
            dos,
            gb,
            pico,
            vic,
            zx,
            def,
            ok,
            cancel,
            replace: edit,
            delete,
            add,
            background,
            selected_color: selected,
            colors: colors.into_iter().take(42).collect(),
            red,
            green,
            blue,
            alpha,
            current_color: WHITE,
            prefs,
        };
        dialog.update_selected_color_display();
        Box::new(dialog)
    }

    fn reset_colors(&mut self, colors: &[Color]) {
        if colors.is_empty() {
            return;
        }
        self.colors.clear();
        self.colors = colors.to_vec();
        self.selected_color = 0;
        self.update_selected_color_display();
    }

    fn update_selected_color_display(&mut self) {
        self.current_color = self.colors[self.selected_color];
        self.red.set_content(&self.current_color.r.to_string());
        self.green.set_content(&self.current_color.g.to_string());
        self.blue.set_content(&self.current_color.b.to_string());
        self.alpha.set_content(&self.current_color.a.to_string());
    }

    fn save_palette(&mut self) {
        if let Some(path) = &self.file_path {
            let output = JascPalette::new(self.colors.clone()).to_file_contents();
            fs::write(path, output).expect("Writing palette to disk");
        }
    }

    fn load_palette(&mut self) {
        if let Some(path) = &self.file_path {
            let input = fs::read_to_string(path).expect("Reading palette from disk");
            let palette = JascPalette::from_file_contents(&input).expect("Decoding palette");
            self.colors = palette.colors;
        }
    }
}

impl Scene<SceneResult, SceneName> for PaletteDialog {
    fn render(&self, graphics: &mut Graphics, mouse: &MouseData, _: &FxHashSet<KeyCode>) {
        self.background.render(graphics);
        self.dos.render(graphics, mouse);
        self.gb.render(graphics, mouse);
        self.pico.render(graphics, mouse);
        self.vic.render(graphics, mouse);
        self.zx.render(graphics, mouse);
        self.def.render(graphics, mouse);
        self.ok.render(graphics, mouse);
        self.cancel.render(graphics, mouse);
        self.replace.render(graphics, mouse);
        self.delete.render(graphics, mouse);
        self.add.render(graphics, mouse);
        self.red.render(graphics, mouse);
        self.green.render(graphics, mouse);
        self.blue.render(graphics, mouse);
        self.alpha.render(graphics, mouse);
        self.save.render(graphics, mouse);
        self.load.render(graphics, mouse);

        graphics.draw_rect(
            Rect::new_with_size(self.dialog_pos + (14, 96), 16, 16),
            fill(LIGHT_GRAY),
        );
        graphics.draw_rect(
            Rect::new_with_size(self.dialog_pos + (30, 96), 16, 16),
            fill(DARK_GRAY),
        );
        graphics.draw_rect(
            Rect::new_with_size(self.dialog_pos + (14, 112), 16, 16),
            fill(DARK_GRAY),
        );
        graphics.draw_rect(
            Rect::new_with_size(self.dialog_pos + (30, 112), 16, 16),
            fill(LIGHT_GRAY),
        );
        graphics.draw_rect(
            Rect::new_with_size(self.dialog_pos + (14, 96), 32, 32),
            fill(self.current_color),
        );

        graphics.draw_text(
            "R",
            TextPos::px(self.dialog_pos + (60, 96)),
            (WHITE, PixelFont::Standard6x7),
        );
        graphics.draw_text(
            "G",
            TextPos::px(self.dialog_pos + (60, 108)),
            (WHITE, PixelFont::Standard6x7),
        );
        graphics.draw_text(
            "B",
            TextPos::px(self.dialog_pos + (60, 120)),
            (WHITE, PixelFont::Standard6x7),
        );
        graphics.draw_text(
            "A",
            TextPos::px(self.dialog_pos + (60, 132)),
            (WHITE, PixelFont::Standard6x7),
        );

        let mut y = 0;
        let mut x = 0;
        for (i, color) in self.colors.iter().enumerate() {
            graphics.draw_rect(
                Rect::new_with_size(
                    self.dialog_pos + PAL_POS + (x * PAL_SPACED, y * PAL_SPACED),
                    PAL_TILE_SIZE,
                    PAL_TILE_SIZE,
                ),
                fill(*color),
            );
            if i == self.selected_color {
                graphics.draw_rect(
                    Rect::new_with_size(
                        self.dialog_pos + PAL_POS + (x * PAL_SPACED, y * PAL_SPACED) - (1, 1),
                        PAL_TILE_SIZE + 2,
                        PAL_TILE_SIZE + 2,
                    ),
                    stroke(WHITE),
                );
            }
            x += 1;
            if x >= PAL_PER_ROW {
                x = 0;
                y += 1;
            }
        }
    }

    fn on_key_up(&mut self, key: KeyCode, _: &MouseData, held: &FxHashSet<KeyCode>) {
        self.red.on_key_press(key, held);
        self.green.on_key_press(key, held);
        self.blue.on_key_press(key, held);
        self.alpha.on_key_press(key, held);
        let r = u8::from_str(self.red.content()).unwrap_or_default();
        let g = u8::from_str(self.green.content()).unwrap_or_default();
        let b = u8::from_str(self.blue.content()).unwrap_or_default();
        let a = u8::from_str(self.alpha.content()).unwrap_or_default();
        self.current_color = Color { r, g, b, a };
        let shift_down = held.contains(&KeyCode::ShiftLeft) || held.contains(&KeyCode::ShiftRight);
        let tab_down = key == KeyCode::Tab;
        if tab_down && self.red.is_focused() {
            self.red.unfocus();
            if shift_down {
                self.blue.focus();
            } else {
                self.green.focus();
            }
        } else if tab_down && self.green.is_focused() {
            self.green.unfocus();
            if shift_down {
                self.red.focus();
            } else {
                self.blue.focus();
            }
        } else if tab_down && self.blue.is_focused() {
            self.blue.unfocus();
            if shift_down {
                self.green.focus();
            } else {
                self.red.focus();
            }
        }
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
        if self.cancel.on_mouse_click(down_at, mouse.xy) {
            self.result = Pop(None);
        }
        if self.ok.on_mouse_click(down_at, mouse.xy) {
            self.result = Pop(Some(SceneResult::Palette(
                self.colors.clone(),
                self.selected_color,
            )));
        }
        if self.gb.on_mouse_click(down_at, mouse.xy) {
            self.reset_colors(&palette_gb().colors);
        }
        if self.dos.on_mouse_click(down_at, mouse.xy) {
            self.reset_colors(&palette_dos().colors);
        }
        if self.vic.on_mouse_click(down_at, mouse.xy) {
            self.reset_colors(&palette_vic().colors);
        }
        if self.pico.on_mouse_click(down_at, mouse.xy) {
            self.reset_colors(&palette_pico().colors);
        }
        if self.zx.on_mouse_click(down_at, mouse.xy) {
            self.reset_colors(&palette_zx().colors);
        }
        if self.def.on_mouse_click(down_at, mouse.xy) {
            self.reset_colors(&palette_default().colors);
        }
        if self.add.on_mouse_click(down_at, mouse.xy) && self.colors.len() < 42 {
            self.colors.push(self.current_color);
        }
        if self.delete.on_mouse_click(down_at, mouse.xy) && self.colors.len() > 1 {
            self.colors.remove(self.selected_color);
            if self.selected_color >= self.colors.len() {
                self.selected_color = 0;
                self.update_selected_color_display();
            }
        }
        if self.replace.on_mouse_click(down_at, mouse.xy) {
            self.colors[self.selected_color] = self.current_color;
        }
        self.red.on_mouse_click(down_at, mouse.xy);
        self.green.on_mouse_click(down_at, mouse.xy);
        self.blue.on_mouse_click(down_at, mouse.xy);
        self.alpha.on_mouse_click(down_at, mouse.xy);
        let start = self.dialog_pos + PAL_POS;
        if Rect::new_with_size(start, PAL_WIDTH, PAL_HEIGHT).contains(mouse.xy) {
            let x = (mouse.xy.x - start.x) / PAL_SPACED as isize;
            let y = (mouse.xy.y - start.y) / PAL_SPACED as isize;
            let idx = (x + y * PAL_PER_ROW as isize) as usize;
            if idx < self.colors.len() {
                self.selected_color = idx;
                self.update_selected_color_display();
            }
        }
        if self.save.on_mouse_click(down_at, mouse.xy) {
            if let Some(path) = file_dialog(
                self.file_path
                    .clone()
                    .unwrap_or(self.prefs.data.last_used_pal_dir.clone()),
                &[("Palette", "pal")],
            )
            .save_file()
            {
                self.file_path = Some(path.clone());
                self.prefs.data.last_used_pal_dir = path;
                self.save_palette();
            }
        }
        if self.load.on_mouse_click(down_at, mouse.xy) {
            if let Some(path) = file_dialog(
                self.file_path
                    .clone()
                    .unwrap_or(self.prefs.data.last_used_pal_dir.clone()),
                &[("Palette", "pal")],
            )
            .pick_file()
            {
                self.file_path = Some(path.clone());
                self.prefs.data.last_used_pal_dir = path;
                self.load_palette();
            }
        }
    }

    fn update(&mut self, timing: &Timing, _: &MouseData, _: &FxHashSet<KeyCode>, _: &Window) -> SUR {
        self.red.update(timing);
        self.green.update(timing);
        self.blue.update(timing);
        self.alpha.update(timing);
        self.result.clone()
    }

    fn resuming(&mut self, _result: Option<SceneResult>) {
        self.result = Nothing;
    }

    fn is_dialog(&self) -> bool {
        true
    }
}
