use std::cell::Cell;
use std::rc::Rc;

use egui::{Align, CursorIcon, Layout, Rounding};
use egui_extras::Column;
use game_data::{CrafterConfiguration, RecipeConfiguration};
use simulator::{Action, Settings, State};

type MacroResult = Option<Vec<Action>>;

pub struct MacroSolverApp {
    actions: Vec<Action>,
    recipe_config: RecipeConfiguration,
    crafter_config: CrafterConfiguration,
    recipe_search_text: String,

    solver_pending: bool,
    bridge: gloo_worker::WorkerBridge<WebWorker>,
    data_update: Rc<Cell<Option<MacroResult>>>,
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

        let item_id = *game_data::ITEM_IDS.get("Indagator's Saw").unwrap();
        let recipe_config = RecipeConfiguration {
            item_id,
            recipe: *game_data::RECIPES.get(&item_id).unwrap(),
            hq_ingredients: [0; 6],
        };
        let crafter_config = CrafterConfiguration {
            craftsmanship: 3858,
            control: 4057,
            cp: 687,
            job_level: 90,
            manipulation: true,
        };
        Self {
            actions: Vec::new(),
            recipe_config,
            crafter_config,
            recipe_search_text: String::new(),
            solver_pending: false,
            data_update,
            bridge,
        }
    }
}

impl eframe::App for MacroSolverApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {}

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
                    ui.set_max_width(800.0);
                    ui.group(|ui| self.draw_simulator_widget(ui));
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.group(|ui| {
                            ui.set_max_width(500.0);
                            ui.set_height(400.0);
                            self.draw_recipe_select_widget(ui);
                        });
                        ui.group(|ui| {
                            ui.set_height(400.0);
                            self.draw_configuration_widget(ui)
                        });
                    });
                });
                ui.group(|ui| {
                    ui.set_width(320.0);
                    ui.set_height(507.0);
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
        let game_settings = game_data::get_game_settings(self.recipe_config, self.crafter_config);
        let game_state = State::new(&game_settings).use_actions(
            &self.actions,
            simulator::Condition::Normal,
            &game_settings,
        );
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("Simulation").strong());
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Progress");
                let max_progress = game_settings.max_progress;
                let progress = match game_state {
                    State::InProgress(state) => max_progress - state.missing_progress,
                    State::Completed { .. } => max_progress,
                    State::Failed { missing_progress } => max_progress - missing_progress,
                    State::Invalid => 0,
                };
                ui.add(
                    egui::ProgressBar::new(progress as f32 / max_progress as f32)
                        .text(format!("{} / {}", progress, max_progress))
                        .rounding(Rounding::ZERO),
                );
            });
            ui.horizontal(|ui| {
                ui.label("Quality");
                let max_quality = game_settings.max_quality;
                let quality = match game_state {
                    State::InProgress(state) => max_quality - state.missing_quality,
                    State::Completed { missing_quality } => max_quality - missing_quality,
                    State::Failed { .. } => 0,
                    State::Invalid => 0,
                };
                ui.add(
                    egui::ProgressBar::new(quality as f32 / max_quality as f32)
                        .text(format!("{} / {}", quality, max_quality))
                        .rounding(Rounding::ZERO),
                );
            });
            ui.horizontal(|ui| {
                ui.label("Durability");
                let max_durability = game_settings.max_durability;
                let durability = match game_state {
                    State::InProgress(state) => state.durability,
                    State::Completed { .. } => 0,
                    State::Failed { .. } => 0,
                    State::Invalid => 0,
                };
                ui.add(
                    egui::ProgressBar::new(durability as f32 / max_durability as f32)
                        .text(format!("{} / {}", durability, max_durability))
                        .rounding(Rounding::ZERO)
                        .desired_width(120.0),
                );
                ui.label("CP");
                let max_cp = game_settings.max_cp;
                let cp = match game_state {
                    State::InProgress(state) => state.cp,
                    State::Completed { .. } => 0,
                    State::Failed { .. } => 0,
                    State::Invalid => 0,
                };
                ui.add(
                    egui::ProgressBar::new(cp as f32 / max_cp as f32)
                        .text(format!("{} / {}", cp, max_cp))
                        .rounding(Rounding::ZERO)
                        .desired_width(120.0),
                );
            });
        });
    }

    fn draw_macro_widget(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("Macro").strong());
            ui.separator();
            for action in self.actions.iter() {
                ui.label(format!(
                    "/ac \"{}\" <wait.{}>",
                    action.display_name(),
                    action.time_cost()
                ));
            }
        });
    }

    fn draw_recipe_select_widget(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("Recipe Selection").strong());
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Selected Recipe");
                ui.label(
                    egui::RichText::new(
                        game_data::ITEMS
                            .get(&self.recipe_config.item_id)
                            .unwrap()
                            .name,
                    )
                    .strong(),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut self.recipe_search_text);
            });

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
                .striped(true)
                .resizable(false)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::auto())
                .column(Column::remainder())
                .drag_to_scroll(false)
                .min_scrolled_height(0.0);
            table
                .header(text_height, |mut header| {
                    header.col(|ui| {
                        ui.label("Item ID");
                    });
                    header.col(|ui| {
                        ui.label("Item Name");
                    });
                })
                .body(|body| {
                    body.rows(text_height, search_result.len(), |mut row| {
                        let item_id = search_result[row.index()];
                        let item = game_data::ITEMS.get(&item_id).unwrap();
                        row.col(|ui| {
                            if ui
                                .button(item_id.to_string())
                                .on_hover_cursor(CursorIcon::PointingHand)
                                .clicked()
                            {
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

    fn draw_configuration_widget(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("Configuration").strong());
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Craftsmanship");
                ui.add(egui::DragValue::new(&mut self.crafter_config.craftsmanship));
            });
            ui.horizontal(|ui| {
                ui.label("Control");
                ui.add(egui::DragValue::new(&mut self.crafter_config.control));
            });
            ui.horizontal(|ui| {
                ui.label("CP");
                ui.add(egui::DragValue::new(&mut self.crafter_config.cp));
            });
            ui.horizontal(|ui| {
                ui.label("Job Level");
                ui.add(
                    egui::DragValue::new(&mut self.crafter_config.job_level).clamp_range(1..=90),
                );
            });
            ui.checkbox(
                &mut self.crafter_config.manipulation,
                "Manipulation unlocked",
            );

            ui.horizontal(|ui| {
                if ui.button("Solve").clicked() {
                    self.solver_pending = true;
                    let game_settings =
                        game_data::get_game_settings(self.recipe_config, self.crafter_config);
                    self.bridge.send(game_settings);
                    log::debug!("Message send {game_settings:?}");
                }
                ui.add_visible(self.solver_pending, egui::Spinner::new());
            });
        });
    }
}

pub struct WebWorker {}

impl gloo_worker::Worker for WebWorker {
    type Message = u64;
    type Input = Settings;
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
        scope.respond(_id, solvers::MacroSolver::new(msg).solve(State::new(&msg)));
    }
}
