use crate::palettes::Palette;
use crate::scenes::dialog_background;
use crate::ui::prelude::TextFilter::Numbers;
use crate::ui::prelude::*;
use crate::SceneName::{LoadFile, SaveFile};
use crate::SceneUpdateResult::{Nothing, Pop};
use crate::{Scene, SceneName, SceneResult, SUR};
use pixels_graphics_lib::prelude::*;
use pixels_graphics_lib::scenes::SceneUpdateResult::Push;
use std::fs;
use std::str::FromStr;

const PAL_POS: Coord = Coord::new(5, 48);
const PAL_TILE_SIZE: usize = 6;
const PAL_SPACED: usize = 9;
const PAL_PER_ROW: usize = 21;
const PAL_WIDTH: usize = PAL_SPACED * PAL_PER_ROW;
const PAL_HEIGHT: usize = PAL_SPACED * 2;

#[derive(Debug)]
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
    current_color: Color,
    dialog_pos: Coord,
    file_path: Option<String>,
}

impl PaletteDialog {
    pub fn new(colors: Vec<Color>, width: usize, height: usize, style: &DialogStyle) -> Box<Self> {
        let dialog_pos = style.bounds.top_left();
        let background = dialog_background(width, height, style);
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
            Normal,
            (None, None),
            "",
            &[Numbers],
            &style.text_field,
        );
        let green = TextField::new(
            dialog_pos + (70, 106),
            3,
            Normal,
            (None, None),
            "",
            &[Numbers],
            &style.text_field,
        );
        let blue = TextField::new(
            dialog_pos + (70, 118),
            3,
            Normal,
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
            selected_color: 0,
            colors: colors.into_iter().take(42).collect(),
            red,
            green,
            blue,
            current_color: WHITE,
        };
        dialog.update_selected_color_display();
        Box::new(dialog)
    }

    fn reset_colors(&mut self, colors: &[Color]) {
        if colors.is_empty() {
            return;
        }
        self.colors.clear();
        self.colors.extend_from_slice(colors);
        self.selected_color = 0;
        self.update_selected_color_display();
    }

    fn update_selected_color_display(&mut self) {
        self.current_color = self.colors[self.selected_color];
        self.red.set_content(&self.current_color.r.to_string());
        self.green.set_content(&self.current_color.g.to_string());
        self.blue.set_content(&self.current_color.b.to_string());
    }

    fn save_palette(&mut self, path: String) {
        let output = Palette {
            colors: self.colors.clone(),
        }
        .to_file_contents();
        fs::write(&path, output).expect("Writing palette to disk");
        self.file_path = Some(path);
    }

    fn load_palette(&mut self, path: String) {
        let input = fs::read_to_string(&path).expect("Reading palette from disk");
        let palette = Palette::from_file_contents(&input).expect("Decoding palette");
        self.colors = palette.colors;
        self.file_path = Some(path);
    }
}

impl Scene<SceneResult, SceneName> for PaletteDialog {
    fn render(&self, graphics: &mut Graphics, mouse_xy: Coord) {
        self.background.render(graphics);
        self.dos.render(graphics, mouse_xy);
        self.gb.render(graphics, mouse_xy);
        self.pico.render(graphics, mouse_xy);
        self.vic.render(graphics, mouse_xy);
        self.zx.render(graphics, mouse_xy);
        self.def.render(graphics, mouse_xy);
        self.ok.render(graphics, mouse_xy);
        self.cancel.render(graphics, mouse_xy);
        self.replace.render(graphics, mouse_xy);
        self.delete.render(graphics, mouse_xy);
        self.add.render(graphics, mouse_xy);
        self.red.render(graphics, mouse_xy);
        self.green.render(graphics, mouse_xy);
        self.blue.render(graphics, mouse_xy);
        self.save.render(graphics, mouse_xy);
        self.load.render(graphics, mouse_xy);

        graphics.draw_rect(
            Rect::new_with_size(self.dialog_pos + (14, 96), 32, 32),
            fill(self.current_color),
        );

        graphics.draw_text(
            "R",
            TextPos::px(self.dialog_pos + (60, 96)),
            (WHITE, Normal),
        );
        graphics.draw_text(
            "G",
            TextPos::px(self.dialog_pos + (60, 108)),
            (WHITE, Normal),
        );
        graphics.draw_text(
            "B",
            TextPos::px(self.dialog_pos + (60, 120)),
            (WHITE, Normal),
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

    fn on_key_up(&mut self, key: VirtualKeyCode, _: &Vec<&VirtualKeyCode>) {
        self.red.on_key_press(key);
        self.green.on_key_press(key);
        self.blue.on_key_press(key);
        let r = u8::from_str(self.red.content()).unwrap_or_default();
        let g = u8::from_str(self.green.content()).unwrap_or_default();
        let b = u8::from_str(self.blue.content()).unwrap_or_default();
        self.current_color = Color { r, g, b, a: 255 };
    }

    fn on_mouse_up(&mut self, xy: Coord, button: MouseButton, _: &Vec<&VirtualKeyCode>) {
        if button != MouseButton::Left {
            return;
        }
        if self.cancel.on_mouse_click(xy) {
            self.result = Pop(None);
        }
        if self.ok.on_mouse_click(xy) {
            self.result = Pop(Some(SceneResult::Palette(self.colors.clone())));
        }
        if self.gb.on_mouse_click(xy) {
            self.reset_colors(&Palette::builtin_gb().colors);
        }
        if self.dos.on_mouse_click(xy) {
            self.reset_colors(&Palette::builtin_dos().colors);
        }
        if self.vic.on_mouse_click(xy) {
            self.reset_colors(&Palette::builtin_vic().colors);
        }
        if self.pico.on_mouse_click(xy) {
            self.reset_colors(&Palette::builtin_pico().colors);
        }
        if self.zx.on_mouse_click(xy) {
            self.reset_colors(&Palette::builtin_zx().colors);
        }
        if self.def.on_mouse_click(xy) {
            self.reset_colors(&Palette::default().colors);
        }
        if self.add.on_mouse_click(xy) && self.colors.len() < 42 {
            self.colors.push(self.current_color);
        }
        if self.delete.on_mouse_click(xy) && self.colors.len() > 1 {
            self.colors.remove(self.selected_color);
            if self.selected_color >= self.colors.len() {
                self.selected_color = 0;
                self.update_selected_color_display();
            }
        }
        if self.replace.on_mouse_click(xy) {
            self.colors[self.selected_color] = self.current_color;
        }
        self.red.on_mouse_click(xy);
        self.green.on_mouse_click(xy);
        self.blue.on_mouse_click(xy);
        let start = self.dialog_pos + PAL_POS;
        if Rect::new_with_size(start, PAL_WIDTH, PAL_HEIGHT).contains(xy) {
            let x = (xy.x - start.x) / PAL_SPACED as isize;
            let y = (xy.y - start.y) / PAL_SPACED as isize;
            let idx = (x + y * PAL_PER_ROW as isize) as usize;
            if idx < self.colors.len() {
                self.selected_color = idx;
                self.update_selected_color_display();
            }
        }
        if self.save.on_mouse_click(xy) {
            self.result = Push(false, SaveFile(String::from("pal"), self.file_path.clone()));
        }
        if self.load.on_mouse_click(xy) {
            self.result = Push(false, LoadFile(String::from("pal")));
        }
    }

    fn update(&mut self, timing: &Timing, _: Coord, _: &Vec<&VirtualKeyCode>) -> SUR {
        self.red.update(timing);
        self.green.update(timing);
        self.blue.update(timing);
        self.result.clone()
    }

    fn resuming(&mut self, result: Option<SceneResult>) {
        if let Some(result) = result {
            match result {
                SceneResult::LoadFilePath(path) => self.load_palette(path),
                SceneResult::SaveFilePath(path) => self.save_palette(path),
                _ => {}
            }
        }
        self.result = Nothing;
    }

    fn is_dialog(&self) -> bool {
        true
    }
}
