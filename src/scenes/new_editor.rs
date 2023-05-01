use crate::palettes::Palette;
use crate::scenes::{file_dialog, BACKGROUND};
use crate::ui::canvas::{Canvas, Tool};
use crate::ui::palette::PaletteView;
use crate::{SceneName, SceneResult, SUR, WIDTH};
use directories::UserDirs;
use log::error;
use pixels_graphics_lib::prelude::*;
use pixels_graphics_lib::scenes::SceneUpdateResult::{Nothing, Pop};
use pixels_graphics_lib::ui::prelude::TextFilter::Decimal;
use pixels_graphics_lib::ui::prelude::*;
use rfd::FileDialog;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum AlertAction {
    Clear,
    Close,
}

const NAME: Coord = Coord::new(2, 2);
const NAME_LINE_Y: isize = 12;
const PADDING: isize = 4;
const BUTTON_Y: isize = NAME_LINE_Y + PADDING;
const FRAME_CONTROL: Coord = Coord::new(4, 200);
const FRAME_CONTROL_SPACING: isize = 20;
const PALETTE_HEIGHT: usize = 200;

const TOOL_PENCIL: usize = 0;
const TOOL_LINE: usize = 1;
const TOOL_RECT: usize = 2;
const TOOL_FILL: usize = 3;

const UNTITLED: &str = "untitled";
const CORRUPT: &str = "???";

#[derive(Debug, Clone, PartialEq)]
pub enum EditorDetails {
    Open(String),
    New(u8, u8),
}

pub struct Editor {
    result: SUR,
    clear: Button,
    tools: ToggleIconButtonGroup<usize>,
    save: Button,
    save_as: Button,
    close: Button,
    edit_palette: Button,
    speed: TextField,
    play_pause: IconButton,
    add_frame: IconButton,
    remove_frame: IconButton,
    copy_frame: IconButton,
    alert: Alert,
    pending_alert_action: Option<AlertAction>,
    filepath: Option<String>,
    canvas: Canvas,
    frames: Vec<IndexedImage>,
    palette: PaletteView,
}

impl Editor {
    pub fn new(
        width: usize,
        height: usize,
        details: EditorDetails,
        style: &UiStyle,
    ) -> Box<Editor> {
        let save = Button::new((PADDING, BUTTON_Y), "Save", None, &style.button);
        let save_as = Button::new(
            (save.bounds().bottom_right().x + PADDING, BUTTON_Y),
            "Save As",
            None,
            &style.button,
        );
        let clear = Button::new((188, BUTTON_Y), "Clear", None, &style.button);
        let close = Button::new(
            (
                (width - clear.bounds().width()) as isize - PADDING,
                BUTTON_Y,
            ),
            "Close",
            None,
            &style.button,
        );
        let edit_palette = Button::new(
            (PADDING, save.bounds().bottom_right().y + PADDING),
            "Palette",
            None,
            &style.button,
        );
        let alert = Alert::new_question(
            &["Are you sure?", "All changes will be lost"],
            "Cancel",
            "Yes",
            width,
            height,
            &style.alert,
        );
        let mut add_frame = IconButton::new(
            FRAME_CONTROL,
            "Add frame",
            Positioning::Center,
            IndexedImage::from_file_contents(include_bytes!("../../assets/icons/add.ici"))
                .unwrap()
                .0,
            &style.icon_button,
        );
        let mut remove_frame = IconButton::new(
            FRAME_CONTROL + (FRAME_CONTROL_SPACING, 0),
            "Remove frame",
            Positioning::Center,
            IndexedImage::from_file_contents(include_bytes!("../../assets/icons/remove.ici"))
                .unwrap()
                .0,
            &style.icon_button,
        );
        let mut copy_frame = IconButton::new(
            FRAME_CONTROL + (FRAME_CONTROL_SPACING * 2, 0),
            "Copy frame",
            Positioning::Center,
            IndexedImage::from_file_contents(include_bytes!("../../assets/icons/copy.ici"))
                .unwrap()
                .0,
            &style.icon_button,
        );
        let mut play_pause = IconButton::new(
            FRAME_CONTROL + (0, FRAME_CONTROL_SPACING),
            "Play/Pause",
            Positioning::Center,
            IndexedImage::from_file_contents(include_bytes!("../../assets/icons/play_pause.ici"))
                .unwrap()
                .0,
            &style.icon_button,
        );
        let mut speed = TextField::new(
            FRAME_CONTROL + (FRAME_CONTROL_SPACING, FRAME_CONTROL_SPACING),
            6,
            Normal,
            (None, Some(40)),
            "0.1",
            &[Decimal],
            &style.text_field,
        );
        let pencil_tool = ToggleIconButton::new(
            (105, BUTTON_Y),
            "Pencil",
            Positioning::CenterBottom,
            IndexedImage::from_file_contents(include_bytes!("../../assets/icons/pencil.ici"))
                .unwrap()
                .0,
            &style.toggle_icon_button,
        );
        let line_tool = ToggleIconButton::new(
            (pencil_tool.bounds().bottom_right().x + PADDING, BUTTON_Y),
            "Line",
            Positioning::CenterBottom,
            IndexedImage::from_file_contents(include_bytes!("../../assets/icons/line.ici"))
                .unwrap()
                .0,
            &style.toggle_icon_button,
        );
        let rect_tool = ToggleIconButton::new(
            (line_tool.bounds().bottom_right().x + PADDING, BUTTON_Y),
            "Rect",
            Positioning::CenterBottom,
            IndexedImage::from_file_contents(include_bytes!("../../assets/icons/rect.ici"))
                .unwrap()
                .0,
            &style.toggle_icon_button,
        );
        let fill_tool = ToggleIconButton::new(
            (rect_tool.bounds().bottom_right().x + PADDING, BUTTON_Y),
            "Fill",
            Positioning::CenterBottom,
            IndexedImage::from_file_contents(include_bytes!("../../assets/icons/fill.ici"))
                .unwrap()
                .0,
            &style.toggle_icon_button,
        );
        let tools = ToggleIconButtonGroup::new(vec![
            (TOOL_PENCIL, pencil_tool),
            (TOOL_LINE, line_tool),
            (TOOL_RECT, rect_tool),
            (TOOL_FILL, fill_tool),
        ]);
        let mut canvas = Canvas::new(
            Coord::new(
                edit_palette.bounds().bottom_right().x + PADDING,
                edit_palette.bounds().top_left().y,
            ),
            (210, 200),
        );
        add_frame.set_state(ElementState::Disabled);
        remove_frame.set_state(ElementState::Disabled);
        copy_frame.set_state(ElementState::Disabled);
        play_pause.set_state(ElementState::Disabled);
        speed.set_state(ElementState::Disabled);
        let frames = match details {
            EditorDetails::Open(path) => {
                let bytes = fs::read(path).expect("Reading image from file");
                let (image, pal) =
                    IndexedImage::from_file_contents(&bytes).expect("Reading image data");
                if pal != FilePalette::Colors {
                    panic!("Currently {pal:?} isn't supported");
                }
                vec![image]
            }
            EditorDetails::New(w, h) => {
                vec![IndexedImage::new(
                    w,
                    h,
                    Palette::default()
                        .colors
                        .iter()
                        .map(|c| c.to_ici())
                        .collect(),
                    vec![0; w as usize * h as usize],
                )
                .unwrap()]
            }
        };

        canvas.set_color_index(1);
        canvas.set_image(frames[0].clone());
        let mut palette = PaletteView::new(
            Coord::new(
                edit_palette.bounds().top_left().x,
                edit_palette.bounds().bottom_right().y + PADDING,
            ),
            (edit_palette.bounds().width(), PALETTE_HEIGHT),
        );
        palette.set_palette(canvas.get_image().get_palette());
        palette.set_color_index(1);
        Box::new(Self {
            result: Nothing,
            clear,
            tools,
            save,
            save_as,
            close,
            edit_palette,
            speed,
            play_pause,
            add_frame,
            remove_frame,
            copy_frame,
            alert,
            palette,
            pending_alert_action: None,
            filepath: None,
            canvas,
            frames,
        })
    }

    fn open_save_as(&mut self) {
        if let Some(path) = file_dialog(&self.filepath, &[("IndexedImage", "ici")]).save_file() {
            self.filepath = Some(path.to_string_lossy().to_string());
            self.save_file();
        }
    }

    fn save_file(&self) {
        if let Some(filepath) = &self.filepath {
            let bytes = self
                .canvas
                .get_image()
                .to_file_contents(&FilePalette::Colors)
                .expect("Unable to save file (converting)");
            fs::write(filepath, bytes).expect("Unable to save file (writing)");
        } else {
            error!("save_file called but no filepath set")
        }
    }
}

impl Scene<SceneResult, SceneName> for Editor {
    fn render(&self, graphics: &mut Graphics, mouse_xy: Coord) {
        graphics.clear(BACKGROUND);

        let name = if let Some(path) = &self.filepath {
            PathBuf::from(path)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .map(|s| {
                    if s.ends_with(".ici") || s.ends_with(".ica") {
                        s.chars().take(s.chars().count() - 4).collect()
                    } else {
                        s
                    }
                })
                .unwrap_or(CORRUPT.to_string())
        } else {
            UNTITLED.to_string()
        };
        graphics.draw_text(
            &name,
            TextPos::px(NAME),
            (WHITE, Normal, WrappingStrategy::Ellipsis(38)),
        );
        graphics.draw_line((0, NAME_LINE_Y), (WIDTH as isize, NAME_LINE_Y), LIGHT_GRAY);

        // self.speed.render(graphics, mouse_xy);
        // self.add_frame.render(graphics, mouse_xy);
        // self.remove_frame.render(graphics, mouse_xy);
        // self.copy_frame.render(graphics, mouse_xy);
        // self.play_pause.render(graphics, mouse_xy);
        self.tools.render(graphics, mouse_xy);
        self.save.render(graphics, mouse_xy);
        self.save_as.render(graphics, mouse_xy);
        self.clear.render(graphics, mouse_xy);
        self.close.render(graphics, mouse_xy);
        self.palette.render(graphics, mouse_xy);
        self.edit_palette.render(graphics, mouse_xy);
        self.canvas.render(graphics, mouse_xy);
    }

    fn on_key_up(&mut self, key: VirtualKeyCode, _: &Vec<&VirtualKeyCode>) {
        self.speed.on_key_press(key);
    }

    fn on_mouse_down(&mut self, xy: Coord, button: MouseButton, _: &Vec<&VirtualKeyCode>) {
        if button != MouseButton::Left {
            return;
        }
        self.canvas.on_mouse_down(xy);
    }

    fn on_mouse_up(&mut self, xy: Coord, button: MouseButton, _: &Vec<&VirtualKeyCode>) {
        if button != MouseButton::Left {
            return;
        }
        if let Some(tool_id) = self.tools.on_mouse_click(xy) {
            match tool_id {
                TOOL_PENCIL => self.canvas.set_tool(Tool::Pencil),
                TOOL_LINE => self.canvas.set_tool(Tool::Line),
                TOOL_RECT => self.canvas.set_tool(Tool::Rect),
                TOOL_FILL => self.canvas.set_tool(Tool::Fill),
                _ => {}
            }
        }
        if self.add_frame.on_mouse_click(xy) {}
        if self.remove_frame.on_mouse_click(xy) {}
        if self.copy_frame.on_mouse_click(xy) {}
        if self.close.on_mouse_click(xy) {
            self.result = Pop(None);
        }
        if self.clear.on_mouse_click(xy) {
            self.canvas.clear();
        }
        if self.save.on_mouse_click(xy) {
            if self.filepath.is_some() {
                self.save_file();
            } else {
                self.open_save_as();
            }
        }
        if self.save_as.on_mouse_click(xy) {
            self.open_save_as();
        }
        if self.edit_palette.on_mouse_click(xy) {
            let colors = self
                .canvas
                .get_image()
                .get_palette()
                .iter()
                .map(|c| c.to_color())
                .collect();
            self.result = SceneUpdateResult::Push(false, SceneName::Palette(colors));
        }
        if self.palette.on_mouse_click(xy) {
            self.canvas.set_color_index(self.palette.get_selected_idx());
        }
        self.canvas.on_mouse_up(xy);
    }

    fn on_scroll(&mut self, xy: Coord, y_diff: isize, x_diff: isize, _: &Vec<&VirtualKeyCode>) {}

    fn update(
        &mut self,
        timing: &Timing,
        _: Coord,
        _: &Vec<&VirtualKeyCode>,
    ) -> SceneUpdateResult<SceneResult, SceneName> {
        self.speed.update(timing);
        self.result.clone()
    }

    fn resuming(&mut self, result: Option<SceneResult>) {
        if let Some(result) = result {
            match result {
                SceneResult::Palette(colors) => {
                    let colors: Vec<IciColor> = colors.iter().map(|c| c.to_ici()).collect();
                    self.palette.set_palette(&colors);
                    self.palette.set_color_index(0);
                    self.canvas.set_color_index(0);
                    if let Err(e) = self.canvas.get_mut_image().set_palette(&colors) {
                        panic!(
                            "Failed to update palette: {} (please raise issue on github)",
                            e
                        );
                    }
                }
                _ => {}
            }
        }
        self.result = Nothing;
    }
}
