use crate::scenes::{file_dialog, import_image, BACKGROUND};
use crate::ui::canvas::{Canvas, Tool};
use crate::ui::palette::PaletteView;
use crate::{DefaultPalette, SceneName, SceneResult, Settings, HEIGHT, SUR, WIDTH};

use pixels_graphics_lib::buffer_graphics_lib::prelude::*;
use pixels_graphics_lib::prelude::*;
use pixels_graphics_lib::scenes::SceneUpdateResult::{Nothing, Pop, Push};
use pixels_graphics_lib::ui::prelude::TextFilter::Decimal;
use pixels_graphics_lib::ui::prelude::*;

use crate::image::resize::resize;
use crate::image::trim::remove_blanks;
use crate::palettes::palette_default;
use crate::scenes::editor_ui::*;
use crate::scenes::resize_dialog::ResizeAnchor;
use crate::ui::edit_history::EditHistory;
use crate::ui::preview::Preview;
use crate::ui::timeline::Timeline;
use crate::SceneName::Resize;
use image_lib::{save_buffer_with_format, ExtendedColorType, ImageFormat};
use log::{debug, error};
use pixels_graphics_lib::prelude::PixelFont::Standard6x7;
use pixels_graphics_lib::ui::layout::relative::LayoutContext;
use pixels_graphics_lib::{layout, px, render};
use serde::{Deserialize, Serialize};
use std::fs;
use std::ops::Add;
use std::path::PathBuf;
use std::time::{Duration, Instant};

const PER_UNDO: u64 = 200;

#[derive(Debug, Clone, Eq, PartialEq)]
enum OneWayAlertAction {
    Double,
    Trim,
    Simplify,
    ChangePalette(Vec<Color>, usize),
    ResizeCanvas,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum DataLossAlertAction {
    New,
    Close,
    Open,
    Import,
}

const PADDING: isize = 4;
const PALETTE_WIDTH: usize = 58;
const PALETTE_HEIGHT: usize = 58;

const UNTITLED: &str = "Untitled";
const CORRUPT: &str = "???";

#[derive(Debug, Clone, PartialEq)]
pub enum EditorDetails {
    Open(PathBuf),
    OpenImage(IndexedImage),
    New(u8, u8, Option<Vec<Color>>),
}

#[derive(Debug, Clone, PartialEq, Default)]
struct SaveData {
    pub path: Option<PathBuf>,
    pub palette: Option<FilePalette>,
    pub index: Option<usize>,
    pub new_file: bool,
    //hack to avoid opening save dialog when save would be called
    //for example if the user clicks the palette details
    pub ignore_next_save: bool,
}

impl SaveData {
    pub fn loaded_image(path: PathBuf, is_animated: bool) -> Self {
        SaveData {
            path: Some(path),
            index: if is_animated { None } else { Some(0) },
            ..SaveData::default()
        }
    }

    pub fn new_image() -> Self {
        SaveData {
            index: Some(0),
            ..SaveData::default()
        }
    }

    pub fn filename(&self) -> String {
        if let Some(path) = &self.path {
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
        }
    }
}

impl SaveData {
    pub fn filters(&self) -> &[(&'static str, &'static str); 1] {
        if self.index.is_some() {
            &[("IndexedImage", "ici")]
        } else {
            &[("AnimatedIndexedImage", "ica")]
        }
    }
}

pub struct Editor {
    result: SUR,
    tools: ToggleIconButtonGroup<Tool>,
    filename: Label,
    speed: TextField,
    play_pause: IconButton,
    add_frame: IconButton,
    remove_frame: IconButton,
    copy_frame: IconButton,
    canvas: Canvas,
    palette: PaletteView,
    last_undo: Instant,
    preview: Preview,
    timeline: Timeline,
    history: EditHistory,
    is_playing: bool,
    next_frame_swap: f64,
    anim_frame_idx: usize,
    prefs: AppPrefs<Settings>,
    error: Option<String>,
    save_data: SaveData,
    data_loss_alert: Alert,
    one_way_alert: Alert,
    data_loss_pending_alert: Option<DataLossAlertAction>,
    one_way_pending_alert: Option<OneWayAlertAction>,
    menubar: MenuBar<MenuId>,
    warning: Option<Alert>,
    alert_style: AlertStyle,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, Eq, PartialEq)]
pub enum BackgroundColors {
    GreyCheck,
    PurpleCheck,
    SolidLightGrey,
    SolidDarkGrey,
    SolidBlack,
    SolidWhite,
}

impl BackgroundColors {
    pub fn menu_items() -> [(MenuId, &'static str); 6] {
        [
            (MenuId::MenuCanvasBackgroundGreyCheck, "Grey check"),
            (MenuId::MenuCanvasBackgroundPurpleCheck, "Purple check"),
            (
                MenuId::MenuCanvasBackgroundSolidLightGrey,
                "Solid light grey",
            ),
            (MenuId::MenuCanvasBackgroundSolidDarkGrey, "Solid dark grey"),
            (MenuId::MenuCanvasBackgroundSolidBlack, "Solid black"),
            (MenuId::MenuCanvasBackgroundSolidWhite, "Solid white"),
        ]
    }

    pub fn idx(&self) -> usize {
        match self {
            BackgroundColors::GreyCheck => 0,
            BackgroundColors::PurpleCheck => 1,
            BackgroundColors::SolidLightGrey => 2,
            BackgroundColors::SolidDarkGrey => 3,
            BackgroundColors::SolidBlack => 4,
            BackgroundColors::SolidWhite => 5,
        }
    }

    pub fn colors(&self) -> (Color, Color) {
        match self {
            BackgroundColors::GreyCheck => (LIGHT_GRAY, DARK_GRAY),
            BackgroundColors::PurpleCheck => (PURPLE, BLACK),
            BackgroundColors::SolidLightGrey => (LIGHT_GRAY, LIGHT_GRAY),
            BackgroundColors::SolidDarkGrey => (DARK_GRAY, DARK_GRAY),
            BackgroundColors::SolidBlack => (BLACK, BLACK),
            BackgroundColors::SolidWhite => (WHITE, WHITE),
        }
    }
}

impl Editor {
    pub fn new(
        width: usize,
        height: usize,
        details: EditorDetails,
        mut prefs: AppPrefs<Settings>,
        default_palette: DefaultPalette,
        style: &UiStyle,
    ) -> Box<Self> {
        let mut menubar = create_menubar(style, &prefs);

        let data_loss_alert = Alert::new_question(
            &["Are you sure?", "All changes will be lost"],
            "Cancel",
            "Yes",
            width,
            height,
            &style.alert,
        );
        let one_way_alert = Alert::new_question(
            &["Are you sure?", "This can not be undone"],
            "Cancel",
            "Yes",
            width,
            height,
            &style.alert,
        );
        let mut add_frame = IconButton::new(
            Coord::default(),
            "ADD FRAME",
            Positioning::LeftCenter,
            IndexedImage::from_file_contents(include_bytes!("../../assets/icons/add.ici"))
                .unwrap()
                .0,
            &style.icon_button,
        );
        let mut remove_frame = IconButton::new(
            Coord::default(),
            "REMOVE FRAME",
            Positioning::LeftCenter,
            IndexedImage::from_file_contents(include_bytes!("../../assets/icons/remove.ici"))
                .unwrap()
                .0,
            &style.icon_button,
        );
        let mut copy_frame = IconButton::new(
            Coord::default(),
            "COPY FRAME",
            Positioning::LeftCenter,
            IndexedImage::from_file_contents(include_bytes!("../../assets/icons/copy.ici"))
                .unwrap()
                .0,
            &style.icon_button,
        );
        let mut play_pause = IconButton::new(
            Coord::default(),
            "PLAY/PAUSE",
            Positioning::LeftCenter,
            IndexedImage::from_file_contents(include_bytes!("../../assets/icons/play_pause.ici"))
                .unwrap()
                .0,
            &style.icon_button,
        );
        let mut speed = TextField::new(
            Coord::default(),
            6,
            PixelFont::Standard6x7,
            (None, Some(40)),
            "0.1",
            &[Decimal],
            &style.text_field,
        );
        let mut pencil_tool = ToggleIconButton::new(
            Coord::default(),
            "PENCIL",
            Positioning::CenterBottom,
            IndexedImage::from_file_contents(include_bytes!("../../assets/icons/pencil.ici"))
                .unwrap()
                .0,
            &style.toggle_icon_button,
        );
        let mut line_tool = ToggleIconButton::new(
            Coord::default(),
            "LINE",
            Positioning::CenterBottom,
            IndexedImage::from_file_contents(include_bytes!("../../assets/icons/line.ici"))
                .unwrap()
                .0,
            &style.toggle_icon_button,
        );
        let mut rect_tool = ToggleIconButton::new(
            Coord::default(),
            "RECT",
            Positioning::CenterBottom,
            IndexedImage::from_file_contents(include_bytes!("../../assets/icons/rect.ici"))
                .unwrap()
                .0,
            &style.toggle_icon_button,
        );
        let mut fill_tool = ToggleIconButton::new(
            Coord::default(),
            "FILL",
            Positioning::CenterBottom,
            IndexedImage::from_file_contents(include_bytes!("../../assets/icons/fill.ici"))
                .unwrap()
                .0,
            &style.toggle_icon_button,
        );
        let mut circle_tool = ToggleIconButton::new(
            Coord::default(),
            "CIRCLE",
            Positioning::CenterBottom,
            IndexedImage::from_file_contents(include_bytes!("../../assets/icons/circle.ici"))
                .unwrap()
                .0,
            &style.toggle_icon_button,
        );
        let mut ellipse_tool = ToggleIconButton::new(
            Coord::default(),
            "ELLIPSE",
            Positioning::CenterBottom,
            IndexedImage::from_file_contents(include_bytes!("../../assets/icons/ellipse.ici"))
                .unwrap()
                .0,
            &style.toggle_icon_button,
        );
        let mut filename = Label::singleline(UNTITLED, (0, 0), WHITE, Standard6x7, WIDTH - 4);
        let mut error = None;
        let mut save_data;
        let frames = match details {
            EditorDetails::Open(path) => {
                let file = path.to_string_lossy().to_string();
                let is_animated = file.contains(".ica");
                save_data = SaveData::loaded_image(path.clone(), is_animated);
                let bytes = fs::read(path.clone()).expect("Reading image from file");
                let (mut images, pal) = if is_animated {
                    let (image, pal) = AnimatedIndexedImage::from_file_contents(&bytes)
                        .expect("Reading animated image data");
                    speed.set_content(&image.get_per_frame().to_string());
                    (image.as_images(), pal)
                } else {
                    let (image, pal) =
                        IndexedImage::from_file_contents(&bytes).expect("Reading image data");
                    (vec![image], pal)
                };
                if pal != FilePalette::Colors {
                    if let DefaultPalette::Palette(_, default) = default_palette {
                        if default.len() == images[0].get_palette().len() {
                            error = Some(format!("Image has palette {pal:?} and default palette was provided but they have a different number of colors: image {}, palette {}", images[0].get_palette().len(), default.len()));
                        } else {
                            for image in &mut images {
                                if let Err(e) = image.set_palette(&default) {
                                    error =
                                        Some(format!("Error when setting default palette: {e}"));
                                }
                            }
                        }
                    }
                }
                save_data.palette = Some(pal);
                filename.update_text(&save_data.filename());
                prefs.data.last_used_dir = path;
                prefs.save();
                images
            }
            EditorDetails::New(w, h, palette) => {
                save_data = SaveData::new_image();
                let colors = palette.unwrap_or_else(|| {
                    if let DefaultPalette::Palette(_, colors) = default_palette {
                        colors
                    } else {
                        palette_default().colors
                    }
                });
                if prefs.data.use_colors {
                    save_data.palette = Some(FilePalette::Colors);
                }
                vec![IndexedImage::new(w, h, colors, vec![0; w as usize * h as usize]).unwrap()]
            }
            EditorDetails::OpenImage(img) => {
                save_data = SaveData::new_image();
                vec![img]
            }
        };

        let mut canvas = Canvas::new(
            Coord::new(
                pencil_tool.bounds().right() + PADDING,
                pencil_tool.bounds().top(),
            ),
            (210, 200),
            prefs.data.background_color.colors(),
        );

        canvas.set_image(frames[0].clone());
        canvas.set_color_index(1);
        let mut palette = PaletteView::new(
            Coord::new(
                filename.bounds().left(),
                filename.bounds().bottom() + PADDING + 6,
            ),
            (PALETTE_WIDTH, PALETTE_HEIGHT),
        );
        palette.set_palette(canvas.get_image().get_palette());
        palette.set_color_index(1);
        let mut preview = Preview::new(Rect::new_with_size((4, 122), 64, 73), true);
        let mut timeline = Timeline::new(Rect::new_with_size((-1, -1), 0, 0));
        timeline.set_frames(frames.clone(), 0);
        let history = EditHistory::new(frames.clone());
        preview.set_image(history.get_current_image().clone());

        match prefs.data.background_color {
            BackgroundColors::SolidLightGrey => preview.set_background(2),
            BackgroundColors::SolidDarkGrey => preview.set_background(3),
            BackgroundColors::SolidBlack => preview.set_background(1),
            _ => preview.set_background(0),
        }

        let context = LayoutContext::new(Rect::new((0, 0), (WIDTH, HEIGHT)));
        layout!(context, menubar, align_top);
        layout!(context, menubar, align_left);

        layout!(context, filename, top_to_bottom_of menubar, px!(4));
        layout!(context, filename, align_left, px!(1));

        layout!(context, palette, align_left, px!(4));
        layout!(context, palette, top_to_bottom_of filename, px!(14));

        layout!(context, pencil_tool, top_to_bottom_of filename, px!(4));
        layout!(context, pencil_tool, left_to_right_of palette, px!(20));
        layout!(context, line_tool, top_to_top_of pencil_tool);
        layout!(context, line_tool, left_to_right_of pencil_tool, px!(4));
        layout!(context, rect_tool, top_to_top_of pencil_tool);
        layout!(context, rect_tool, left_to_right_of line_tool, px!(4));
        layout!(context, fill_tool, top_to_top_of pencil_tool);
        layout!(context, fill_tool, left_to_right_of rect_tool, px!(4));
        layout!(context, circle_tool, top_to_top_of pencil_tool);
        layout!(context, circle_tool, left_to_right_of fill_tool, px!(4));
        layout!(context, ellipse_tool, top_to_top_of pencil_tool);
        layout!(context, ellipse_tool, left_to_right_of circle_tool, px!(4));

        layout!(context, play_pause, align_left, px!(4));
        layout!(context, play_pause, align_bottom, px!(4));

        layout!(context, speed, top_to_top_of play_pause);
        layout!(context, speed, left_to_right_of play_pause, px!(6));
        layout!(context, add_frame, left_to_left_of play_pause);
        layout!(context, add_frame, bottom_to_top_of play_pause, px!(6));
        layout!(context, remove_frame, left_to_right_of add_frame, px!(6));
        layout!(context, remove_frame, top_to_top_of add_frame);
        layout!(context, copy_frame, left_to_right_of remove_frame, px!(6));
        layout!(context, copy_frame, top_to_top_of add_frame);

        layout!(context, preview, left_to_left_of add_frame);
        layout!(context, preview, bottom_to_top_of add_frame, px!(6));

        let tools = ToggleIconButtonGroup::new(vec![
            (Tool::Pencil, pencil_tool),
            (Tool::Line, line_tool),
            (Tool::Rect, rect_tool),
            (Tool::Fill, fill_tool),
            (Tool::Circle, circle_tool),
            (Tool::Ellipse, ellipse_tool),
        ]);

        let mut editor = Self {
            data_loss_alert,
            one_way_alert,
            data_loss_pending_alert: None,
            save_data,
            error,
            timeline,
            menubar,
            filename,
            result: Nothing,
            tools,
            speed,
            play_pause,
            add_frame,
            remove_frame,
            copy_frame,
            palette,
            canvas,
            preview,
            last_undo: Instant::now(),
            history,
            is_playing: false,
            next_frame_swap: 0.0,
            anim_frame_idx: 0,
            prefs,
            warning: None,
            alert_style: style.alert.clone(),
            one_way_pending_alert: None,
        };
        editor.relayout_canvas(frames.len() > 1);
        Box::new(editor)
    }

    fn save(&mut self) {
        if self.save_data.ignore_next_save {
            self.save_data.ignore_next_save = false;
            return;
        }
        if self.save_data.palette.is_none() {
            self.result = Push(
                false,
                SceneName::SavePaletteData(self.save_data.palette.clone()),
            );
            return;
        }
        if self.save_data.path.is_none() || self.save_data.new_file {
            if let Some(path) = file_dialog(
                self.prefs.data.last_used_dir.clone(),
                self.save_data.filters(),
            )
                .save_file()
            {
                self.save_data.path = Some(path.clone());
                self.filename.update_text(&self.save_data.filename());
                self.prefs.data.last_used_dir = path;
                self.prefs.save();
            } else {
                return;
            }
        }
        self.save_file();
        if self.history.frame_count() > 1 {
            self.save_data.index = None;
        } else {
            self.save_data.index = Some(0);
        }
    }

    fn save_file(&self) {
        if let Some(filepath) = &self.save_data.path {
            if let Some(palette) = &self.save_data.palette {
                if let Some(idx) = self.save_data.index {
                    let bytes = self
                        .history
                        .get_image(idx)
                        .to_file_contents(palette)
                        .expect("Unable to save ici file (converting)");
                    fs::write(filepath, bytes).expect("Unable to save ici file (writing)");
                } else {
                    let frames = self.history.get_images();
                    let pixels = frames
                        .iter()
                        .flat_map(|i| i.get_pixels().to_vec())
                        .collect();
                    let image = AnimatedIndexedImage::new(
                        frames[0].width(),
                        frames[1].height(),
                        self.speed.content().parse::<f64>().unwrap_or(1.0),
                        frames.len() as u8,
                        frames[0].get_palette().to_vec(),
                        pixels,
                        PlayType::Loops,
                    )
                        .unwrap();
                    let bytes = image
                        .to_file_contents(palette)
                        .expect("Unable to save ica file (converting)");
                    fs::write(filepath, bytes).expect("Unable to save ica file (writing)");
                }
            } else {
                error!("Missing save_data.palette")
            }
        } else {
            error!("Missing save_data.path")
        }
    }

    fn relayout_canvas(&mut self, show_timeline: bool) {
        let state = self.canvas.get_usage_state();
        if show_timeline {
            let image_height = self.history.get_current_image().height() as usize;
            self.timeline = Timeline::new(Rect::new_with_size(
                (
                    self.preview.bounds().right() + PADDING,
                    self.play_pause.bounds().bottom() - (image_height as isize),
                ),
                200,
                image_height + 4,
            ));
            self.canvas = Canvas::new(
                Coord::new(
                    self.tools.get(Tool::Pencil).bounds().left(),
                    self.tools.get(Tool::Pencil).bounds().bottom() + PADDING,
                ),
                (
                    (WIDTH as isize - self.tools.get(Tool::Pencil).bounds().left()) as usize,
                    (self.timeline.bounds().top()
                        - self.tools.get(Tool::Pencil).bounds().bottom()
                        - PADDING) as usize,
                ),
                self.prefs.data.background_color.colors(),
            );
            self.play_pause.set_state(ViewState::Normal);
        } else {
            self.timeline = Timeline::new(Rect::new_with_size((-1, -1), 0, 0));
            self.canvas = Canvas::new(
                Coord::new(
                    self.tools.get(Tool::Pencil).bounds().left(),
                    self.tools.get(Tool::Pencil).bounds().bottom() + PADDING,
                ),
                (
                    (WIDTH as isize - self.tools.get(Tool::Pencil).bounds().left()) as usize,
                    (HEIGHT as isize
                        - self.tools.get(Tool::Pencil).bounds().bottom()
                        - PADDING
                        - PADDING) as usize,
                ),
                self.prefs.data.background_color.colors(),
            );
            self.play_pause.set_state(ViewState::Disabled);
        }
        self.canvas.set_color_index(self.palette.get_selected_idx());
        self.canvas
            .set_image(self.history.get_current_image().clone());
        self.timeline
            .set_frames(self.history.get_images(), self.history.active_frame());
        self.canvas.set_usage_state(state);
    }

    fn undo(&mut self) {
        self.history.undo().unwrap();
        self.last_undo = Instant::now().add(Duration::from_millis(PER_UNDO));
        self.image_update();
    }

    fn redo(&mut self) {
        self.history.redo().unwrap();
        self.last_undo = Instant::now().add(Duration::from_millis(PER_UNDO));
        self.image_update();
    }

    fn image_update(&mut self) {
        self.canvas
            .set_image(self.history.get_current_image().clone());
        self.timeline
            .set_frames(self.history.get_images(), self.history.active_frame());
        self.preview
            .set_image(self.history.get_current_image().clone());
        self.palette.set_palette(self.canvas.get_palette());
    }

    fn open_file(&mut self) {
        if let Some(path) = file_dialog(
            self.prefs.data.last_used_dir.clone(),
            &[("IndexedImage", "ici"), ("AnimatedIndexedImage", "ica")],
        )
            .pick_file()
        {
            self.result = Push(false, SceneName::Editor(EditorDetails::Open(path)));
        }
    }

    fn open_resize(&mut self) {
        self.result = Push(
            false,
            Resize(
                self.history.get_current_image().width(),
                self.history.get_current_image().height(),
            ),
        );
    }

    fn double_size(&mut self) {
        if self.canvas.get_image().width() > 32 || self.canvas.get_image().height() > 32 {
            self.warning = Some(Alert::new_warning(
                &["Can not double size, image too large"],
                WIDTH,
                HEIGHT,
                &self.alert_style,
            ));
            return;
        }
        match self.history.get_current_image().scale(Scaling::nn_double()) {
            Ok(image) => {
                debug!("Image size doubled to {}x{}", image.width(), image.height());
                self.history = EditHistory::new(vec![image]);
                self.image_update();
            }
            Err(err) => {
                error!("Failed to resize image: {err:?}");
                self.warning = Some(Alert::new_warning(
                    &["Couldn't resize image"],
                    WIDTH,
                    HEIGHT,
                    &self.alert_style,
                ));
            }
        }
    }

    fn trim(&mut self) {
        match remove_blanks(self.history.get_current_image().clone()) {
            Ok(image) => {
                debug!("Canvas resized to {}x{}", image.width(), image.height());
                self.history = EditHistory::new(vec![image]);
                self.image_update();
            }
            Err(err) => {
                error!("Error trimming: {err:?}");
                self.error = Some(format!("Error trimming canvas: {err}"));
            }
        }
    }

    fn resize(&mut self, w: u8, h: u8, anchor: ResizeAnchor) {
        match resize(w, h, anchor, self.history.get_current_image()) {
            Ok(img) => {
                debug!("Canvas resized to {w}x{h}");
                self.history = EditHistory::new(vec![img]);
                self.image_update();
            }
            Err(e) => {
                error!("Error resizing canvas: {e:?}");
                self.warning = Some(Alert::new_warning(
                    &["Error resizing canvas"],
                    WIDTH,
                    HEIGHT,
                    &self.alert_style,
                ));
            }
        }
    }

    fn export(&mut self, format: ImageFormat) {
        if let Some(path) = file_dialog(
            self.prefs.data.last_used_dir.clone(),
            &[(image_format_name(format), format.extensions_str()[0])],
        )
            .save_file()
        {
            let image = Image::from_indexed(self.history.get_current_image());
            if image.is_transparent() && format == ImageFormat::Jpeg {
                self.warning = Some(Alert::new_warning(
                    &["Can not export image: JPEG", "doesn't support transparency"],
                    WIDTH,
                    HEIGHT,
                    &self.alert_style,
                ));
                return;
            }
            let mut values: Vec<u8> = image.pixels().iter().flat_map(|c| c.as_array()).collect();
            if format == ImageFormat::Jpeg {
                values = values
                    .chunks_exact(4)
                    .flat_map(|arr| [arr[0], arr[1], arr[2]])
                    .collect()
            }
            if let Err(e) = save_buffer_with_format(
                &path,
                &values,
                image.width() as u32,
                image.height() as u32,
                color_type(format),
                format,
            ) {
                error!("Error saving image to {path:?}: {e:?}");
                self.warning = Some(Alert::new_warning(
                    &["Error saving image"],
                    WIDTH,
                    HEIGHT,
                    &self.alert_style,
                ));
            }
        }
    }

    fn import(&mut self) {
        if let Some(result) = import_image(&self.alert_style, &mut self.prefs) {
            match result {
                Ok(img) => {
                    debug!("New image imported");
                    self.history = EditHistory::new(vec![img]);
                    self.image_update();
                    self.palette.set_color_index(0);
                    self.canvas.set_color_index(0);
                }
                Err(alert) => self.warning = Some(alert),
            }
        }
    }

    fn change_palette(&mut self, colors: &[Color], selected: usize) {
        if let Err(e) = self.history.add_palette_change(colors) {
            panic!("Failed to update palette: (please raise issue on github) {e:?}");
        }
        self.image_update();
        self.palette.set_color_index(selected as u8);
        self.canvas.set_color_index(selected as u8);
    }

    fn set_bg_color(&mut self, color: BackgroundColors) {
        self.prefs.data.background_color = color;
        self.prefs.save();
        self.canvas
            .set_trans_background_colors(self.prefs.data.background_color.colors());
        self.menubar
            .uncheck_all_children(MenuId::MenuCanvasBackground);
        match color {
            BackgroundColors::GreyCheck => {
                self.menubar
                    .set_checked(MenuId::MenuCanvasBackgroundGreyCheck, true);
            }
            BackgroundColors::PurpleCheck => {
                self.menubar
                    .set_checked(MenuId::MenuCanvasBackgroundPurpleCheck, true);
            }
            BackgroundColors::SolidLightGrey => {
                self.menubar
                    .set_checked(MenuId::MenuCanvasBackgroundSolidLightGrey, true);
                self.preview.set_background(2);
            }
            BackgroundColors::SolidDarkGrey => {
                self.menubar
                    .set_checked(MenuId::MenuCanvasBackgroundSolidDarkGrey, true);
                self.preview.set_background(3);
            }
            BackgroundColors::SolidBlack => {
                self.menubar
                    .set_checked(MenuId::MenuCanvasBackgroundSolidBlack, true);
                self.preview.set_background(1);
            }
            BackgroundColors::SolidWhite => {
                self.menubar
                    .set_checked(MenuId::MenuCanvasBackgroundSolidWhite, true);
                self.preview.set_background(0);
            }
        }
    }
}

fn color_type(format: ImageFormat) -> ExtendedColorType {
    if format == ImageFormat::Jpeg {
        ExtendedColorType::Rgb8
    } else {
        ExtendedColorType::Rgba8
    }
}

fn image_format_name(format: ImageFormat) -> &'static str {
    match format {
        ImageFormat::Png => "PNG",
        ImageFormat::Jpeg => "JPEG",
        ImageFormat::Tga => "TGA",
        ImageFormat::Bmp => "BMP",
        ImageFormat::Ico => "Icon",
        _ => "",
    }
}

impl Scene<SceneResult, SceneName> for Editor {
    fn render(&self, graphics: &mut Graphics, mouse: &MouseData, _: &FxHashSet<KeyCode>) {
        graphics.clear(BACKGROUND);

        if let Some(msg) = &self.error {
            graphics.clear(BACKGROUND.with_brightness(0.3));
            graphics.draw_text(
                &format!("{msg}\nPress escape to close"),
                TextPos::px(coord!(WIDTH / 2, HEIGHT / 2)),
                (
                    RED,
                    Standard6x7,
                    WrappingStrategy::SpaceBeforeCol(
                        Standard6x7.px_to_cols((WIDTH as f32 * 0.8).floor() as usize),
                    ),
                    Positioning::Center,
                ),
            );
            return;
        }

        graphics.draw_line(
            (0, self.filename.bounds().bottom()),
            (WIDTH as isize, self.filename.bounds().bottom()),
            LIGHT_GRAY,
        );
        let text = if let Some(pal) = &self.save_data.palette {
            match pal {
                FilePalette::NoData => String::from("DON'T INCLUDE"),
                FilePalette::ID(id) => format!("ID: {id}"),
                FilePalette::Name(name) => format!("\"{name}\""),
                FilePalette::Colors => String::from("INCL AS COLORS"),
            }
        } else {
            String::from("-")
        };
        graphics.draw_text(
            &text,
            TextPos::px(self.filename.bounds().bottom_left() + (0, 4)),
            (WHITE, PixelFont::Standard4x5, WrappingStrategy::Cutoff(14)),
        );

        graphics.draw_text(
            &format!(
                "{}x{}",
                self.canvas.get_image().width(),
                self.canvas.get_image().height()
            ),
            TextPos::px(coord!(WIDTH, self.filename.bounds().bottom() as usize + 2)),
            (WHITE, PixelFont::Standard4x5, Positioning::RightTop),
        );

        render!(
            graphics,
            mouse,
            self.canvas,
            self.speed,
            self.copy_frame,
            self.remove_frame,
            self.add_frame,
            self.play_pause,
            self.tools,
            self.palette,
            self.preview,
            self.timeline,
            self.filename,
            self.menubar,
        );

        if self.data_loss_pending_alert.is_some() {
            self.data_loss_alert.render(graphics, mouse);
        }

        if self.one_way_pending_alert.is_some() {
            self.one_way_alert.render(graphics, mouse);
        }

        if let Some(alert) = &self.warning {
            alert.render(graphics, mouse);
        }
    }

    fn on_key_down(&mut self, key: KeyCode, _: &MouseData, held: &FxHashSet<KeyCode>) {
        if self.error.is_some() && key == KeyCode::Escape {
            self.result = Pop(None);
            return;
        }
        if self.data_loss_pending_alert.is_some() || self.one_way_pending_alert.is_some() {
            return;
        }
        if self.warning.is_some() && matches!(key, KeyCode::Escape | KeyCode::Enter) {
            self.warning = None;
        }
        if self.last_undo < Instant::now() {
            if key == KeyCode::KeyZ
                && !held.contains(&KeyCode::ShiftLeft)
                && !held.contains(&KeyCode::ShiftRight)
                && (held.contains(&KeyCode::ControlLeft)
                || held.contains(&KeyCode::SuperLeft)
                || held.contains(&KeyCode::ControlRight)
                || held.contains(&KeyCode::SuperRight))
            {
                self.undo();
            }
            if ((key == KeyCode::KeyZ
                && (held.contains(&KeyCode::ShiftLeft) || held.contains(&KeyCode::ShiftRight)))
                || key == KeyCode::KeyY)
                && (held.contains(&KeyCode::ControlLeft)
                || held.contains(&KeyCode::SuperLeft)
                || held.contains(&KeyCode::ControlRight)
                || held.contains(&KeyCode::SuperRight))
            {
                self.redo();
            }
        }
    }

    fn on_key_up(&mut self, key: KeyCode, _: &MouseData, held: &FxHashSet<KeyCode>) {
        if self.data_loss_pending_alert.is_some()
            || self.one_way_pending_alert.is_some()
            || self.warning.is_some()
        {
            return;
        }
        self.speed.on_key_press(key, held);

        if !self.speed.is_focused() {
            let shift_down =
                held.contains(&KeyCode::ShiftLeft) || held.contains(&KeyCode::ShiftRight);
            if shift_down && key == KeyCode::ArrowUp {
                self.history.move_up().unwrap();
                self.image_update();
            } else if shift_down && key == KeyCode::ArrowDown {
                self.history.move_down().unwrap();
                self.image_update();
            } else if shift_down && key == KeyCode::ArrowLeft {
                self.history.move_left().unwrap();
                self.image_update();
            } else if shift_down && key == KeyCode::ArrowRight {
                self.history.move_right().unwrap();
                self.image_update();
            }
        }
    }

    fn on_mouse_click(
        &mut self,
        down_at: Coord,
        mouse: &MouseData,
        button: MouseButton,
        keys: &FxHashSet<KeyCode>,
    ) {
        if button != MouseButton::Left {
            return;
        }
        if self.error.is_some() {
            return;
        }
        if let Some(pending) = self.data_loss_pending_alert.clone() {
            if let Some(result) = self.data_loss_alert.on_mouse_click(down_at, mouse.xy) {
                if result == AlertResult::Positive {
                    match pending {
                        DataLossAlertAction::New => {
                            self.result = Push(
                                false,
                                SceneName::NewImage(Some(
                                    self.canvas.get_image().get_palette().to_vec(),
                                )),
                            )
                        }
                        DataLossAlertAction::Close => self.result = Pop(None),
                        DataLossAlertAction::Open => self.open_file(),
                        DataLossAlertAction::Import => self.import(),
                    }
                }
                self.data_loss_pending_alert = None;
            }
        }
        if let Some(pending) = self.one_way_pending_alert.clone() {
            if let Some(result) = self.one_way_alert.on_mouse_click(down_at, mouse.xy) {
                if result == AlertResult::Positive {
                    match pending {
                        OneWayAlertAction::ResizeCanvas => self.open_resize(),
                        OneWayAlertAction::Double => self.double_size(),
                        OneWayAlertAction::Trim => self.trim(),
                        OneWayAlertAction::Simplify => {
                            self.result = Push(
                                false,
                                SceneName::Simplify(
                                    self.history.get_current_image().clone(),
                                    self.preview.selected_background(),
                                ),
                            )
                        }
                        OneWayAlertAction::ChangePalette(colors, selected) => {
                            self.change_palette(&colors, selected)
                        }
                    }
                }
                self.one_way_pending_alert = None;
            }
            return;
        }
        if let Some(alert) = &mut self.warning {
            if alert.on_mouse_click(down_at, mouse.xy).is_some() {
                self.warning = None;
            }
            return;
        }
        if self.menubar.is_expanded() {
            if let Some(id) = self.menubar.on_mouse_click(down_at, mouse.xy) {
                match id {
                    MenuId::MenuFileQuit => {
                        if self.history.is_empty() {
                            self.result = Pop(None);
                        } else {
                            self.data_loss_pending_alert = Some(DataLossAlertAction::Close);
                        }
                    }
                    MenuId::MenuImageClear => {
                        self.history.add_clear().unwrap();
                        self.image_update();
                    }
                    MenuId::MenuFileSave => {
                        self.save_data.new_file = false;
                        if keys.contains(&KeyCode::ShiftLeft)
                            || keys.contains(&KeyCode::ShiftRight)
                            || self.history.frame_count() == 1
                        {
                            self.save_data.index = Some(self.history.active_frame())
                        } else {
                            self.save_data.index = None;
                        }
                        self.save();
                    }
                    MenuId::MenuFileSaveAs => {
                        self.save_data.new_file = true;
                        if keys.contains(&KeyCode::ShiftLeft)
                            || keys.contains(&KeyCode::ShiftRight)
                            || self.history.frame_count() == 1
                        {
                            self.save_data.index = Some(self.history.active_frame())
                        } else {
                            self.save_data.index = None;
                        }
                        self.save();
                    }
                    MenuId::MenuEditUndo => self.undo(),
                    MenuId::MenuEditRedo => self.redo(),
                    MenuId::MenuPaletteEdit => {
                        let colors = self.canvas.get_image().get_palette().to_vec();
                        self.result = Push(
                            false,
                            SceneName::Palette(colors, self.palette.get_selected_idx() as usize),
                        );
                    }
                    MenuId::MenuPaletteMode => {
                        self.save_data.ignore_next_save = true;
                        self.result = Push(
                            false,
                            SceneName::SavePaletteData(self.save_data.palette.clone()),
                        )
                    }
                    MenuId::MenuFileNew => {
                        if self.history.is_empty() {
                            self.result = Push(
                                false,
                                SceneName::NewImage(Some(
                                    self.canvas.get_image().get_palette().to_vec(),
                                )),
                            );
                        } else {
                            self.data_loss_pending_alert = Some(DataLossAlertAction::New);
                        }
                    }
                    MenuId::MenuFileOpen => {
                        if self.history.is_empty() {
                            self.open_file();
                        } else {
                            self.data_loss_pending_alert = Some(DataLossAlertAction::Open);
                        }
                    }
                    MenuId::MenuImageFlipH => {
                        self.history.flip_h().unwrap();
                        self.image_update();
                    }
                    MenuId::MenuImageFlipV => {
                        self.history.flip_v().unwrap();
                        self.image_update();
                    }
                    MenuId::MenuImageRotCw90 => {
                        self.history.rotate_cw_90().unwrap();
                        self.image_update();
                    }
                    MenuId::MenuImageRotCw180 => {
                        self.history.rotate_cw_180().unwrap();
                        self.image_update();
                    }
                    MenuId::MenuImageRotCw270 => {
                        self.history.rotate_cw_270().unwrap();
                        self.image_update();
                    }
                    MenuId::MenuImageRotCcw90 => {
                        self.history.rotate_ccw_90().unwrap();
                        self.image_update();
                    }
                    MenuId::MenuImageRotCcw180 => {
                        self.history.rotate_ccw_180().unwrap();
                        self.image_update();
                    }
                    MenuId::MenuImageRotCcw270 => {
                        self.history.rotate_ccw_270().unwrap();
                        self.image_update();
                    }
                    MenuId::MenuImageShiftUp => {
                        self.history.move_up().unwrap();
                        self.image_update();
                    }
                    MenuId::MenuImageShiftDown => {
                        self.history.move_down().unwrap();
                        self.image_update();
                    }
                    MenuId::MenuImageShiftLeft => {
                        self.history.move_left().unwrap();
                        self.image_update();
                    }
                    MenuId::MenuImageShiftRight => {
                        self.history.move_right().unwrap();
                        self.image_update();
                    }
                    MenuId::MenuFile => {}
                    MenuId::MenuEdit => {}
                    MenuId::MenuImage => {}
                    MenuId::MenuPalette => {}
                    MenuId::MenuImageRotCw => {}
                    MenuId::MenuImageRotCcw => {}
                    MenuId::MenuImageShift => {}
                    MenuId::MenuCanvas => {}
                    MenuId::MenuCanvasResize => {
                        if self.history.is_empty() {
                            self.open_resize();
                        } else {
                            self.one_way_pending_alert = Some(OneWayAlertAction::ResizeCanvas);
                        }
                    }
                    MenuId::MenuCanvasTrim => {
                        if self.history.is_empty() {
                            self.trim();
                        } else {
                            self.one_way_pending_alert = Some(OneWayAlertAction::Trim);
                        }
                    }
                    MenuId::MenuImageDoubleSize => {
                        if self.history.is_empty() {
                            self.double_size();
                        } else {
                            self.one_way_pending_alert = Some(OneWayAlertAction::Double);
                        }
                    }
                    MenuId::MenuFileExportPng => self.export(ImageFormat::Png),
                    MenuId::MenuFileExportJpeg => self.export(ImageFormat::Jpeg),
                    MenuId::MenuFileExportBmp => self.export(ImageFormat::Bmp),
                    MenuId::MenuFileExportIco => self.export(ImageFormat::Ico),
                    MenuId::MenuFileExportTga => self.export(ImageFormat::Tga),
                    MenuId::MenuFileExport => {}
                    MenuId::MenuFileImport => {
                        if self.history.is_empty() {
                            self.import();
                        } else {
                            self.data_loss_pending_alert = Some(DataLossAlertAction::Import);
                        }
                    }
                    MenuId::MenuPaletteSimplify => {
                        if self.history.is_empty() {
                            self.result = Push(
                                false,
                                SceneName::Simplify(
                                    self.history.get_current_image().clone(),
                                    self.preview.selected_background(),
                                ),
                            )
                        } else {
                            self.one_way_pending_alert = Some(OneWayAlertAction::Simplify);
                        }
                    }
                    MenuId::MenuCanvasBackground => {}
                    MenuId::MenuCanvasBackgroundGreyCheck => {
                        self.set_bg_color(BackgroundColors::GreyCheck)
                    }
                    MenuId::MenuCanvasBackgroundPurpleCheck => {
                        self.set_bg_color(BackgroundColors::PurpleCheck)
                    }
                    MenuId::MenuCanvasBackgroundSolidLightGrey => {
                        self.set_bg_color(BackgroundColors::SolidLightGrey)
                    }
                    MenuId::MenuCanvasBackgroundSolidDarkGrey => {
                        self.set_bg_color(BackgroundColors::SolidDarkGrey)
                    }
                    MenuId::MenuCanvasBackgroundSolidWhite => {
                        self.set_bg_color(BackgroundColors::SolidWhite)
                    }
                    MenuId::MenuCanvasBackgroundSolidBlack => {
                        self.set_bg_color(BackgroundColors::SolidBlack)
                    }
                }
            }
            return;
        }
        if let Some(tool) = self.tools.on_mouse_click(down_at, mouse.xy) {
            self.canvas.set_tool(tool)
        }
        if self.play_pause.on_mouse_click(down_at, mouse.xy) {
            if self.is_playing {
                self.is_playing = false;
                self.add_frame.set_state(ViewState::Normal);
                self.speed.set_state(ViewState::Normal);
                self.remove_frame.set_state(ViewState::Normal);
                self.copy_frame.set_state(ViewState::Normal);
                self.timeline.set_state(ViewState::Normal);
                self.palette.set_state(ViewState::Normal);
                self.canvas.set_state(ViewState::Normal);
                self.menubar.set_state(MenuId::MenuEdit, ViewState::Normal);
                self.menubar.set_state(MenuId::MenuImage, ViewState::Normal);
                self.menubar
                    .set_state(MenuId::MenuPalette, ViewState::Normal);
            } else {
                self.is_playing = true;
                self.anim_frame_idx = 0;
                self.next_frame_swap = self.speed.content().parse::<f64>().unwrap_or(1.0);
                self.add_frame.set_state(ViewState::Disabled);
                self.speed.set_state(ViewState::Disabled);
                self.remove_frame.set_state(ViewState::Disabled);
                self.copy_frame.set_state(ViewState::Disabled);
                self.timeline.set_state(ViewState::Disabled);
                self.palette.set_state(ViewState::Disabled);
                self.canvas.set_state(ViewState::Disabled);
                self.menubar
                    .set_state(MenuId::MenuEdit, ViewState::Disabled);
                self.menubar
                    .set_state(MenuId::MenuImage, ViewState::Disabled);
                self.menubar
                    .set_state(MenuId::MenuPalette, ViewState::Disabled);
            }
        }
        if self.add_frame.on_mouse_click(down_at, mouse.xy) {
            self.history.add_blank_frame().unwrap();
            self.timeline
                .set_frames(self.history.get_images(), self.history.active_frame());
            if self.history.frame_count() == 2 {
                self.relayout_canvas(true);
            }
            self.save_data.index = None;
        }
        if self.remove_frame.on_mouse_click(down_at, mouse.xy) {
            self.history.remove_frame().unwrap();
            self.timeline
                .set_frames(self.history.get_images(), self.history.active_frame());
            if self.history.frame_count() == 1 {
                self.relayout_canvas(false);
                self.save_data.index = Some(0);
            }
        }
        if self.copy_frame.on_mouse_click(down_at, mouse.xy) {
            self.history.add_duplicate_frame().unwrap();
            self.timeline
                .set_frames(self.history.get_images(), self.history.active_frame());
            if self.history.frame_count() == 2 {
                self.relayout_canvas(true);
            }
            self.save_data.index = None;
        }
        self.speed.on_mouse_click(down_at, mouse.xy);
        if self.palette.on_mouse_click(mouse.xy) {
            self.canvas.set_color_index(self.palette.get_selected_idx());
        }
        self.canvas.on_mouse_up(mouse.xy, &mut self.history);
        let background_color = self.preview.on_mouse_click(mouse.xy);
        self.timeline.set_background(background_color);
        if let Some(frame) = self.timeline.on_mouse_click(mouse.xy) {
            self.history.add_frame_select(frame).unwrap();
        }

        self.canvas
            .set_image(self.history.get_current_image().clone());
        self.preview.set_image(self.canvas.get_image().clone());
        self.timeline
            .update_frame(self.history.get_current_image().clone());
    }

    fn on_scroll(
        &mut self,
        mouse: &MouseData,
        x_diff: isize,
        y_diff: isize,
        _: &FxHashSet<KeyCode>,
    ) {
        self.palette.on_scroll(mouse.xy, y_diff);
        self.timeline.on_scroll(mouse.xy, x_diff);
    }

    fn update(
        &mut self,
        timing: &Timing,
        mouse: &MouseData,
        held: &FxHashSet<KeyCode>,
    ) -> SceneUpdateResult<SceneResult, SceneName> {
        self.speed.update(timing);

        self.canvas.set_shift_pressed(held.contains(&KeyCode::ShiftLeft) || held.contains(&KeyCode::ShiftRight));

        if self.is_playing {
            self.next_frame_swap -= timing.fixed_time_step;
            if self.next_frame_swap <= 0.0 {
                self.next_frame_swap = self.speed.content().parse::<f64>().unwrap_or(1.0);
                self.anim_frame_idx += 1;
                if self.anim_frame_idx >= self.history.frame_count() {
                    self.anim_frame_idx = 0;
                }
                let image = self.history.get_image(self.anim_frame_idx);
                self.canvas.set_image(image.clone());
                self.preview.set_image(image.clone());
                self.timeline.set_active(self.anim_frame_idx)
            }
        } else {
            let frame_count = self.history.frame_count();
            if frame_count == 1 {
                self.remove_frame.set_state(ViewState::Disabled);
            } else {
                self.remove_frame.set_state(ViewState::Normal);
            }
            if frame_count > 254 {
                self.add_frame.set_state(ViewState::Disabled);
                self.copy_frame.set_state(ViewState::Disabled);
            } else {
                self.add_frame.set_state(ViewState::Normal);
                self.copy_frame.set_state(ViewState::Normal);
            }

            if mouse.is_down(MouseButton::Left).is_some()
                && self.data_loss_pending_alert.is_none()
                && self.one_way_pending_alert.is_none()
                && self.warning.is_none()
                && !self.menubar.is_expanded()
                && self.canvas.on_mouse_down(mouse.xy, &mut self.history)
            {
                self.canvas
                    .set_image(self.history.get_current_image().clone());
                self.preview.set_image(self.canvas.get_image().clone());
                self.timeline
                    .update_frame(self.history.get_current_image().clone());
                if let Some(c) = self.history.is_first_event() {
                    match (self.preview.selected_background(), c) {
                        (0, WHITE) => {
                            self.preview.set_background(1);
                            self.timeline.set_background(BLACK);
                            self.canvas
                                .set_trans_background_colors(BackgroundColors::SolidBlack.colors());
                        }
                        (1, BLACK) => {
                            self.preview.set_background(0);
                            self.timeline.set_background(WHITE);
                            self.canvas
                                .set_trans_background_colors(BackgroundColors::SolidWhite.colors());
                        }
                        _ => {}
                    }
                }
            }
        }

        if self.history.frame_count() == 1 {
            self.menubar
                .set_state(MenuId::MenuCanvas, ViewState::Normal);
            self.menubar
                .set_state(MenuId::MenuImageDoubleSize, ViewState::Normal);
            self.menubar
                .set_state(MenuId::MenuPaletteSimplify, ViewState::Normal);
        } else {
            self.menubar
                .set_state(MenuId::MenuCanvas, ViewState::Disabled);
            self.menubar
                .set_state(MenuId::MenuImageDoubleSize, ViewState::Disabled);
            self.menubar
                .set_state(MenuId::MenuPaletteSimplify, ViewState::Disabled);
        }

        self.menubar.on_mouse_move(mouse.xy);

        self.result.clone()
    }

    fn resuming(&mut self, result: Option<SceneResult>) {
        if let Some(result) = result {
            match result {
                SceneResult::ResizeData(w, h, anchor) => {
                    if self.history.get_current_image().width() != w
                        || self.history.get_current_image().height() != h
                    {
                        self.resize(w, h, anchor);
                    }
                }
                SceneResult::SavePaletteData(fp) => {
                    self.save_data.palette = Some(fp);
                    self.save();
                }
                SceneResult::Palette(colors, selected) => {
                    if colors.len()
                        <= self
                        .history
                        .get_current_image()
                        .min_palette_size_supported() as usize
                    {
                        self.one_way_pending_alert =
                            Some(OneWayAlertAction::ChangePalette(colors, selected));
                    } else {
                        self.change_palette(&colors, selected);
                    }
                }
                SceneResult::SimplifyError => {
                    self.warning = Some(Alert::new_warning(
                        &["An error occurred when", "simplifying the palette"],
                        WIDTH,
                        HEIGHT,
                        &self.alert_style,
                    ));
                }
                SceneResult::Simplify(img) => {
                    debug!("Palette simplified to {:?}", img.get_palette());
                    self.history = EditHistory::new(vec![img]);
                    self.image_update();
                    self.palette.set_color_index(0);
                    self.canvas.set_color_index(0);
                }
            }
        }
        self.result = Nothing;
    }
}
