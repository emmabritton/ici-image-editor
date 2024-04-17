use crate::ui::preview::Preview;
use crate::SceneResult::Simplify;
use crate::{SceneName, SceneResult, HEIGHT, SUR, WIDTH};
use log::error;
use pixels_graphics_lib::prelude::palette::simplify_palette;
use pixels_graphics_lib::prelude::PixelFont::Standard6x7;
use pixels_graphics_lib::prelude::*;
use pixels_graphics_lib::scenes::SceneUpdateResult::Pop;
use pixels_graphics_lib::ui::prelude::relative::LayoutContext;
use pixels_graphics_lib::ui::prelude::*;
use pixels_graphics_lib::{layout, px, render};
use std::collections::HashSet;

pub struct SimplifyDialog {
    result: SUR,
    image: IndexedImage,
    new_image: IndexedImage,
    bg: ShapeCollection,
    cancel: Button,
    current_preview: Preview,
    new_preview: Preview,
    new_label: Label,
    current_label: Label,
    color_count: Label,
    current_color_count: Label,
    new_color_count: Label,
    save: Button,
    update: Button,
    amount_label: Label,
    amount: TextField,
    amount_note: Label,
}

impl SimplifyDialog {
    pub fn new(style: &UiStyle, image: IndexedImage, preview_bg: usize) -> Box<Self> {
        let mut save = Button::new((0, 0), "Save", Some(60), &style.button);
        let mut cancel = Button::new((0, 0), "Cancel", Some(60), &style.button);
        let mut update = Button::new((0, 0), "Refresh", Some(60), &style.button);
        let mut amount = TextField::new(
            (0, 0),
            4,
            Standard6x7,
            (None, None),
            "0",
            &[TextFilter::Numbers],
            &style.text_field,
        );
        let mut color_count = Label::singleline("Colors", (0, 0), WHITE, Standard6x7, WIDTH);
        let mut new_color_count = Label::singleline(
            &image.get_palette().len().to_string(),
            (0, 0),
            WHITE,
            Standard6x7,
            WIDTH,
        );
        let mut current_color_count = Label::singleline(
            &image.get_palette().len().to_string(),
            (0, 0),
            WHITE,
            Standard6x7,
            WIDTH,
        );
        let mut amount_label = Label::singleline("Threshold", (0, 0), WHITE, Standard6x7, WIDTH);
        let mut new_preview = Preview::new(Rect::new((0, 0), (70, 70)), false);
        let mut current_preview = Preview::new(Rect::new((0, 0), (70, 70)), false);
        let mut amount_note =
            Label::singleline("0..1020", (0, 0), WHITE, PixelFont::Standard4x5, WIDTH);
        let mut new_label = Label::singleline("New", (0, 0), WHITE, Standard6x7, WIDTH);
        let mut current_label = Label::singleline("Current", (0, 0), WHITE, Standard6x7, WIDTH);

        new_preview.set_background(preview_bg);
        current_preview.set_background(preview_bg);
        new_preview.set_image(image.clone());
        current_preview.set_image(image.clone());

        let context = LayoutContext::new(style.dialog.bounds.clone());

        layout!(context, current_label, align_top, px!(4));
        layout!(context, new_label, align_top, px!(4));
        layout!(context, color_count, align_left, px!(4));
        layout!(context, current_preview, left_to_right_of color_count);
        layout!(context, current_preview, top_to_bottom_of current_label);
        layout!(context, new_preview, align_right, px!(4));
        layout!(context, new_preview, top_to_top_of current_preview);
        layout!(context, current_label, centerh_to_centerh_of current_preview);
        layout!(context, new_label, centerh_to_centerh_of new_preview);
        layout!(context, color_count, top_to_bottom_of  current_preview, px!(4));
        layout!(context, current_color_count, right_to_right_of  current_preview);
        layout!(context, current_color_count, top_to_top_of color_count);
        layout!(context, new_color_count, right_to_right_of new_preview);
        layout!(context, new_color_count, top_to_top_of color_count);

        layout!(context, update, right_to_right_of  new_color_count);
        layout!(context, update, top_to_bottom_of new_color_count, px!(6));
        layout!(context, amount_label, align_left, px!(4));
        layout!(context, amount_label, bottom_to_bottom_of  update);
        layout!(context, amount, left_to_right_of amount_label, px!(4));
        layout!(context, amount, centerv_to_centerv_of amount_label);
        layout!(context, amount_note, left_to_left_of  amount_label);
        layout!(context, amount_note, top_to_bottom_of amount_label, px!(2));

        layout!(context, save, align_bottom, px!(4));
        layout!(context, save, align_right, px!(4));
        layout!(context, cancel, align_bottom, px!(4));
        layout!(context, cancel, align_left, px!(4));

        Box::new(SimplifyDialog {
            result: SceneUpdateResult::Nothing,
            image: image.clone(),
            new_image: image,
            bg: dialog_background(WIDTH, HEIGHT, &style.dialog),
            cancel,
            color_count,
            current_preview,
            new_preview,
            new_label,
            current_color_count,
            new_color_count,
            save,
            update,
            amount,
            current_label,
            amount_note,
            amount_label,
        })
    }
}

impl SimplifyDialog {
    fn refresh(&mut self) {
        if let Ok(value) = self.amount.content().parse::<u16>() {
            let merged_palette = simplify_palette(self.image.get_palette(), value as usize);
            let new_palette = merged_palette
                .iter()
                .copied()
                .collect::<HashSet<Color>>()
                .into_iter()
                .collect::<Vec<Color>>();
            let mapping = merged_palette
                .iter()
                .map(|color| {
                    new_palette.iter().position(|c| c == color).expect(
                        "missing color during simplification (please raise issue on github)",
                    )
                })
                .collect::<Vec<usize>>();

            let mut pixels = vec![];
            for px in self.image.get_pixels() {
                pixels.push(mapping[*px as usize] as u8)
            }

            match IndexedImage::new(self.image.width(), self.image.height(), new_palette, pixels) {
                Ok(img) => {
                    self.new_image = img;
                    self.new_preview.set_image(self.new_image.clone());
                    self.new_color_count
                        .update_text(&self.new_image.get_palette().len().to_string());
                }
                Err(err) => {
                    error!("Creating new palette image: {err:?}");
                    self.result = Pop(Some(SceneResult::SimplifyError));
                }
            }
        }
    }
}

impl Scene<SceneResult, SceneName> for SimplifyDialog {
    fn render(&self, graphics: &mut Graphics, mouse: &MouseData, _: &FxHashSet<KeyCode>) {
        self.bg.render(graphics);
        render!(
            graphics,
            mouse,
            self.save,
            self.update,
            self.amount,
            self.new_color_count,
            self.current_color_count,
            self.cancel,
            self.color_count,
            self.new_label,
            self.current_label,
            self.amount_label,
            self.amount_note,
            self.current_preview,
            self.new_preview
        );
    }

    fn on_key_up(&mut self, key: KeyCode, _: &MouseData, held_keys: &FxHashSet<KeyCode>) {
        self.amount.on_key_press(key, held_keys);
    }

    fn on_mouse_click(
        &mut self,
        down_at: Coord,
        mouse: &MouseData,
        mouse_button: MouseButton,
        _: &FxHashSet<KeyCode>,
    ) {
        if mouse_button == MouseButton::Left {
            if self.update.on_mouse_click(down_at, mouse.xy) {
                self.refresh();
            }
            if self.cancel.on_mouse_click(down_at, mouse.xy) {
                self.result = Pop(None);
            }
            if self.save.on_mouse_click(down_at, mouse.xy) {
                self.result = Pop(Some(Simplify(self.new_image.clone())));
            }
            self.amount.on_mouse_click(down_at, mouse.xy);
        }
    }

    fn update(
        &mut self,
        timing: &Timing,
        _: &MouseData,
        _: &FxHashSet<KeyCode>,
    ) -> SceneUpdateResult<SceneResult, SceneName> {
        self.amount.update(timing);
        self.result.clone()
    }

    fn is_dialog(&self) -> bool {
        true
    }
}
