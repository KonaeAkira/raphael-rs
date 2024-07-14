use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use web_time::Instant;

use egui::{
    Align, CursorIcon, FontData, FontDefinitions, FontFamily, Layout, TextStyle, TextureHandle,
    TextureOptions,
};
use game_data::{
    action_name, get_item_name, get_job_name, Consumable, Locale, RecipeConfiguration, ITEMS,
};
use simulator::{state::InProgress, Action, Settings};

use crate::{
    config::{CrafterConfig, QualityTarget},
    widgets::{ConsumableSelect, MacroView, MacroViewConfig, RecipeSelect, Simulator, StatsEdit},
};

fn load<T: DeserializeOwned>(cc: &eframe::CreationContext<'_>, key: &'static str, default: T) -> T {
    match cc.storage {
        Some(storage) => eframe::get_value(storage, key).unwrap_or(default),
        None => default,
    }
}

type MacroResult = Option<Vec<Action>>;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
struct SolverConfig {
    quality_target: QualityTarget,
    backload_progress: bool,
}

pub struct MacroSolverApp {
    locale: Locale,
    recipe_config: RecipeConfiguration,
    selected_food: Option<Consumable>,
    selected_potion: Option<Consumable>,
    crafter_config: CrafterConfig,
    solver_config: SolverConfig,
    macro_view_config: MacroViewConfig,

    custom_recipe: bool,
    recipe_search_text: String,
    food_search_text: String,
    potion_search_text: String,

    stats_edit_window_open: bool,
    actions: Vec<Action>,
    solver_pending: bool,
    start_time: Option<Instant>,
    duration: Option<Duration>,
    bridge: gloo_worker::WorkerBridge<WebWorker>,
    data_update: Rc<Cell<Option<MacroResult>>>,

    action_icons: std::collections::HashMap<Action, TextureHandle>,
}

impl MacroSolverApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let ctx = cc.egui_ctx.clone();
        let data_update = Rc::new(Cell::new(None));
        let sender = data_update.clone();

        let bridge = <WebWorker as gloo_worker::Spawnable>::spawner()
            .callback(move |response| {
                sender.set(Some(response));
                ctx.request_repaint();
            })
            .spawn(concat!("./webworker", env!("RANDOM_SUFFIX"), ".js"));

        cc.egui_ctx.set_pixels_per_point(1.2);
        cc.egui_ctx.style_mut(|style| {
            style.visuals.interact_cursor = Some(CursorIcon::PointingHand);
            style.url_in_tooltip = true;
            style.always_scroll_the_only_direction = true;
        });

        Self::load_fonts(&cc.egui_ctx);

        let default_recipe_config = RecipeConfiguration {
            recipe: *game_data::RECIPES.last().unwrap(),
            hq_ingredients: [0; 6],
        };

        Self {
            locale: load(cc, "LOCALE", Locale::EN),
            recipe_config: load(cc, "RECIPE_CONFIG", default_recipe_config),
            selected_food: load(cc, "SELECTED_FOOD", None),
            selected_potion: load(cc, "SELECTED_POTION", None),
            crafter_config: load(cc, "CRAFTER_CONFIG", Default::default()),
            solver_config: load(cc, "SOLVER_CONFIG", Default::default()),
            macro_view_config: load(cc, "MACRO_VIEW_CONFIG", Default::default()),

            custom_recipe: load(cc, "CUSTOM_RECIPE", false),
            recipe_search_text: load(cc, "RECIPE_SEARCH_TEXT", Default::default()),
            food_search_text: load(cc, "FOOD_SEARCH_TEXT", Default::default()),
            potion_search_text: load(cc, "POTION_SEARCH_TEXT", Default::default()),

            stats_edit_window_open: false,
            actions: Vec::new(),
            solver_pending: false,
            start_time: None,
            duration: None,
            data_update,
            bridge,
            action_icons: Self::load_action_icons(&cc.egui_ctx),
        }
    }
}

impl eframe::App for MacroSolverApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "LOCALE", &self.locale);
        eframe::set_value(storage, "RECIPE_CONFIG", &self.recipe_config);
        eframe::set_value(storage, "SELECTED_FOOD", &self.selected_food);
        eframe::set_value(storage, "SELECTED_POTION", &self.selected_potion);
        eframe::set_value(storage, "CRAFTER_CONFIG", &self.crafter_config);
        eframe::set_value(storage, "SOLVER_CONFIG", &self.solver_config);
        eframe::set_value(storage, "MACRO_VIEW_CONFIG", &self.macro_view_config);

        eframe::set_value(storage, "CUSTOM_RECIPE", &self.custom_recipe);
        eframe::set_value(storage, "RECIPE_SEARCH_TEXT", &self.recipe_search_text);
        eframe::set_value(storage, "FOOD_SEARCH_TEXT", &self.food_search_text);
        eframe::set_value(storage, "POTION_SEARCH_TEXT", &self.potion_search_text);
    }

    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(1)
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(update) = self.data_update.take() {
            log::debug!("Received update: {update:?}");
            self.actions = update.unwrap_or(Vec::new());
            self.solver_pending = false;
            self.duration = Some(Instant::now() - self.start_time.unwrap());
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.label(egui::RichText::new("Raphael  |  FFXIV Crafting Solver").strong());
                ui.label(format!("v{}", env!("CARGO_PKG_VERSION")));

                egui::ComboBox::from_id_source("LOCALE")
                    .selected_text(format!("{}", self.locale))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.locale,
                            Locale::EN,
                            format!("{}", Locale::EN),
                        );
                        ui.selectable_value(
                            &mut self.locale,
                            Locale::DE,
                            format!("{}", Locale::DE),
                        );
                        ui.selectable_value(
                            &mut self.locale,
                            Locale::FR,
                            format!("{}", Locale::FR),
                        );
                        ui.selectable_value(
                            &mut self.locale,
                            Locale::JP,
                            format!("{}", Locale::JP),
                        );
                    });

                egui::widgets::global_dark_light_mode_buttons(ui);
                ui.add(
                    egui::Hyperlink::from_label_and_url(
                        egui::RichText::new(format!(
                            "{} View source on GitHub",
                            egui::special_emojis::GITHUB
                        )),
                        "https://github.com/KonaeAkira/raphael-rs",
                    )
                    .open_in_new_tab(true),
                );
                ui.label("/");
                ui.add(
                    egui::Hyperlink::from_label_and_url(
                        "Join Discord",
                        "https://discord.com/invite/m2aCy3y8he",
                    )
                    .open_in_new_tab(true),
                );
                ui.label("/");
                ui.add(
                    egui::Hyperlink::from_label_and_url(
                        "Support me on Ko-fi",
                        "https://ko-fi.com/konaeakira",
                    )
                    .open_in_new_tab(true),
                );
                ui.with_layout(
                    Layout::right_to_left(Align::Center),
                    egui::warn_if_debug_build,
                );
            });
        });

        let game_settings = game_data::get_game_settings(
            self.recipe_config,
            self.crafter_config.stats(),
            self.selected_food,
            self.selected_potion,
        );

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.horizontal(|ui| {
                    if self.solver_pending {
                        ui.disable();
                    }
                    ui.with_layout(Layout::top_down_justified(Align::TOP), |ui| {
                        ui.set_max_width(885.0);
                        ui.add(Simulator::new(
                            &game_settings,
                            &self.actions,
                            game_data::ITEMS
                                .get(&self.recipe_config.recipe.item_id)
                                .unwrap(),
                            &self.action_icons,
                            self.locale,
                        ));
                        ui.add_space(5.5);
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.push_id("RECIPE_SELECT", |ui| {
                                    ui.set_max_width(612.0);
                                    ui.set_max_height(212.0);
                                    ui.add(RecipeSelect::new(
                                        &mut self.crafter_config.selected_job,
                                        &mut self.recipe_config,
                                        &mut self.custom_recipe,
                                        &mut self.recipe_search_text,
                                        self.locale,
                                    ));
                                    // ui.shrink_height_to_current();
                                });
                                ui.add_space(5.0);
                                ui.push_id("FOOD_SELECT", |ui| {
                                    ui.set_max_width(612.0);
                                    ui.set_max_height(172.0);
                                    ui.add(ConsumableSelect::new(
                                        "Food",
                                        self.crafter_config.crafter_stats
                                            [self.crafter_config.selected_job as usize],
                                        game_data::MEALS,
                                        &mut self.food_search_text,
                                        &mut self.selected_food,
                                        self.locale,
                                    ));
                                });
                                ui.add_space(5.0);
                                ui.push_id("POTION_SELECT", |ui| {
                                    ui.set_max_width(612.0);
                                    ui.set_max_height(172.0);
                                    ui.add(ConsumableSelect::new(
                                        "Potion",
                                        self.crafter_config.crafter_stats
                                            [self.crafter_config.selected_job as usize],
                                        game_data::POTIONS,
                                        &mut self.potion_search_text,
                                        &mut self.selected_potion,
                                        self.locale,
                                    ));
                                });
                            });
                            ui.group(|ui| {
                                ui.set_height(560.0);
                                self.draw_configuration_widget(ui)
                            });
                        });
                    });
                    ui.add_sized(
                        [320.0, 730.0],
                        MacroView::new(&mut self.actions, &mut self.macro_view_config, self.locale),
                    );
                    // fill remaining horizontal space
                    ui.with_layout(Layout::right_to_left(Align::Center), |_| {});
                });
                // fill remaining vertical space
                ui.with_layout(Layout::bottom_up(Align::Center), |_| {});
            });
        });

        egui::Window::new(
            egui::RichText::new("Edit crafter stats")
                .strong()
                .text_style(TextStyle::Body),
        )
        .open(&mut self.stats_edit_window_open)
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.add(StatsEdit::new(self.locale, &mut self.crafter_config));
        });
    }
}

impl MacroSolverApp {
    fn draw_configuration_widget(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Configuration").strong());
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.style_mut().spacing.item_spacing = [4.0, 4.0].into();
                    if ui.button("Edit").clicked() {
                        self.stats_edit_window_open = true;
                    }
                    egui::ComboBox::from_id_source("SELECTED_JOB")
                        .width(20.0)
                        .selected_text(get_job_name(self.crafter_config.selected_job, self.locale))
                        .show_ui(ui, |ui| {
                            for i in 0..8 {
                                ui.selectable_value(
                                    &mut self.crafter_config.selected_job,
                                    i,
                                    get_job_name(i, self.locale),
                                );
                            }
                        });
                });
            });
            ui.separator();

            ui.label(egui::RichText::new("Crafter stats").strong());
            ui.horizontal(|ui| {
                ui.label("Craftsmanship:");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add_enabled(
                        false,
                        egui::DragValue::new(&mut game_data::craftsmanship_bonus(
                            self.crafter_config.craftsmanship(),
                            &[self.selected_food, self.selected_potion],
                        )),
                    );
                    ui.monospace("+");
                    ui.add(
                        egui::DragValue::new(self.crafter_config.craftsmanship_mut())
                            .range(0..=9999),
                    );
                });
            });
            ui.horizontal(|ui| {
                ui.label("Control:");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add_enabled(
                        false,
                        egui::DragValue::new(&mut game_data::control_bonus(
                            self.crafter_config.control(),
                            &[self.selected_food, self.selected_potion],
                        )),
                    );
                    ui.monospace("+");
                    ui.add(egui::DragValue::new(self.crafter_config.control_mut()).range(0..=9999));
                });
            });
            ui.horizontal(|ui| {
                ui.label("CP:");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add_enabled(
                        false,
                        egui::DragValue::new(&mut game_data::cp_bonus(
                            self.crafter_config.cp(),
                            &[self.selected_food, self.selected_potion],
                        )),
                    );
                    ui.monospace("+");
                    ui.add(egui::DragValue::new(self.crafter_config.cp_mut()).range(0..=9999));
                });
            });
            ui.horizontal(|ui| {
                ui.label("Job Level:");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add(egui::DragValue::new(self.crafter_config.level_mut()).range(1..=100));
                });
            });
            ui.separator();

            ui.label(egui::RichText::new("HQ ingredients").strong());
            let mut has_hq_ingredient = false;
            let ingredients = self.recipe_config.recipe.ingredients;
            for (index, ingredient) in ingredients.into_iter().enumerate() {
                match ITEMS.get(&ingredient.item_id) {
                    Some(item) if item.can_be_hq => {
                        has_hq_ingredient = true;
                        ui.horizontal(|ui| {
                            ui.label(get_item_name(ingredient.item_id, false, self.locale));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                let mut max_placeholder = ingredient.amount;
                                ui.add_enabled(false, egui::DragValue::new(&mut max_placeholder));
                                ui.monospace("/");
                                ui.add(
                                    egui::DragValue::new(
                                        &mut self.recipe_config.hq_ingredients[index],
                                    )
                                    .range(0..=ingredient.amount),
                                );
                            });
                        });
                    }
                    _ => (),
                }
            }
            if !has_hq_ingredient {
                ui.label("None");
            }
            ui.separator();

            ui.label(egui::RichText::new("Actions").strong());
            if self.crafter_config.level() as u32 >= Action::Manipulation.level_requirement() {
                ui.add(egui::Checkbox::new(
                    self.crafter_config.manipulation_mut(),
                    "Enable Manipulation",
                ));
            } else {
                ui.add_enabled(
                    false,
                    egui::Checkbox::new(&mut false, "Enable Manipulation"),
                );
            }
            ui.add_enabled(
                false,
                egui::Checkbox::new(&mut false, "Enable specialist actions (WIP)"),
            );
            ui.separator();

            ui.label(egui::RichText::new("Solver settings").strong());
            ui.horizontal(|ui| {
                ui.label("Target quality");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.style_mut().spacing.item_spacing = [4.0, 4.0].into();
                    let game_settings = game_data::get_game_settings(
                        self.recipe_config,
                        self.crafter_config.crafter_stats
                            [self.crafter_config.selected_job as usize],
                        self.selected_food,
                        self.selected_potion,
                    );
                    let mut current_value = self
                        .solver_config
                        .quality_target
                        .get_target(game_settings.max_quality);
                    match &mut self.solver_config.quality_target {
                        QualityTarget::Custom(value) => {
                            ui.add(egui::DragValue::new(value));
                        }
                        _ => {
                            ui.add_enabled(false, egui::DragValue::new(&mut current_value));
                        }
                    };
                    egui::ComboBox::from_id_source("TARGET_QUALITY")
                        .selected_text(format!("{}", self.solver_config.quality_target))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.solver_config.quality_target,
                                QualityTarget::Zero,
                                format!("{}", QualityTarget::Zero),
                            );
                            ui.selectable_value(
                                &mut self.solver_config.quality_target,
                                QualityTarget::CollectableT1,
                                format!("{}", QualityTarget::CollectableT1),
                            );
                            ui.selectable_value(
                                &mut self.solver_config.quality_target,
                                QualityTarget::CollectableT2,
                                format!("{}", QualityTarget::CollectableT2),
                            );
                            ui.selectable_value(
                                &mut self.solver_config.quality_target,
                                QualityTarget::CollectableT3,
                                format!("{}", QualityTarget::CollectableT3),
                            );
                            ui.selectable_value(
                                &mut self.solver_config.quality_target,
                                QualityTarget::Full,
                                format!("{}", QualityTarget::Full),
                            );
                            ui.selectable_value(
                                &mut self.solver_config.quality_target,
                                QualityTarget::Custom(current_value),
                                format!("{}", QualityTarget::Custom(0)),
                            )
                        });
                });
            });
            ui.checkbox(
                &mut self.solver_config.backload_progress,
                "Backload progress (Quick solve)",
            );
            if self.solver_config.backload_progress {
                ui.label(
                    egui::RichText::new("⚠ Backloading progress may decrease achievable quality.")
                        .small()
                        .color(ui.visuals().warn_fg_color),
                );
            }

            ui.add_space(5.5);
            ui.horizontal(|ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("Solve").clicked() {
                        self.solver_pending = true;
                        self.start_time = Some(Instant::now());
                        let mut game_settings = game_data::get_game_settings(
                            self.recipe_config,
                            self.crafter_config.crafter_stats
                                [self.crafter_config.selected_job as usize],
                            self.selected_food,
                            self.selected_potion,
                        );
                        let target_quality = self
                            .solver_config
                            .quality_target
                            .get_target(game_settings.max_quality);
                        game_settings.max_quality =
                            std::cmp::max(game_settings.initial_quality, target_quality);
                        self.bridge
                            .send((game_settings, self.solver_config.backload_progress));
                        log::debug!("Message send {game_settings:?}");
                    }
                    if self.solver_pending {
                        ui.spinner();
                    } else if let Some(duration) = self.duration {
                        ui.label(format!("Time: {:.3}s", duration.as_secs_f64()));
                    }
                });
            });
        });
    }

    fn load_fonts(ctx: &egui::Context) {
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            String::from("japanese_monospace"),
            FontData::from_static(include_bytes!(
                "../assets/fonts/M_PLUS_1_Code/static/MPLUS1Code-Regular.ttf"
            )),
        );
        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .push("japanese_monospace".to_owned());
        fonts
            .families
            .get_mut(&FontFamily::Monospace)
            .unwrap()
            .push("japanese_monospace".to_owned());
        ctx.set_fonts(fonts);
    }

    fn load_action_icons(ctx: &egui::Context) -> std::collections::HashMap<Action, TextureHandle> {
        crate::assets::get_action_icons()
            .into_iter()
            .map(|(action, image)| {
                let texture = ctx.load_texture(
                    action_name(action, Locale::EN),
                    egui::ColorImage::from_rgb([64, 64], image.as_flat_samples().as_slice()),
                    TextureOptions::LINEAR,
                );
                (action, texture)
            })
            .collect()
    }
}

pub struct WebWorker {}

impl gloo_worker::Worker for WebWorker {
    type Message = u64;
    type Input = (Settings, bool);
    type Output = MacroResult;

    fn create(_scope: &gloo_worker::WorkerScope<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _scope: &gloo_worker::WorkerScope<Self>, _msg: Self::Message) {}

    fn received(
        &mut self,
        scope: &gloo_worker::WorkerScope<Self>,
        msg: Self::Input,
        _id: gloo_worker::HandlerId,
    ) {
        let settings = msg.0;
        let backload_progress = msg.1;
        scope.respond(
            _id,
            solvers::MacroSolver::new(settings)
                .solve(InProgress::new(&settings), backload_progress),
        );
    }
}
