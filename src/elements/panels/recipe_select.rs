use egui::{
    Align, Id, Layout, Widget,
    util::cache::{ComputerMut, FrameCache},
};
use egui_extras::Column;
use raphael_data::{
    Consumable, CosmicExplorationZone, Locale, RLVLS, Recipe, RecipeSearchEntry, RecipeSearchQuery,
    StellarMissionSearchEntry, StellarSearchQuery, find_recipes, find_stellar_missions,
    get_cosmic_exploration_zone_name, get_game_settings, get_job_name,
};
use raphael_translations::{t, t_format};

use crate::{
    config::{CrafterConfig, QualitySource, RecipeConfiguration, RecipeSource},
    context::{AppContext, RecipeSearchFilters, RecipeSearchState, SolverConfig},
    elements::{
        util::{self, TableColumnWidth},
        widgets::{DropDown, GameDataNameLabel, NameSource, collapse_persisted},
    },
};

#[derive(Debug, Clone, Copy, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SearchDomain {
    #[default]
    Recipes,
    StellarMissions,
}

impl SearchDomain {
    fn display(self, locale: Locale) -> &'static str {
        match self {
            Self::Recipes => t!(locale, "Recipes"),
            Self::StellarMissions => t!(locale, "Missions"),
        }
    }
}

#[derive(Default)]
struct RecipeFinder {}

impl ComputerMut<RecipeSearchQuery<'_>, Vec<RecipeSearchEntry>> for RecipeFinder {
    fn compute(&mut self, query: RecipeSearchQuery) -> Vec<RecipeSearchEntry> {
        find_recipes(query).collect::<Vec<_>>()
    }
}

type RecipeSearchCache<'a> = FrameCache<Vec<RecipeSearchEntry>, RecipeFinder>;

#[derive(Default)]
struct StellarMissionFinder {}

impl ComputerMut<StellarSearchQuery<'_>, Vec<StellarMissionSearchEntry>> for StellarMissionFinder {
    fn compute(&mut self, query: StellarSearchQuery) -> Vec<StellarMissionSearchEntry> {
        find_stellar_missions(query).collect::<Vec<_>>()
    }
}

type StellarMissionSearchCache<'a> =
    FrameCache<Vec<StellarMissionSearchEntry>, StellarMissionFinder>;

pub struct RecipeSelect<'a> {
    search_state: &'a mut RecipeSearchState,
    crafter_config: &'a mut CrafterConfig,
    recipe_config: &'a mut RecipeConfiguration,
    solver_config: &'a mut SolverConfig,
    selected_food: Option<Consumable>, // used for base prog/qual display
    selected_potion: Option<Consumable>, // used for base prog/qual display
    locale: Locale,
}

impl<'a> RecipeSelect<'a> {
    pub fn new(app_context: &'a mut AppContext) -> Self {
        let AppContext {
            locale,
            search_state,
            recipe_config,
            selected_food,
            selected_potion,
            crafter_config,
            solver_config,
            ..
        } = app_context;

        Self {
            search_state: &mut search_state.recipe,
            crafter_config,
            recipe_config,
            solver_config,
            selected_food: *selected_food,
            selected_potion: *selected_potion,
            locale: *locale,
        }
    }

    fn select_normal_recipe(&mut self, recipe_id: u32, recipe: Recipe) {
        self.crafter_config.selected_job = recipe.job_id;
        let recipe_source = RecipeSource::Normal {
            id: recipe_id,
            data: recipe,
        };
        *self.recipe_config = RecipeConfiguration {
            recipe_source,
            quality_source: QualitySource::HqMaterialList([0; 6]),
        };
        self.solver_config.stellar_steady_hand_charges = 0;
    }

    fn draw_normal_recipe_select(&mut self, ui: &mut egui::Ui) {
        let locale = self.locale;
        let filters_active = self.search_state.filters_active();
        let Self {
            search_state:
                RecipeSearchState {
                    search_domain,
                    search_text,
                    filters,
                    ..
                },
            ..
        } = self;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            ui.add(DropDown::new(
                "RECIPE_SEARCH_DOMAIN",
                search_domain,
                [SearchDomain::Recipes, SearchDomain::StellarMissions],
                |search_domain: SearchDomain| search_domain.display(locale),
            ));
            let button_text_size = egui::TextStyle::Button.resolve(ui.style()).size;
            let icon_size = egui::Vec2::splat(button_text_size);
            let search_text_edit_width = ui.available_width()
                - (icon_size.x + ui.style().spacing.button_padding.x * 2.0)
                - ui.style().spacing.item_spacing.x;
            if egui::TextEdit::singleline(search_text)
                .desired_width(search_text_edit_width)
                .hint_text(t!(locale, "🔍 Search"))
                .ui(ui)
                .changed()
            {
                *search_text = search_text.replace('\0', "");
            }
            Self::draw_recipe_filter_button(ui, filters_active, icon_size);
        });

        ui.separator();

        match search_domain {
            SearchDomain::Recipes => {
                let search_result = ui.ctx().memory_mut(|mem| {
                    mem.caches
                        .cache::<RecipeSearchCache<'_>>()
                        .get(RecipeSearchQuery {
                            text: search_text,
                            locale,
                            filters: filters.construct_recipe_filter(self.crafter_config),
                        })
                        .clone()
                });
                self.draw_recipe_select_table(ui, search_result);
            }
            SearchDomain::StellarMissions => {
                let search_result = ui.ctx().memory_mut(|mem| {
                    mem.caches
                        .cache::<StellarMissionSearchCache<'_>>()
                        .get(StellarSearchQuery {
                            text: search_text,
                            locale,
                            filters: filters.construct_stellar_mission_filter(self.crafter_config),
                        })
                        .clone()
                });
                self.draw_mission_recipe_select(ui, search_result);
            }
        }
    }

    fn draw_recipe_select_table(
        &mut self,
        ui: &mut egui::Ui,
        search_result: Vec<RecipeSearchEntry>,
    ) {
        let locale = self.locale;
        let line_height = ui.spacing().interact_size.y;
        let line_spacing = ui.spacing().item_spacing.y;
        let table_height = 6.3 * line_height + 6.0 * line_spacing;

        // Column::remainder().clip(true) is buggy when resizing the table
        let column_widths = util::calculate_column_widths(
            ui,
            [
                TableColumnWidth::SelectButton,
                TableColumnWidth::JobName,
                TableColumnWidth::Remaining,
            ],
            locale,
        );

        let table = egui_extras::TableBuilder::new(ui)
            .id_salt("RECIPE_SELECT_TABLE")
            .auto_shrink(false)
            .striped(true)
            .column(Column::exact(column_widths[0]))
            .column(Column::exact(column_widths[1]))
            .column(Column::exact(column_widths[2]))
            .min_scrolled_height(table_height)
            .max_scroll_height(table_height);
        table.body(|body| {
            body.rows(line_height, search_result.len(), |mut row| {
                let (recipe_id, recipe) = search_result[row.index()];
                row.col(|ui| {
                    if ui.button(t!(locale, "Select")).clicked() {
                        self.select_normal_recipe(recipe_id, *recipe);
                    }
                });
                row.col(|ui| {
                    ui.label(get_job_name(recipe.job_id, locale));
                });
                row.col(|ui| {
                    ui.add(GameDataNameLabel::new(recipe, locale));
                });
            });
        });
    }

    fn draw_mission_recipe_select(
        &mut self,
        ui: &mut egui::Ui,
        search_result: Vec<raphael_data::StellarMissionSearchEntry>,
    ) {
        let locale = self.locale;
        let line_height = ui.spacing().interact_size.y;
        let line_spacing = ui.spacing().item_spacing.y;
        let table_height = 6.3 * line_height + 6.0 * line_spacing;

        let line_heights = search_result.iter().map(|(_mission_id, mission)| {
            let recipe_count = mission.recipe_ids.len();
            line_height * (1 + recipe_count) as f32 + line_spacing * recipe_count as f32
        });

        // See above note, 'Column::remainder().clip(true) is buggy [...]'
        let column_widths = util::calculate_column_widths(
            ui,
            [TableColumnWidth::JobName, TableColumnWidth::Remaining],
            locale,
        );

        let table = egui_extras::TableBuilder::new(ui)
            .id_salt("MISSION_RECIPE_SELECT_TABLE")
            .auto_shrink(false)
            .striped(true)
            .column(Column::exact(column_widths[0]))
            .column(Column::exact(column_widths[1]))
            .min_scrolled_height(table_height)
            .max_scroll_height(table_height);
        table.body(|body| {
            body.heterogeneous_rows(line_heights, |mut row| {
                let (mission_id, mission) = search_result[row.index()];
                row.col(|ui| {
                    ui.label(get_job_name(mission.job_id, locale));
                });
                let row_index = row.index();
                row.col(|ui| {
                    ui.add(GameDataNameLabel::new(
                        NameSource::Mission { mission_id },
                        locale,
                    ));
                    for (index, recipe_id) in mission.recipe_ids.iter().enumerate() {
                        let recipe = &raphael_data::RECIPES[*recipe_id];
                        const DARKENING_COLOR_DARK_MODE: egui::Color32 =
                            egui::Color32::from_black_alpha(25);
                        const DARKENING_COLOR_LIGHT_MODE: egui::Color32 =
                            egui::Color32::from_black_alpha(3);
                        let background_color = match (index % 2) == 0 {
                            true => match (row_index % 2) == 0 {
                                true => ui.style().visuals.faint_bg_color,
                                false => match ui.visuals().dark_mode {
                                    true => DARKENING_COLOR_DARK_MODE,
                                    false => DARKENING_COLOR_LIGHT_MODE,
                                },
                            },
                            false => egui::Color32::TRANSPARENT,
                        };
                        egui::Frame::new().fill(background_color).show(ui, |ui| {
                            ui.horizontal(|ui| {
                                if ui.button(t!(locale, "Select")).clicked() {
                                    self.select_normal_recipe(*recipe_id, *recipe);
                                }
                                ui.add(GameDataNameLabel::new(recipe, locale));
                                ui.allocate_space(ui.available_size());
                            });
                        });
                    }
                });
            });
        });
    }

    fn draw_custom_recipe_select(self, ui: &mut egui::Ui) {
        let locale = self.locale;
        let default_game_settings = get_game_settings(
            *self.recipe_config.recipe(),
            None,
            *self.crafter_config.active_stats(),
            self.selected_food,
            self.selected_potion,
        );
        let (recipe, custom_recipe_overrides) = match &mut self.recipe_config.recipe_source {
            RecipeSource::Normal { .. } => unimplemented!(
                "Custom recipe select should only be drawn for an actual custom recipe"
            ),
            RecipeSource::Custom { data, overrides } => (data, overrides),
        };
        let mut use_base_increase_overrides =
            custom_recipe_overrides.base_progress_override.is_some();
        ui.label(egui::RichText::new(t_format!(
            locale,
            "⚠ Patch {ffxiv_patch} recipes and items are already included. Only use custom recipes if you are an advanced user or if new recipes haven't been added yet.",
            ffxiv_patch = "7.51"
        )).small().color(ui.visuals().warn_fg_color));
        ui.separator();
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                let mut recipe_job_level = RLVLS[recipe.recipe_level as usize].job_level;
                ui.horizontal(|ui| {
                    ui.label(t!(locale, "Level:"));
                    ui.add_enabled_ui(use_base_increase_overrides, |ui| {
                        ui.add(egui::DragValue::new(&mut recipe_job_level).range(1..=100));
                        if use_base_increase_overrides {
                            recipe.recipe_level =
                                raphael_data::LEVEL_ADJUST_TABLE[recipe_job_level as usize];
                        }
                    });
                });
                ui.horizontal(|ui| {
                    ui.add_enabled_ui(!use_base_increase_overrides, |ui| {
                        ui.label(t!(locale, "Recipe Level:"));
                        let mut rlvl_drag_value_widget =
                            egui::DragValue::new(&mut recipe.recipe_level)
                                .range(1..=RLVLS.len() - 1);
                        if use_base_increase_overrides && recipe_job_level >= 50 {
                            rlvl_drag_value_widget = rlvl_drag_value_widget.suffix("+");
                        }
                        ui.add(rlvl_drag_value_widget);
                    });
                });
                ui.horizontal(|ui| {
                    ui.label(t!(locale, "Progress:"));
                    ui.add(egui::DragValue::new(
                        &mut custom_recipe_overrides.max_progress_override,
                    ));
                });
                ui.horizontal(|ui| {
                    ui.label(t!(locale, "Quality:"));
                    ui.add(egui::DragValue::new(
                        &mut custom_recipe_overrides.max_quality_override,
                    ));
                });
                if let QualitySource::Value(initial_quality) =
                    &mut self.recipe_config.quality_source
                {
                    ui.horizontal(|ui| {
                        ui.label(t!(locale, "Initial Quality:"));
                        ui.add(egui::DragValue::new(initial_quality));
                    });
                }
                ui.horizontal(|ui| {
                    ui.label(t!(locale, "Durability:"));
                    ui.add(
                        egui::DragValue::new(&mut custom_recipe_overrides.max_durability_override)
                            .range(10..=100),
                    );
                });
                ui.checkbox(&mut recipe.is_expert, t!(locale, "Expert recipe"));
            });
            ui.separator();
            ui.vertical(|ui| {
                let mut rlvl = RLVLS[recipe.recipe_level as usize];
                ui.add_enabled_ui(!use_base_increase_overrides, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(t!(locale, "Progress divider"));
                        ui.add_enabled(false, egui::DragValue::new(&mut rlvl.progress_div));
                    });
                    ui.horizontal(|ui| {
                        ui.label(t!(locale, "Quality divider"));
                        ui.add_enabled(false, egui::DragValue::new(&mut rlvl.quality_div));
                    });
                    ui.horizontal(|ui| {
                        ui.label(t!(locale, "Progress modifier"));
                        ui.add_enabled(false, egui::DragValue::new(&mut rlvl.progress_mod));
                    });
                    ui.horizontal(|ui| {
                        ui.label(t!(locale, "Quality modifier"));
                        ui.add_enabled(false, egui::DragValue::new(&mut rlvl.quality_mod));
                    });
                });

                ui.horizontal(|ui| {
                    ui.label(t!(locale, "Progress per 100% efficiency:"));
                    if !use_base_increase_overrides {
                        ui.label(
                            egui::RichText::new(default_game_settings.base_progress.to_string())
                                .strong(),
                        );
                    } else {
                        let mut base_progress_override_value =
                            custom_recipe_overrides.base_progress_override.unwrap();
                        ui.add(
                            egui::DragValue::new(&mut base_progress_override_value).range(0..=9999),
                        );
                        custom_recipe_overrides.base_progress_override =
                            Some(base_progress_override_value);
                    }
                });
                ui.horizontal(|ui| {
                    ui.label(t!(locale, "Quality per 100% efficiency:"));
                    if !use_base_increase_overrides {
                        ui.label(
                            egui::RichText::new(default_game_settings.base_quality.to_string())
                                .strong(),
                        );
                    } else {
                        let mut base_quality_override_value =
                            custom_recipe_overrides.base_quality_override.unwrap();
                        ui.add(
                            egui::DragValue::new(&mut base_quality_override_value).range(0..=9999),
                        );
                        custom_recipe_overrides.base_quality_override =
                            Some(base_quality_override_value);
                    }
                });
                if ui
                    .checkbox(
                        &mut use_base_increase_overrides,
                        t!(locale, "Override per 100% efficiency values"),
                    )
                    .changed()
                {
                    if use_base_increase_overrides {
                        custom_recipe_overrides.base_progress_override =
                            Some(default_game_settings.base_progress);
                        custom_recipe_overrides.base_quality_override =
                            Some(default_game_settings.base_quality);
                    } else {
                        custom_recipe_overrides.base_progress_override = None;
                        custom_recipe_overrides.base_quality_override = None;
                    }
                }
            });
        });
    }

    fn draw_recipe_filter_modal(&mut self, ctx: &egui::Context) {
        let locale = self.locale;
        let RecipeSearchFilters {
            active_job_only,
            cosmic_exploration_zone,
        } = &mut self.search_state.filters;

        egui::containers::Modal::new(egui::Id::new("RECIPE_FILTER_MODAL")).show(ctx, |ui| {
            ui.set_width(
                (ctx.content_rect().width() - ui.style().spacing.item_spacing.x * 4.0)
                    .clamp(0.0, 395.0),
            );
            ui.style_mut().spacing.item_spacing = egui::Vec2::new(4.0, 4.0);
            ui.label(egui::RichText::new(t!(locale, "Recipe filters")).strong());
            ui.separator();
            ui.checkbox(active_job_only, t!(locale, "Active job only"));
            ui.separator();
            ui.label(
                egui::RichText::new(t!(locale, "Stellar missions"))
                    .color(ui.visuals().widgets.hovered.text_color()),
            );
            ui.horizontal_wrapped(|ui| {
                for zone in [
                    CosmicExplorationZone::SinusArdorum,
                    CosmicExplorationZone::Phaenna,
                    CosmicExplorationZone::Oizys,
                    CosmicExplorationZone::Auxesia,
                ] {
                    let response = ui.selectable_label(
                        cosmic_exploration_zone.is_none_or(|selected| selected == zone),
                        get_cosmic_exploration_zone_name(zone, locale),
                    );
                    if response.clicked() {
                        if *cosmic_exploration_zone != Some(zone) {
                            *cosmic_exploration_zone = Some(zone);
                        } else {
                            *cosmic_exploration_zone = None;
                        }
                    }
                }
            });
            ui.separator();
            ui.vertical_centered_justified(|ui| {
                if ui.button(t!(locale, "Close")).clicked()
                    || ui.input(|i| i.key_pressed(egui::Key::Escape))
                {
                    set_filter_modal_visibility(ctx, false);
                }
            });
        });
    }

    fn draw_recipe_filter_button(ui: &mut egui::Ui, filters_active: bool, icon_size: egui::Vec2) {
        // Placing the first point at the top left causes the shape to be misdrawn.
        // As a workaround, the first point is placed in the middle horizontally and the top left one is added at the end.
        const FUNNEL_POINTS_RELATIVE: [[f32; 2]; 7] = [
            [0.5, 0.055],
            [1.0, 0.055],
            [0.6, 0.55],
            [0.6, 0.975],
            [0.4, 0.875],
            [0.4, 0.55],
            [0.0, 0.055],
        ];

        ui.scope(|ui| {
            if filters_active {
                util::use_highlighted_widget_bg_color(ui);
            }
            let icon_id = ui.next_auto_id();
            let response = egui::Button::new(egui::Atom::custom(icon_id, icon_size)).atom_ui(ui);
            if let Some(rect) = response.rect(icon_id) {
                let points = FUNNEL_POINTS_RELATIVE
                    .into_iter()
                    .map(|point| rect.lerp_inside(point))
                    .collect();
                let fill = if response.hovered() {
                    ui.visuals().widgets.hovered.text_color()
                } else if response.is_pointer_button_down_on() {
                    ui.visuals().widgets.active.text_color()
                } else {
                    ui.visuals().widgets.inactive.text_color()
                };
                ui.painter_at(rect).add(egui::epaint::PathShape {
                    points,
                    closed: true,
                    fill,
                    stroke: egui::epaint::PathStroke::NONE,
                });
            }
            if response.clicked() {
                set_filter_modal_visibility(ui.ctx(), true);
            }
        });
    }
}

fn filter_modal_is_visible(ctx: &egui::Context) -> bool {
    let id = egui::Id::new("RECIPE_FILTER_MODAL_VISIBLE");
    ctx.data(|data| data.get_temp(id) == Some(true))
}

fn set_filter_modal_visibility(ctx: &egui::Context, visible: bool) {
    let id = egui::Id::new("RECIPE_FILTER_MODAL_VISIBLE");
    ctx.data_mut(|data| data.insert_temp(id, visible));
}

impl Widget for RecipeSelect<'_> {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        let locale = self.locale;

        if filter_modal_is_visible(ui.ctx()) {
            self.draw_recipe_filter_modal(ui.ctx());
        }
        ui.group(|ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 3.0);
            ui.vertical(|ui| {
                let mut collapsed = false;

                ui.horizontal(|ui| {
                    collapse_persisted(ui, Id::new("RECIPE_SEARCH_COLLAPSED"), &mut collapsed);
                    ui.label(egui::RichText::new(t!(locale, "Recipe")).strong());
                    ui.add(GameDataNameLabel::new(self.recipe_config.recipe(), locale));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .checkbox(
                                &mut self.search_state.show_custom_recipe_select,
                                t!(locale, "Custom"),
                            )
                            .changed()
                            && self.search_state.show_custom_recipe_select
                            && let RecipeSource::Normal { .. } = self.recipe_config.recipe_source
                        {
                            let default_game_settings = get_game_settings(
                                *self.recipe_config.recipe(),
                                None,
                                *self.crafter_config.active_stats(),
                                self.selected_food,
                                self.selected_potion,
                            );
                            self.recipe_config.recipe_source =
                                self.recipe_config.recipe_source.into_custom(
                                    self.crafter_config.active_stats().level,
                                    default_game_settings,
                                );
                            self.recipe_config.quality_source = QualitySource::Value(0);
                        }
                    });
                });

                if collapsed {
                    return;
                }

                ui.separator();

                if self.search_state.show_custom_recipe_select {
                    self.draw_custom_recipe_select(ui);
                } else {
                    self.draw_normal_recipe_select(ui);
                }
            });
        })
        .response
    }
}
