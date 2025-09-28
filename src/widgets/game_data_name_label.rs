use raphael_data::{
    Consumable, Ingredient, Locale, Recipe, get_item_name, get_stellar_mission_name,
};
use raphael_translations::t;

// The game seems to not have the expert recipe icon as a text char, instead using a texture/image.
// We need to store it somewhere where it doesn't collide.
// The game uses up to ~`\u{e0e0}`, egui's default fonts use space from `\u{e600}` upwards.
const EXPERT_RECIPE_ICON_CHAR: char = '\u{e100}';
const EXPERT_RECIPE_ICON_COLOR: egui::Color32 = egui::Color32::from_rgb(226, 122, 94);

#[derive(Debug, Hash)]
pub enum NameSource {
    Item(u32, bool),
    Recipe(u32, bool),
    Mission(u32),
}

impl From<&Consumable> for NameSource {
    fn from(consumable: &Consumable) -> Self {
        Self::Item(consumable.item_id, consumable.hq)
    }
}

impl From<&Ingredient> for NameSource {
    fn from(ingredient: &Ingredient) -> Self {
        Self::Item(ingredient.item_id, false)
    }
}

impl From<&Recipe> for NameSource {
    fn from(recipe: &Recipe) -> Self {
        Self::Recipe(recipe.item_id, recipe.is_expert)
    }
}

pub struct GameDataNameLabel {
    name_source: NameSource,
    locale: Locale,
}

impl GameDataNameLabel {
    pub fn new(name_source: impl Into<NameSource>, locale: Locale) -> Self {
        Self {
            name_source: name_source.into(),
            locale,
        }
    }
}

impl egui::Widget for GameDataNameLabel {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let Self {
            name_source,
            locale,
        } = self;
        let id = ui.id().with(&name_source);
        let mut layout_job = egui::text::LayoutJob::default();
        let style = ui.style();

        let name = match name_source {
            NameSource::Item(item_id, hq) => {
                get_item_name(item_id, hq, locale).unwrap_or(t!(locale, "Unknown item").to_owned())
            }
            NameSource::Recipe(item_id, _) => get_item_name(item_id, false, locale)
                .unwrap_or(t!(locale, "Unknown item").to_owned()),
            NameSource::Mission(mission_id) => get_stellar_mission_name(mission_id, locale)
                .unwrap_or(t!(locale, "Unknown mission"))
                .to_owned(),
        };
        if ui.ctx().animate_bool_with_time(id, false, 0.25) == 0.0 {
            egui::RichText::new(&name)
        } else {
            egui::RichText::new(&name).color(style.visuals.weak_text_color())
        }
        .append_to(
            &mut layout_job,
            style,
            egui::FontSelection::Default,
            egui::Align::Center,
        );

        if let NameSource::Recipe(_, expert_recipe) = name_source
            && expert_recipe
        {
            egui::RichText::new(format!(" {}", EXPERT_RECIPE_ICON_CHAR))
                .color(EXPERT_RECIPE_ICON_COLOR)
                .append_to(
                    &mut layout_job,
                    style,
                    egui::FontSelection::Default,
                    egui::Align::Center,
                );
        }

        let response = ui.add(egui::Label::new(layout_job).sense(egui::Sense::CLICK));
        response.context_menu(|ui| {
            let copy_name_button_text = match name_source {
                NameSource::Item(_, _) | NameSource::Recipe(_, _) => t!(locale, "Copy item name"),
                NameSource::Mission(_) => t!(locale, "Copy mission name"),
            };
            if ui.button(copy_name_button_text).clicked() {
                let copied_name = name
                    .trim_end_matches([' ', raphael_data::HQ_ICON_CHAR, raphael_data::CL_ICON_CHAR])
                    .to_string();
                ui.ctx().copy_text(copied_name);
                ui.ctx().animate_bool_with_time(id, true, 0.0);
            }
        });
        response
    }
}
