use std::cell::Cell;
use std::rc::Rc;

use egui::{Align, CursorIcon, Layout, Rounding, TextureHandle, TextureOptions};
use egui_extras::Column;
use game_data::{Consumable, Item, RecipeConfiguration};
use simulator::{state::InProgress, Action, Settings, SimulationState};

use crate::{
    config::{CrafterConfig, JOB_NAMES},
    widgets::MacroTextView,
};

type MacroResult = Option<Vec<Action>>;

struct MacroViewConfig {
    split: bool,
    split_length: usize,
    delay: bool,
}

impl Default for MacroViewConfig {
    fn default() -> Self {
        Self {
            split: false,
            split_length: 15,
            delay: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QualityTarget {
    Zero,
    CollectableT1,
    CollectableT2,
    CollectableT3,
    Full,
}

impl QualityTarget {
    pub fn get_target(self, max_quality: u16) -> u16 {
        (max_quality as f64
            * match self {
                Self::Zero => 0.0,
                Self::CollectableT1 => 0.55,
                Self::CollectableT2 => 0.75,
                Self::CollectableT3 => 0.95,
                Self::Full => 1.00,
            })
        .ceil() as u16
    }
}

impl std::fmt::Display for QualityTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Zero => "0% quality",
                Self::CollectableT1 => "55% quality",
                Self::CollectableT2 => "75% quality",
                Self::CollectableT3 => "95% quality",
                Self::Full => "100% quality",
            }
        )
    }
}

struct SolverConfig {
    quality_target: QualityTarget,
    backload_progress: bool,
}

pub struct MacroSolverApp {
    actions: Vec<Action>,
    recipe_config: RecipeConfiguration,

    crafter_config: CrafterConfig,
    saved_crafter_config: CrafterConfig,

    solver_config: SolverConfig,
    selected_food: Option<Consumable>,
    selected_potion: Option<Consumable>,

    recipe_search_text: String,

    macro_view_config: MacroViewConfig,

    solver_pending: bool,
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
            .spawn("./dummy_worker.js");

        cc.egui_ctx.set_pixels_per_point(1.2);
        cc.egui_ctx.style_mut(|style| {
            style.visuals.interact_cursor = Some(CursorIcon::PointingHand);
            style.url_in_tooltip = true;
            style.always_scroll_the_only_direction = true;
        });

        let item_id: u32 = 38890;
        let recipe_config = RecipeConfiguration {
            item_id, // Indagator's Saw
            recipe: *game_data::RECIPES.get(&item_id).unwrap(),
            hq_ingredients: [0; 6],
        };

        let crafter_config: CrafterConfig = match cc.storage {
            Some(storage) => eframe::get_value(storage, "CRAFTER_CONFIG").unwrap_or_default(),
            None => Default::default(),
        };

        let solver_config = SolverConfig {
            quality_target: QualityTarget::Full,
            backload_progress: false,
        };

        Self {
            actions: Vec::new(),
            recipe_config,

            crafter_config,
            saved_crafter_config: crafter_config,

            solver_config,
            selected_food: None,
            selected_potion: None,

            recipe_search_text: String::new(),
            macro_view_config: Default::default(),
            solver_pending: false,
            data_update,
            bridge,
            action_icons: Self::load_action_icons(&cc.egui_ctx),
        }
    }
}

impl eframe::App for MacroSolverApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if self.saved_crafter_config != self.crafter_config {
            eframe::set_value(storage, "CRAFTER_CONFIG", &self.crafter_config);
            self.saved_crafter_config = self.crafter_config;
            log::debug!("Saved crafter config: {:?}", self.crafter_config);
        }
    }

    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(5)
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(update) = self.data_update.take() {
            log::debug!("Received update: {update:?}");
            self.actions = update.unwrap_or(Vec::new());
            self.solver_pending = false;
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.label(egui::RichText::new("Raphael  |  FFXIV Crafting Solver").strong());
                ui.label(format!("v{}", env!("CARGO_PKG_VERSION")));
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
                        "https://discord.gg/Qd9u9CtaYj",
                    )
                    .open_in_new_tab(true),
                );
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel")
            .show_separator_line(false)
            .show(ctx, |ui| {
                egui::warn_if_debug_build(ui);
                powered_by_egui_and_eframe(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.set_enabled(!self.solver_pending);
                ui.with_layout(Layout::top_down_justified(Align::TOP), |ui| {
                    ui.set_max_width(885.0);
                    ui.group(|ui| self.draw_simulator_widget(ui));
                    ui.add_space(5.5);
                    ui.group(|ui| {
                        ui.set_width(873.0);
                        ui.set_height(30.0);
                        self.draw_actions_widget(ui);
                    });
                    ui.add_space(5.5);
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.push_id("RECIPE_SELECT", |ui| {
                                ui.group(|ui| {
                                    ui.set_max_width(600.0);
                                    ui.set_max_height(200.0);
                                    self.draw_recipe_select_widget(ui);
                                    ui.shrink_height_to_current();
                                });
                            });
                            ui.add_space(5.5);
                            ui.push_id("FOOD_SELECT", |ui| {
                                ui.group(|ui| {
                                    ui.set_max_width(600.0);
                                    ui.set_max_height(160.0);
                                    self.draw_meal_select_widget(ui);
                                });
                            });
                            ui.add_space(5.5);
                            ui.push_id("POTION_SELECT", |ui| {
                                ui.group(|ui| {
                                    ui.set_max_width(600.0);
                                    ui.set_max_height(160.0);
                                    self.draw_potion_select_widget(ui);
                                });
                            });
                        });
                        ui.group(|ui| {
                            ui.set_height(560.0);
                            self.draw_configuration_widget(ui)
                        });
                    });
                });
                ui.group(|ui| {
                    ui.set_width(320.0);
                    ui.set_height(717.0);
                    self.draw_macro_widget(ui);
                });
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}

impl MacroSolverApp {
    fn draw_simulator_widget(&mut self, ui: &mut egui::Ui) {
        let game_settings = game_data::get_game_settings(
            self.recipe_config,
            self.crafter_config.stats(),
            self.selected_food,
            self.selected_potion,
        );
        let game_state = match SimulationState::from_macro(&game_settings, &self.actions) {
            Ok(state) => state,
            Err(_) => SimulationState::new(&game_settings),
        };
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("Simulation").strong());
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Progress:");
                let max_progress = game_settings.max_progress;
                let progress = game_settings.max_progress - game_state.missing_progress;
                ui.add(
                    egui::ProgressBar::new(progress as f32 / max_progress as f32)
                        .text(format!("{} / {}", progress, max_progress))
                        .rounding(Rounding::ZERO),
                );
            });
            ui.horizontal(|ui| {
                ui.label("Quality:");
                let max_quality = game_settings.max_quality;
                let quality = game_settings.max_quality - game_state.missing_quality;
                ui.add(
                    egui::ProgressBar::new(quality as f32 / max_quality as f32)
                        .text(format!("{} / {}", quality, max_quality))
                        .rounding(Rounding::ZERO),
                );
            });
            ui.horizontal(|ui| {
                ui.label("Durability:");
                let max_durability = game_settings.max_durability;
                let durability = game_state.durability;
                ui.add(
                    egui::ProgressBar::new(durability as f32 / max_durability as f32)
                        .text(format!("{} / {}", durability, max_durability))
                        .rounding(Rounding::ZERO)
                        .desired_width(120.0),
                );
                ui.label("CP:");
                let max_cp = game_settings.max_cp;
                let cp = game_state.cp;
                ui.add(
                    egui::ProgressBar::new(cp as f32 / max_cp as f32)
                        .text(format!("{} / {}", cp, max_cp))
                        .rounding(Rounding::ZERO)
                        .desired_width(120.0),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let item = game_data::ITEMS.get(&self.recipe_config.item_id).unwrap();
                    if item.can_be_hq {
                        let quality = game_settings.max_quality - game_state.missing_quality;
                        let hq = match game_state.missing_progress {
                            0 => game_data::hq_percentage(quality, game_settings.max_quality),
                            _ => 0,
                        };
                        ui.label(egui::RichText::new(format!("{hq}% HQ")).strong());
                    } else if item.is_collectable {
                        let quality = game_settings.max_quality - game_state.missing_quality;
                        let t1 = QualityTarget::CollectableT1.get_target(game_settings.max_quality);
                        let t2 = QualityTarget::CollectableT2.get_target(game_settings.max_quality);
                        let t3 = QualityTarget::CollectableT3.get_target(game_settings.max_quality);
                        let tier = match quality {
                            quality if quality >= t3 => 3,
                            quality if quality >= t2 => 2,
                            quality if quality >= t1 => 1,
                            _ => 0,
                        };
                        ui.label(
                            egui::RichText::new(format!("Tier {tier} collectable reached"))
                                .strong(),
                        );
                    } else {
                        ui.label("Item cannot be HQ");
                    }
                });
            });
        });
    }

    fn draw_macro_widget(&mut self, ui: &mut egui::Ui) {
        let macro_steps = self.actions.len();
        let macro_duration: i16 = self.actions.iter().map(|action| action.time_cost()).sum();
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Macro").strong());
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(format!(
                        "{} steps | {} seconds",
                        macro_steps, macro_duration
                    ));
                });
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.add(egui::Checkbox::new(
                    &mut self.macro_view_config.delay,
                    "Include delay",
                ));
                ui.add(egui::Checkbox::new(
                    &mut self.macro_view_config.split,
                    "Split macro",
                ));
            });
            ui.separator();
            let chunk_size = match self.macro_view_config.split {
                true => self.macro_view_config.split_length,
                false => usize::MAX,
            };
            for actions in self.actions.chunks(chunk_size) {
                ui.add(MacroTextView::new(actions, self.macro_view_config.delay));
            }
        });
    }

    fn draw_actions_widget(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::horizontal().show(ui, |ui| {
            ui.horizontal(|ui| {
                for action in self.actions.iter() {
                    ui.add(
                        egui::Image::new(self.action_icons.get(action).unwrap())
                            .max_height(30.0)
                            .rounding(4.0),
                    )
                    .on_hover_text(action.display_name());
                }
            });
        });
    }

    fn draw_recipe_select_widget(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Recipe").strong());
                ui.label(egui::RichText::new(
                    game_data::ITEMS
                        .get(&self.recipe_config.item_id)
                        .unwrap()
                        .name,
                ));
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut self.recipe_search_text);
            });
            ui.separator();

            let mut search_result: Vec<u32> = game_data::RECIPES
                .keys()
                .copied()
                .filter(|item_id| match game_data::ITEMS.get(item_id) {
                    Some(item) => item
                        .name
                        .to_lowercase()
                        .contains(&self.recipe_search_text.to_lowercase()),
                    _ => false,
                })
                .collect();
            search_result.sort();

            let text_height = egui::TextStyle::Body
                .resolve(ui.style())
                .size
                .max(ui.spacing().interact_size.y);
            let table = egui_extras::TableBuilder::new(ui)
                .auto_shrink(false)
                .striped(true)
                .resizable(false)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::auto())
                .column(Column::remainder())
                .min_scrolled_height(0.0);
            table
                .header(text_height, |mut header| {
                    header.col(|_| {});
                    header.col(|ui| {
                        ui.label("Item");
                    });
                })
                .body(|body| {
                    body.rows(text_height, search_result.len(), |mut row| {
                        let item_id = search_result[row.index()];
                        let item = game_data::ITEMS.get(&item_id).unwrap();
                        row.col(|ui| {
                            if ui.button("Select").clicked() {
                                self.recipe_config = RecipeConfiguration {
                                    item_id,
                                    recipe: *game_data::RECIPES.get(&item_id).unwrap(),
                                    hq_ingredients: [0; 6],
                                }
                            };
                        });
                        row.col(|ui| {
                            ui.label(item.name);
                        });
                    });
                });
        });
    }

    fn draw_meal_select_widget(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Food").strong());
                ui.label(match self.selected_food {
                    Some(food) => food.name,
                    None => "None",
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .add_enabled(self.selected_food.is_some(), egui::Button::new("Clear"))
                        .clicked()
                    {
                        self.selected_food = None;
                    }
                });
            });
            ui.separator();
            let text_height = egui::TextStyle::Body
                .resolve(ui.style())
                .size
                .max(ui.spacing().interact_size.y);
            let table = egui_extras::TableBuilder::new(ui)
                .auto_shrink(false)
                .striped(true)
                .resizable(false)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::auto())
                .column(Column::exact(240.0))
                .column(Column::remainder())
                .min_scrolled_height(0.0);
            table
                .header(text_height, |mut header| {
                    header.col(|_| {});
                    header.col(|ui| {
                        ui.label("Item Name");
                    });
                    header.col(|ui| {
                        ui.label("Effect");
                    });
                })
                .body(|body| {
                    body.rows(text_height, game_data::MEALS.len(), |mut row| {
                        let item = game_data::MEALS[row.index()];
                        row.col(|ui| {
                            if ui.button("Select").clicked() {
                                self.selected_food = Some(item);
                            }
                        });
                        row.col(|ui| {
                            ui.label(item.name);
                        });
                        row.col(|ui| {
                            ui.label(item.effect_string(
                                self.crafter_config.craftsmanship(),
                                self.crafter_config.control(),
                                self.crafter_config.cp(),
                            ));
                        });
                    });
                });
        });
    }

    fn draw_potion_select_widget(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Potion").strong());
                ui.label(match self.selected_potion {
                    Some(item) => item.name,
                    None => "None",
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .add_enabled(self.selected_potion.is_some(), egui::Button::new("Clear"))
                        .clicked()
                    {
                        self.selected_potion = None;
                    }
                });
            });
            ui.separator();
            let text_height = egui::TextStyle::Body
                .resolve(ui.style())
                .size
                .max(ui.spacing().interact_size.y);
            let table = egui_extras::TableBuilder::new(ui)
                .auto_shrink(false)
                .striped(true)
                .resizable(false)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::auto())
                .column(Column::exact(240.0))
                .column(Column::remainder())
                .min_scrolled_height(0.0);
            table
                .header(text_height, |mut header| {
                    header.col(|_| {});
                    header.col(|ui| {
                        ui.label("Item Name");
                    });
                    header.col(|ui| {
                        ui.label("Effect");
                    });
                })
                .body(|body| {
                    body.rows(text_height, game_data::POTIONS.len(), |mut row| {
                        let item = game_data::POTIONS[row.index()];
                        row.col(|ui| {
                            if ui.button("Select").clicked() {
                                self.selected_potion = Some(item);
                            }
                        });
                        row.col(|ui| {
                            ui.label(item.name);
                        });
                        row.col(|ui| {
                            ui.label(item.effect_string(
                                self.crafter_config.craftsmanship(),
                                self.crafter_config.control(),
                                self.crafter_config.cp(),
                            ));
                        });
                    });
                });
        });
    }

    fn draw_configuration_widget(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("Configuration").strong());
            ui.separator();

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Crafter stats").strong());
                egui::ComboBox::from_id_source("SELECTED_JOB")
                    .width(20.0)
                    .selected_text(JOB_NAMES[self.crafter_config.selected_job])
                    .show_ui(ui, |ui| {
                        for i in 0..8 {
                            ui.selectable_value(
                                &mut self.crafter_config.selected_job,
                                i,
                                JOB_NAMES[i],
                            );
                        }
                    });
            });
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
                            .clamp_range(0..=9999),
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
                    ui.add(
                        egui::DragValue::new(self.crafter_config.control_mut())
                            .clamp_range(0..=9999),
                    );
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
                    ui.add(
                        egui::DragValue::new(self.crafter_config.cp_mut()).clamp_range(0..=9999),
                    );
                });
            });
            ui.horizontal(|ui| {
                ui.label("Job Level:");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add(
                        egui::DragValue::new(self.crafter_config.level_mut()).clamp_range(1..=100),
                    );
                });
            });
            ui.separator();

            ui.label(egui::RichText::new("HQ ingredients").strong());
            let ingredients: Vec<(Item, u32)> = self
                .recipe_config
                .recipe
                .ingredients
                .iter()
                .filter_map(|ingredient| match ingredient.item_id {
                    0 => None,
                    id => Some((*game_data::ITEMS.get(&id).unwrap(), ingredient.amount)),
                })
                .collect();
            let mut has_hq_ingredient = false;
            for (index, (item, max_amount)) in ingredients.iter().enumerate() {
                if item.can_be_hq {
                    has_hq_ingredient = true;
                    ui.horizontal(|ui| {
                        ui.label(item.name);
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let mut max_placeholder = *max_amount;
                            ui.add_enabled(false, egui::DragValue::new(&mut max_placeholder));
                            ui.monospace("/");
                            ui.add(
                                egui::DragValue::new(&mut self.recipe_config.hq_ingredients[index])
                                    .clamp_range(0..=*max_amount),
                            );
                        });
                    });
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
                egui::Checkbox::new(&mut false, "Enable specialist actions"),
            );
            ui.separator();

            ui.label(egui::RichText::new("Solver settings").strong());
            ui.horizontal(|ui| {
                ui.label("Target quality");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
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
                        });
                });
            });
            ui.checkbox(
                &mut self.solver_config.backload_progress,
                "Backload progress actions",
            );
            if self.solver_config.backload_progress {
                ui.label(
                    egui::RichText::new("⚠ Backloading progress may decrease achievable Quality.")
                        .small()
                        .color(ui.visuals().warn_fg_color),
                );
            }

            ui.add_space(5.5);
            ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                if ui.button("Solve").clicked() {
                    self.solver_pending = true;
                    let mut game_settings = game_data::get_game_settings(
                        self.recipe_config,
                        self.crafter_config.crafter_stats[self.crafter_config.selected_job],
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
                ui.add_visible(self.solver_pending, egui::Spinner::new());
            });
        });
    }

    fn load_action_icons(ctx: &egui::Context) -> std::collections::HashMap<Action, TextureHandle> {
        crate::assets::get_action_icons()
            .into_iter()
            .map(|(action, image)| {
                let texture = ctx.load_texture(
                    action.display_name(),
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
