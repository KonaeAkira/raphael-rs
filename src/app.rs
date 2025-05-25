use std::collections::VecDeque;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};

use raphael_solver::SolverException;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use egui::{Align, CursorIcon, Id, Layout, TextStyle};
use raphael_data::{Consumable, Locale, action_name, get_job_name};

use raphael_sim::{Action, ActionImpl, HeartAndSoul, Manipulation, QuickInnovation};

use crate::config::{
    AppConfig, CrafterConfig, CustomRecipeOverridesConfiguration, QualitySource, QualityTarget,
    RecipeConfiguration,
};
use crate::{thread_pool, util, widgets::*};

fn load<T: DeserializeOwned>(cc: &eframe::CreationContext<'_>, key: &'static str, default: T) -> T {
    match cc.storage {
        Some(storage) => eframe::get_value(storage, key).unwrap_or(default),
        None => default,
    }
}

enum SolverEvent {
    NodesVisited(usize),
    Actions(Vec<Action>),
    LoadedFromHistory(),
    Finished(Option<SolverException>),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SolverConfig {
    pub quality_target: QualityTarget,
    pub backload_progress: bool,
    pub adversarial: bool,
}

#[cfg(any(debug_assertions, feature = "dev-panel"))]
#[derive(Debug, Default)]
struct DevPanelState {
    show_dev_panel: bool,
    render_info_state: RenderInfoState,
}

pub struct MacroSolverApp {
    locale: Locale,
    app_config: AppConfig,
    recipe_config: RecipeConfiguration,
    custom_recipe_overrides_config: CustomRecipeOverridesConfiguration,
    selected_food: Option<Consumable>,
    selected_potion: Option<Consumable>,
    crafter_config: CrafterConfig,
    solver_config: SolverConfig,
    macro_view_config: MacroViewConfig,
    saved_rotations_config: SavedRotationsConfig,
    saved_rotations_data: SavedRotationsData,

    #[cfg(any(debug_assertions, feature = "dev-panel"))]
    dev_panel_state: DevPanelState,

    latest_version: Arc<Mutex<semver::Version>>,
    current_version: semver::Version,

    stats_edit_window_open: bool,
    saved_rotations_window_open: bool,
    missing_stats_error_window_open: bool,

    actions: Vec<Action>,
    solver_pending: bool,
    solver_progress: usize,
    start_time: web_time::Instant,
    duration: web_time::Duration,
    solver_error: Option<SolverException>,

    solver_events: Arc<Mutex<VecDeque<SolverEvent>>>,
    solver_interrupt: raphael_solver::AtomicFlag,
}

impl MacroSolverApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let app_config = load(cc, "APP_CONFIG", AppConfig::default());
        cc.egui_ctx
            .set_zoom_factor(f32::from(app_config.zoom_percentage) * 0.01);

        cc.egui_ctx.all_styles_mut(|style| {
            style.visuals.interact_cursor = Some(CursorIcon::PointingHand);
            style.url_in_tooltip = true;
            style.always_scroll_the_only_direction = false;
            style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        });

        load_fonts(&cc.egui_ctx);

        let latest_version = Arc::new(Mutex::new(semver::Version::new(0, 0, 0)));
        #[cfg(not(target_arch = "wasm32"))]
        fetch_latest_version(latest_version.clone());

        Self {
            locale: load(cc, "LOCALE", Locale::EN),
            app_config,
            recipe_config: load(cc, "RECIPE_CONFIG", RecipeConfiguration::default()),
            custom_recipe_overrides_config: load(
                cc,
                "CUSTOM_RECIPE_OVERRIDES_CONFIG",
                CustomRecipeOverridesConfiguration::default(),
            ),
            selected_food: load(cc, "SELECTED_FOOD", None),
            selected_potion: load(cc, "SELECTED_POTION", None),
            crafter_config: load(cc, "CRAFTER_CONFIG", CrafterConfig::default()),
            solver_config: load(cc, "SOLVER_CONFIG", SolverConfig::default()),
            macro_view_config: load(cc, "MACRO_VIEW_CONFIG", MacroViewConfig::default()),
            saved_rotations_config: load(
                cc,
                "SAVED_ROTATIONS_CONFIG",
                SavedRotationsConfig::default(),
            ),
            saved_rotations_data: load(cc, "SAVED_ROTATIONS", SavedRotationsData::default()),

            #[cfg(any(debug_assertions, feature = "dev-panel"))]
            dev_panel_state: DevPanelState::default(),

            latest_version: latest_version.clone(),
            current_version: semver::Version::parse(env!("CARGO_PKG_VERSION")).unwrap(),

            stats_edit_window_open: false,
            saved_rotations_window_open: false,
            missing_stats_error_window_open: false,

            actions: Vec::new(),
            solver_pending: false,
            solver_progress: 0,
            start_time: web_time::Instant::now(),
            duration: web_time::Duration::ZERO,
            solver_error: None,

            solver_events: Arc::new(Mutex::new(VecDeque::new())),
            solver_interrupt: raphael_solver::AtomicFlag::new(),
        }
    }
}

impl eframe::App for MacroSolverApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        #[cfg(target_arch = "wasm32")]
        self.load_fonts_dyn(ctx);

        self.process_solver_events();

        if self
            .current_version
            .lt(self.latest_version.lock().unwrap().deref())
        {
            egui::Modal::new(egui::Id::new("version_check")).show(ctx, |ui| {
                let mut latest_version = self.latest_version.lock().unwrap();
                ui.style_mut().spacing.item_spacing = egui::vec2(3.0, 3.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("New version available!").strong());
                    ui.label(format!("(v{})", latest_version.deref()));
                });
                ui.add(egui::Hyperlink::from_label_and_url(
                    "Download from GitHub",
                    "https://github.com/KonaeAkira/raphael-rs/releases/latest",
                ));
                ui.separator();
                ui.vertical_centered_justified(|ui| {
                    if ui.button("Close").clicked() {
                        *latest_version.deref_mut() = semver::Version::new(0, 0, 0);
                    }
                });
            });
        }

        if self.missing_stats_error_window_open {
            egui::Modal::new(egui::Id::new("min_stats_warning")).show(ctx, |ui| {
                let req_cms = self.recipe_config.recipe.req_craftsmanship;
                let req_ctrl = self.recipe_config.recipe.req_control;
                ui.style_mut().spacing.item_spacing = egui::vec2(3.0, 3.0);
                ui.label(egui::RichText::new("Error").strong());
                ui.separator();
                ui.label("Your stats are below the minimum requirement for this recipe.");
                ui.label(format!(
                    "Requirement: {req_cms} Craftsmanship, {req_ctrl} Control."
                ));
                ui.separator();
                ui.vertical_centered_justified(|ui| {
                    if ui.button("Close").clicked() {
                        self.missing_stats_error_window_open = false;
                    }
                });
            });
        }

        #[cfg(target_arch = "wasm32")]
        if crate::OOM_PANIC_OCCURED.load(std::sync::atomic::Ordering::Relaxed) {
            self.solver_error = Some(SolverException::AllocError);
        }
        if let Some(error) = self.solver_error.clone() {
            egui::Modal::new(egui::Id::new("solver_error")).show(ctx, |ui| {
                ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 3.0);
                let unrecoverable_error;
                match error {
                    SolverException::NoSolution => {
                        ui.label(egui::RichText::new("No solution").strong());
                        ui.separator();
                        ui.label("Make sure that the recipe is set correctly and that your stats are enough to craft this item.");
                        unrecoverable_error = false;
                    }
                    SolverException::Interrupted => {
                        self.solver_error = None;
                        unrecoverable_error = false;
                    },
                    SolverException::InternalError(message) => {
                        ui.label(egui::RichText::new("Error").strong());
                        ui.separator();
                        ui.label(message);
                        ui.label("This is an internal error. Please submit a bug report :)");
                        unrecoverable_error = false;
                    },
                    #[cfg(target_arch = "wasm32")]
                    SolverException::AllocError => {
                        ui.label(egui::RichText::new("Error: Solver ran out of memory!").strong());
                        ui.separator();
                        ui.label("The solver reached the 4GB memory limit of 32-bit web assembly and crashed.");
                        ui.label("Consider enabling fewer memory intensive options.\n");
                        ui.label("Alternatively, a native version is available from the release page on GitHub.");
                        ui.label("The native version doesn't have the 4GB limit, in addition to better performance.");
                        ui.add(
                            egui::Hyperlink::from_label_and_url(
                                "View latest release on GitHub",
                                "https://github.com/KonaeAkira/raphael-rs/releases/latest",
                            )
                            .open_in_new_tab(true),
                        );
                        unrecoverable_error = true;
                    }
                }
                ui.separator();
                ui.vertical_centered_justified(|ui| {
                    if unrecoverable_error {
                        ui.label("Reload the page to reset the app");
                    } else if ui.button("Close").clicked() {
                        self.solver_error = None;
                    }
                });
            });
        }

        if self.solver_pending {
            let interrupt_pending = self.solver_interrupt.is_set();
            egui::Modal::new(egui::Id::new("solver_busy")).show(ctx, |ui| {
                ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 3.0);
                ui.set_width(180.0);
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(if interrupt_pending {
                                    "Cancelling ..."
                                } else {
                                    "Solving ..."
                                })
                                .strong(),
                            );
                            ui.label(format!("({:.2}s)", self.start_time.elapsed().as_secs_f32()));
                        });
                        if self.solver_progress == 0 {
                            ui.label("Computing ...");
                        } else {
                            // format with thousands separator
                            let num = self
                                .solver_progress
                                .to_string()
                                .as_bytes()
                                .rchunks(3)
                                .rev()
                                .map(std::str::from_utf8)
                                .collect::<Result<Vec<&str>, _>>()
                                .unwrap()
                                .join(",");
                            ui.label(format!("{} nodes visited", num));
                        }
                    });
                });

                ui.vertical_centered_justified(|ui| {
                    ui.separator();
                    let response = ui.add_enabled(!interrupt_pending, egui::Button::new("Cancel"));
                    if response.clicked() {
                        self.solver_interrupt.set();
                    }
                });
            });
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::ScrollArea::horizontal()
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .show(ui, |ui| {
                    egui::containers::menu::Bar::new().ui(ui, |ui| {
                        ui.label(egui::RichText::new("Raphael  |  FFXIV Crafting Solver").strong());
                        ui.label(format!("v{}", env!("CARGO_PKG_VERSION")));
                        self.draw_app_config_menu_button(ui, ctx);

                        egui::ComboBox::from_id_salt("LOCALE")
                            .selected_text(format!("{}", self.locale))
                            .width(0.0)
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
                                ui.selectable_value(
                                    &mut self.locale,
                                    Locale::KR,
                                    format!("{}", Locale::KR),
                                );
                            });

                        ui.add(
                            egui::Hyperlink::from_label_and_url(
                                "View source on GitHub",
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
                        #[cfg(debug_assertions)]
                        ui.allocate_space(egui::vec2(145.0, 0.0));
                        #[cfg(all(not(debug_assertions), feature = "dev-panel"))]
                        ui.allocate_space(egui::vec2(68.0, 0.0));
                        #[cfg(any(debug_assertions, feature = "dev-panel"))]
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .selectable_label(self.dev_panel_state.show_dev_panel, "Dev Panel")
                                .clicked()
                            {
                                self.dev_panel_state.show_dev_panel =
                                    !self.dev_panel_state.show_dev_panel;
                            }
                            egui::warn_if_debug_build(ui);
                            ui.separator();
                        });
                    });
                });
        });

        #[cfg(any(debug_assertions, feature = "dev-panel"))]
        if self.dev_panel_state.show_dev_panel {
            egui::SidePanel::right("dev_panel")
                .resizable(true)
                .show(ctx, |ui| {
                    ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 3.0);
                    RenderInfo::new(&mut self.dev_panel_state.render_info_state).ui(ui, _frame);
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                self.draw_simulator_widget(ui);
                ui.with_layout(
                    Layout::left_to_right(Align::TOP).with_main_wrap(true),
                    |ui| {
                        let select_min_width: f32 = 612.0;
                        let config_min_width: f32 = 300.0;
                        let macro_min_width: f32 = 290.0;

                        let select_width;
                        let config_width;
                        let macro_width;

                        let row_width = ui.available_width();
                        if row_width >= select_min_width + config_min_width + macro_min_width {
                            select_width = row_width
                                - config_min_width
                                - macro_min_width
                                - 2.0 * ui.spacing().item_spacing.x;
                            config_width = config_min_width;
                            macro_width = macro_min_width;
                        } else if row_width >= select_min_width + config_min_width {
                            select_width =
                                row_width - config_min_width - ui.spacing().item_spacing.x;
                            config_width = config_min_width;
                            macro_width = row_width;
                        } else if row_width >= config_min_width + macro_min_width {
                            select_width = row_width;
                            config_width = config_min_width;
                            macro_width =
                                row_width - config_min_width - ui.spacing().item_spacing.x;
                        } else {
                            select_width = row_width;
                            config_width = row_width;
                            macro_width = row_width;
                        }

                        let response = ui
                            .allocate_ui(egui::vec2(select_width, 0.0), |ui| {
                                self.draw_list_select_widgets(ui);
                            })
                            .response;

                        let config_min_height = match ui.available_size_before_wrap().x {
                            x if x < config_width => 0.0,
                            _ => response.rect.height(),
                        };
                        let response = ui
                            .allocate_ui(egui::vec2(config_width, config_min_height), |ui| {
                                self.draw_config_and_results_widget(ui);
                            })
                            .response;

                        let macro_min_height = match ui.available_size_before_wrap().x {
                            x if x < macro_width => 0.0,
                            _ => response.rect.height(),
                        };
                        ui.allocate_ui(egui::vec2(macro_width, macro_min_height), |ui| {
                            self.draw_macro_output_widget(ui);
                        });
                    },
                );
                #[cfg(target_arch = "wasm32")]
                ui.add_space(120.0); // Extra space to prevent anchor ad from hiding something important
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
        .min_width(400.0)
        .max_width(400.0)
        .show(ctx, |ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 3.0);
            ui.add(StatsEdit::new(self.locale, &mut self.crafter_config));
        });

        egui::Window::new(
            egui::RichText::new("Saved macros & solve history")
                .strong()
                .text_style(TextStyle::Body),
        )
        .open(&mut self.saved_rotations_window_open)
        .collapsible(false)
        .default_size((400.0, 600.0))
        .show(ctx, |ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 3.0);
            ui.add(SavedRotationsWidget::new(
                self.locale,
                &mut self.saved_rotations_config,
                &mut self.saved_rotations_data,
                &mut self.actions,
                &mut self.crafter_config,
                &mut self.recipe_config,
                &mut self.custom_recipe_overrides_config,
                &mut self.selected_food,
                &mut self.selected_potion,
            ));
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "LOCALE", &self.locale);
        eframe::set_value(storage, "APP_CONFIG", &self.app_config);
        eframe::set_value(storage, "RECIPE_CONFIG", &self.recipe_config);
        eframe::set_value(
            storage,
            "CUSTOM_RECIPE_OVERRIDES_CONFIG",
            &self.custom_recipe_overrides_config,
        );
        eframe::set_value(storage, "SELECTED_FOOD", &self.selected_food);
        eframe::set_value(storage, "SELECTED_POTION", &self.selected_potion);
        eframe::set_value(storage, "CRAFTER_CONFIG", &self.crafter_config);
        eframe::set_value(storage, "SOLVER_CONFIG", &self.solver_config);
        eframe::set_value(storage, "MACRO_VIEW_CONFIG", &self.macro_view_config);
        eframe::set_value(
            storage,
            "SAVED_ROTATIONS_CONFIG",
            &self.saved_rotations_config,
        );
        eframe::set_value(storage, "SAVED_ROTATIONS", &self.saved_rotations_data);
    }

    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(1)
    }
}

impl MacroSolverApp {
    fn process_solver_events(&mut self) {
        let mut solver_events = self.solver_events.lock().unwrap();
        while let Some(event) = solver_events.pop_front() {
            match event {
                SolverEvent::NodesVisited(count) => self.solver_progress = count,
                SolverEvent::Actions(actions) => self.actions = actions,
                SolverEvent::LoadedFromHistory() => self.solver_progress = usize::MAX,
                SolverEvent::Finished(exception) => {
                    self.duration = self.start_time.elapsed();
                    self.solver_pending = false;
                    self.solver_interrupt.clear();
                    if exception.is_none() {
                        let mut game_settings = raphael_data::get_game_settings(
                            self.recipe_config.recipe,
                            match self.custom_recipe_overrides_config.use_custom_recipe {
                                true => Some(
                                    self.custom_recipe_overrides_config.custom_recipe_overrides,
                                ),
                                false => None,
                            },
                            self.crafter_config.crafter_stats
                                [self.crafter_config.selected_job as usize],
                            self.selected_food,
                            self.selected_potion,
                        );
                        game_settings.adversarial = self.solver_config.adversarial;
                        game_settings.backload_progress = self.solver_config.backload_progress;
                        let new_rotation = Rotation::new(
                            raphael_data::get_item_name(
                                self.recipe_config.recipe.item_id,
                                false,
                                self.locale,
                            )
                            .unwrap_or("Unknown item".to_owned()),
                            self.actions.clone(),
                            &self.recipe_config,
                            &self.custom_recipe_overrides_config,
                            &game_settings,
                            &self.solver_config,
                            self.selected_food,
                            self.selected_potion,
                            &self.crafter_config,
                        );
                        self.saved_rotations_data
                            .add_solved_rotation(new_rotation, &self.saved_rotations_config);
                    } else {
                        self.solver_error = exception;
                    }
                }
            }
        }
    }

    fn draw_app_config_menu_button(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.add_enabled_ui(true, |ui| {
            ui.reset_style();
            egui::containers::menu::MenuButton::new("⚙ Settings")
                .config(
                    egui::containers::menu::MenuConfig::default()
                        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside),
                )
                .ui(ui, |ui| {
                    ui.reset_style();
                    ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 3.0);
                    ui.horizontal(|ui| {
                        ui.label("Zoom");

                        let mut zoom_percentage = (ctx.zoom_factor() * 100.0).round() as u16;
                        ui.horizontal(|ui| {
                            ui.style_mut().spacing.item_spacing.x = 4.0;
                            ui.add_enabled_ui(zoom_percentage > 50, |ui| {
                                if ui.button(egui::RichText::new("-").monospace()).clicked() {
                                    zoom_percentage -= 10;
                                }
                            });
                            ui.add_enabled_ui(zoom_percentage != 100, |ui| {
                                if ui.button("Reset").clicked() {
                                    zoom_percentage = 100;
                                }
                            });
                            ui.add_enabled_ui(zoom_percentage < 500, |ui| {
                                if ui.button(egui::RichText::new("+").monospace()).clicked() {
                                    zoom_percentage += 10;
                                }
                            });
                        });

                        ui.add(
                            egui::DragValue::new(&mut zoom_percentage)
                                .range(50..=500)
                                .suffix("%")
                                // dragging would cause the UI scale to jump arround erratically
                                .speed(0.0)
                                .update_while_editing(false),
                        );

                        self.app_config.zoom_percentage = zoom_percentage;
                        ctx.set_zoom_factor(f32::from(zoom_percentage) * 0.01);
                    });

                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("Theme");
                        egui::global_theme_preference_buttons(ui);
                    });
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Max solver threads");
                        ui.add_enabled_ui(!thread_pool::initialization_attempted(), |ui| {
                            let mut auto_thread_count = self.app_config.num_threads.is_none();
                            if ui.checkbox(&mut auto_thread_count, "Auto").changed() {
                                if auto_thread_count {
                                    self.app_config.num_threads = None;
                                } else {
                                    self.app_config.num_threads =
                                        Some(thread_pool::default_thread_count());
                                }
                            }
                            if thread_pool::is_initialized() {
                                ui.add_enabled(
                                    false,
                                    egui::DragValue::new(&mut rayon::current_num_threads()),
                                );
                            } else if let Some(num_threads) = self.app_config.num_threads.as_mut() {
                                ui.add(egui::DragValue::new(num_threads));
                            } else {
                                ui.add_enabled(
                                    false,
                                    egui::DragValue::new(&mut thread_pool::default_thread_count()),
                                );
                            }
                        });
                    });
                    if thread_pool::initialization_attempted() {
                        #[cfg(target_arch = "wasm32")]
                        let app_restart_text = "Reload the page to change max solver threads.";
                        #[cfg(not(target_arch = "wasm32"))]
                        let app_restart_text = "Restart the app to change max solver threads.";
                        ui.label(
                            egui::RichText::new("⚠ Unavailable after the solver was started.")
                                .small()
                                .color(ui.visuals().warn_fg_color),
                        );
                        ui.label(
                            egui::RichText::new(app_restart_text)
                                .small()
                                .color(ui.visuals().warn_fg_color),
                        );
                    }
                });
        });
    }

    fn draw_simulator_widget(&mut self, ui: &mut egui::Ui) {
        let game_settings = util::get_game_settings(
            &self.recipe_config,
            &self.custom_recipe_overrides_config,
            &self.solver_config,
            &self.crafter_config,
            self.selected_food,
            self.selected_potion,
        );
        let initial_quality = util::get_initial_quality(&self.recipe_config, &self.crafter_config);
        let item = raphael_data::ITEMS
            .get(&self.recipe_config.recipe.item_id)
            .copied()
            .unwrap_or_default();
        ui.add(Simulator::new(
            &game_settings,
            initial_quality,
            self.solver_config,
            &self.crafter_config,
            &self.actions,
            &item,
            self.locale,
        ));
    }

    fn draw_list_select_widgets(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.add(RecipeSelect::new(
                &mut self.crafter_config,
                &mut self.recipe_config,
                &mut self.custom_recipe_overrides_config,
                self.selected_food,
                self.selected_potion,
                self.locale,
            ));
            ui.add(FoodSelect::new(
                self.crafter_config.crafter_stats[self.crafter_config.selected_job as usize],
                &mut self.selected_food,
                self.locale,
            ));
            ui.add(PotionSelect::new(
                self.crafter_config.crafter_stats[self.crafter_config.selected_job as usize],
                &mut self.selected_potion,
                self.locale,
            ));
        });
    }

    fn draw_config_and_results_widget(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 3.0);
            ui.vertical(|ui| {
                self.draw_configuration_widget(ui);
                ui.separator();
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui.button("📑").clicked() {
                        self.saved_rotations_window_open = true;
                    }
                    ui.add_space(-5.0);
                    ui.vertical_centered_justified(|ui| {
                        let text_color = ui.ctx().style().visuals.selection.stroke.color;
                        let text = egui::RichText::new("Solve").color(text_color);
                        let fill_color = ui.ctx().style().visuals.selection.bg_fill;
                        let id = egui::Id::new("SOLVE_INITIATED");
                        let mut solve_initiated = ui
                            .ctx()
                            .data(|data| data.get_temp::<bool>(id).unwrap_or_default());
                        let button = ui.add_enabled(
                            !solve_initiated,
                            egui::Button::new(text).fill(fill_color),
                        );
                        if button.clicked() {
                            ui.ctx().data_mut(|data| {
                                data.insert_temp(id, true);
                            });
                            solve_initiated = true;
                        }
                        if solve_initiated {
                            self.on_solve_initiated(ui.ctx());
                        }
                    });
                });
                ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                    if self.solver_progress == usize::MAX {
                        ui.label("Loaded from saved rotations");
                    } else if self.solver_progress > 0 {
                        ui.label(format!("Elapsed time: {:.2}s", self.duration.as_secs_f32()));
                    }
                });
                // fill the remaining space
                ui.with_layout(Layout::bottom_up(Align::LEFT), |_| {});
            });
        });
    }

    fn draw_configuration_widget(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Configuration").strong());
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.style_mut().spacing.item_spacing = [4.0, 4.0].into();
                if ui.button("✏").clicked() {
                    self.stats_edit_window_open = true;
                }
                egui::ComboBox::from_id_salt("SELECTED_JOB")
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
            ui.label("Craftsmanship");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let cms_base = &mut self.crafter_config.active_stats_mut().craftsmanship;
                let cms_bonus = raphael_data::craftsmanship_bonus(
                    *cms_base,
                    &[self.selected_food, self.selected_potion],
                );
                let mut cms_total = *cms_base + cms_bonus;
                ui.style_mut().spacing.item_spacing.x = 5.0;
                ui.add_enabled(false, egui::DragValue::new(&mut cms_total));
                ui.label("➡");
                ui.add(egui::DragValue::new(cms_base).range(0..=9000));
            });
        });
        ui.horizontal(|ui| {
            ui.label("Control");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let control_base = &mut self.crafter_config.active_stats_mut().control;
                let control_bonus = raphael_data::control_bonus(
                    *control_base,
                    &[self.selected_food, self.selected_potion],
                );
                let mut control_total = *control_base + control_bonus;
                ui.style_mut().spacing.item_spacing.x = 5.0;
                ui.add_enabled(false, egui::DragValue::new(&mut control_total));
                ui.label("➡");
                ui.add(egui::DragValue::new(control_base).range(0..=9000));
            });
        });
        ui.horizontal(|ui| {
            ui.label("CP");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let cp_base = &mut self.crafter_config.active_stats_mut().cp;
                let cp_bonus =
                    raphael_data::cp_bonus(*cp_base, &[self.selected_food, self.selected_potion]);
                let mut cp_total = *cp_base + cp_bonus;
                ui.style_mut().spacing.item_spacing.x = 5.0;
                ui.add_enabled(false, egui::DragValue::new(&mut cp_total));
                ui.label("➡");
                ui.add(egui::DragValue::new(cp_base).range(0..=9000));
            });
        });
        ui.horizontal(|ui| {
            ui.label("Job level");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add(
                    egui::DragValue::new(&mut self.crafter_config.active_stats_mut().level)
                        .range(1..=100),
                );
            });
        });
        ui.separator();

        ui.label(egui::RichText::new("HQ materials").strong());
        let mut has_hq_ingredient = false;
        let recipe_ingredients = self.recipe_config.recipe.ingredients;
        if let QualitySource::HqMaterialList(provided_ingredients) =
            &mut self.recipe_config.quality_source
        {
            for (index, ingredient) in recipe_ingredients.into_iter().enumerate() {
                if let Some(item) = raphael_data::ITEMS.get(&ingredient.item_id) {
                    if item.can_be_hq {
                        has_hq_ingredient = true;
                        ui.horizontal(|ui| {
                            ui.add(ItemNameLabel::new(ingredient.item_id, false, self.locale));
                            ui.with_layout(
                                Layout::right_to_left(Align::Center),
                                |ui: &mut egui::Ui| {
                                    let mut max_placeholder = ingredient.amount;
                                    ui.add_enabled(
                                        false,
                                        egui::DragValue::new(&mut max_placeholder),
                                    );
                                    ui.monospace("/");
                                    ui.add(
                                        egui::DragValue::new(&mut provided_ingredients[index])
                                            .range(0..=ingredient.amount),
                                    );
                                },
                            );
                        });
                    }
                }
            }
        }
        if !has_hq_ingredient {
            ui.label("None");
        }
        ui.separator();

        ui.label(egui::RichText::new("Actions").strong());
        if self.crafter_config.active_stats().level >= Manipulation::LEVEL_REQUIREMENT {
            ui.add(egui::Checkbox::new(
                &mut self.crafter_config.active_stats_mut().manipulation,
                action_name(Action::Manipulation, self.locale),
            ));
        } else {
            ui.add_enabled(
                false,
                egui::Checkbox::new(&mut false, action_name(Action::Manipulation, self.locale)),
            );
        }
        if self.crafter_config.active_stats().level >= HeartAndSoul::LEVEL_REQUIREMENT {
            ui.add(egui::Checkbox::new(
                &mut self.crafter_config.active_stats_mut().heart_and_soul,
                action_name(Action::HeartAndSoul, self.locale),
            ));
        } else {
            ui.add_enabled(
                false,
                egui::Checkbox::new(&mut false, action_name(Action::HeartAndSoul, self.locale)),
            );
        }
        if self.crafter_config.active_stats().level >= QuickInnovation::LEVEL_REQUIREMENT {
            ui.add(egui::Checkbox::new(
                &mut self.crafter_config.active_stats_mut().quick_innovation,
                action_name(Action::QuickInnovation, self.locale),
            ));
        } else {
            ui.add_enabled(
                false,
                egui::Checkbox::new(
                    &mut false,
                    action_name(Action::QuickInnovation, self.locale),
                ),
            );
        }
        let heart_and_soul_enabled = self.crafter_config.active_stats().level
            >= HeartAndSoul::LEVEL_REQUIREMENT
            && self.crafter_config.active_stats_mut().heart_and_soul;
        let quick_innovation_enabled = self.crafter_config.active_stats().level
            >= QuickInnovation::LEVEL_REQUIREMENT
            && self.crafter_config.active_stats_mut().quick_innovation;
        if heart_and_soul_enabled || quick_innovation_enabled {
            #[cfg(not(target_arch = "wasm32"))]
            ui.label(
                egui::RichText::new(
                    "⚠ Specialist actions substantially increase solve time and memory usage.",
                )
                .small()
                .color(ui.visuals().warn_fg_color),
            );
            #[cfg(target_arch = "wasm32")]
            {
                ui.label(
                    egui::RichText::new(
                        "⚠ Specialist actions substantially increase solve time and memory usage. It is recommended that you download and use the native version if you want to enable specialist actions.",
                    )
                    .small()
                    .color(ui.visuals().warn_fg_color),
                );
                ui.add(egui::Hyperlink::from_label_and_url(
                    egui::RichText::new("Download latest release from GitHub").small(),
                    "https://github.com/KonaeAkira/raphael-rs/releases/latest",
                ));
            }
        }
        ui.separator();

        ui.label(egui::RichText::new("Solver settings").strong());
        ui.horizontal(|ui| {
            ui.label("Target quality");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.style_mut().spacing.item_spacing = [4.0, 4.0].into();
                let game_settings = raphael_data::get_game_settings(
                    self.recipe_config.recipe,
                    match self.custom_recipe_overrides_config.use_custom_recipe {
                        true => Some(self.custom_recipe_overrides_config.custom_recipe_overrides),
                        false => None,
                    },
                    self.crafter_config.crafter_stats[self.crafter_config.selected_job as usize],
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
                }
                egui::ComboBox::from_id_salt("TARGET_QUALITY")
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

        ui.horizontal(|ui| {
            ui.checkbox(
                &mut self.solver_config.backload_progress,
                "Backload progress",
            );
            ui.add(HelpText::new("Find a rotation that only uses Progress-increasing actions at the end of the rotation.\n  - May decrease achievable Quality.\n  - May increase macro duration."));
        });

        if self.recipe_config.recipe.is_expert {
            self.solver_config.adversarial = false;
        }
        ui.horizontal(|ui| {
            ui.add_enabled(
                !self.recipe_config.recipe.is_expert,
                egui::Checkbox::new(
                    &mut self.solver_config.adversarial,
                    "Ensure 100% reliability",
                ),
            );
            ui.add(HelpText::new("Find a rotation that can reach the target quality no matter how unlucky the random conditions are.\n  - May decrease achievable Quality.\n  - May increase macro duration.\n  - Much longer solve time.\nThe solver never tries to use Tricks of the Trade to \"eat\" Excellent quality procs, so in some cases this option does not produce the optimal macro."));
        });
        if self.solver_config.adversarial {
            ui.label(
                egui::RichText::new(Self::experimental_warning_text())
                    .small()
                    .color(ui.visuals().warn_fg_color),
            );
        }
    }

    fn on_solve_initiated(&mut self, ctx: &egui::Context) {
        if thread_pool::is_initialized() {
            ctx.data_mut(|data| {
                data.insert_temp(Id::new("SOLVE_INITIATED"), false);
            });

            let craftsmanship_req = self.recipe_config.recipe.req_craftsmanship;
            let control_req = self.recipe_config.recipe.req_control;
            let craftsmanship = self.crafter_config.active_stats().craftsmanship;
            let control = self.crafter_config.active_stats().control;
            let craftsmanship_bonus = raphael_data::craftsmanship_bonus(
                craftsmanship,
                &[self.selected_food, self.selected_potion],
            );
            let control_bonus =
                raphael_data::control_bonus(control, &[self.selected_food, self.selected_potion]);
            if craftsmanship + craftsmanship_bonus >= craftsmanship_req
                && control + control_bonus >= control_req
            {
                self.solve(ctx);
            } else {
                self.missing_stats_error_window_open = true;
            }
        } else {
            thread_pool::attempt_initialization(self.app_config.num_threads);
            ctx.request_repaint();
        }
    }

    fn solve(&mut self, ctx: &egui::Context) {
        self.solver_pending = true;
        self.solver_interrupt.clear();

        let mut game_settings = util::get_game_settings(
            &self.recipe_config,
            &self.custom_recipe_overrides_config,
            &self.solver_config,
            &self.crafter_config,
            self.selected_food,
            self.selected_potion,
        );
        let initial_quality = util::get_initial_quality(&self.recipe_config, &self.crafter_config);
        ctx.data_mut(|data| {
            data.insert_temp(
                Id::new("LAST_SOLVE_PARAMS"),
                (game_settings, initial_quality, self.solver_config),
            );
        });

        if self.saved_rotations_config.load_from_saved_rotations
            && let Some(actions) = self.saved_rotations_data.find_solved_rotation(
                &game_settings,
                initial_quality,
                &self.solver_config,
            )
        {
            let mut solver_events = self.solver_events.lock().unwrap();
            solver_events.push_back(SolverEvent::Actions(actions));
            solver_events.push_back(SolverEvent::LoadedFromHistory());
            solver_events.push_back(SolverEvent::Finished(None));
        } else {
            let target_quality = self
                .solver_config
                .quality_target
                .get_target(game_settings.max_quality);
            game_settings.max_quality = target_quality.saturating_sub(initial_quality) as u16;
            self.actions = Vec::new();
            self.solver_progress = 0;
            self.start_time = web_time::Instant::now();
            spawn_solver(
                game_settings,
                self.solver_events.clone(),
                self.solver_interrupt.clone(),
            );
        }
    }

    fn draw_macro_output_widget(&mut self, ui: &mut egui::Ui) {
        ui.add(MacroView::new(
            &mut self.actions,
            &mut self.macro_view_config,
            self.locale,
        ));
    }

    fn experimental_warning_text() -> &'static str {
        #[cfg(not(target_arch = "wasm32"))]
        return "⚠ EXPERIMENTAL FEATURE\nThis option may use a lot of memory (sometimes well above 4GB) which may cause your system to run out of memory.";
        #[cfg(target_arch = "wasm32")]
        return "⚠ EXPERIMENTAL FEATURE\nMay crash the solver due to reaching the 4GB memory limit of 32-bit web assembly, causing the UI to get stuck in the \"solving\" state indefinitely.";
    }

    #[cfg(target_arch = "wasm32")]
    fn load_fonts_dyn(&self, ctx: &egui::Context) {
        if self.locale == Locale::JP {
            let uri = concat!(
                env!("BASE_URL"),
                "/fonts/M_PLUS_1_Code/static/MPLUS1Code-Regular.ttf"
            );
            load_font_dyn(ctx, "MPLUS1Code-Regular", uri);
        } else if self.locale == Locale::KR {
            let uri = concat!(
                env!("BASE_URL"),
                "/fonts/Noto_Sans_KR/static/NotoSansKR-Regular.ttf"
            );
            load_font_dyn(ctx, "NotoSansKR-Regular", uri);
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn load_font_dyn(ctx: &egui::Context, font_name: &str, uri: &str) {
    use egui::epaint::text::{FontInsert, FontPriority, InsertFontFamily};
    let id = egui::Id::new(format!("{} loaded", uri));
    if ctx.data(|data| data.get_temp(id).unwrap_or(false)) {
        return;
    }
    if let Ok(egui::load::BytesPoll::Ready { bytes, .. }) = ctx.try_load_bytes(uri) {
        ctx.add_font(FontInsert::new(
            font_name,
            egui::FontData::from_owned(bytes.to_vec()),
            vec![
                InsertFontFamily {
                    family: egui::FontFamily::Proportional,
                    priority: FontPriority::Lowest,
                },
                InsertFontFamily {
                    family: egui::FontFamily::Monospace,
                    priority: FontPriority::Lowest,
                },
            ],
        ));
        ctx.data_mut(|data| *data.get_temp_mut_or_default(id) = true);
        log::debug!("Font loaded: {}", font_name);
    };
}

fn load_fonts(ctx: &egui::Context) {
    use egui::epaint::text::{FontInsert, FontPriority, InsertFontFamily};
    ctx.add_font(FontInsert::new(
        "XIV_Icon_Recreations",
        egui::FontData::from_static(include_bytes!(
            "../assets/fonts/XIV_Icon_Recreations/XIV_Icon_Recreations.ttf"
        )),
        vec![
            InsertFontFamily {
                family: egui::FontFamily::Proportional,
                priority: FontPriority::Lowest,
            },
            InsertFontFamily {
                family: egui::FontFamily::Monospace,
                priority: FontPriority::Lowest,
            },
        ],
    ));
    #[cfg(not(target_arch = "wasm32"))]
    ctx.add_font(FontInsert::new(
        "MPLUS1Code-Regular",
        egui::FontData::from_static(include_bytes!(
            "../assets/fonts/M_PLUS_1_Code/static/MPLUS1Code-Regular.ttf"
        )),
        vec![
            InsertFontFamily {
                family: egui::FontFamily::Proportional,
                priority: FontPriority::Lowest,
            },
            InsertFontFamily {
                family: egui::FontFamily::Monospace,
                priority: FontPriority::Lowest,
            },
        ],
    ));
    #[cfg(not(target_arch = "wasm32"))]
    ctx.add_font(FontInsert::new(
        "NotoSansKR-Regular",
        egui::FontData::from_static(include_bytes!(
            "../assets/fonts/Noto_Sans_KR/static/NotoSansKR-Regular.ttf"
        )),
        vec![
            InsertFontFamily {
                family: egui::FontFamily::Proportional,
                priority: FontPriority::Lowest,
            },
            InsertFontFamily {
                family: egui::FontFamily::Monospace,
                priority: FontPriority::Lowest,
            },
        ],
    ));
}

fn spawn_solver(
    simulator_settings: raphael_sim::Settings,
    solver_events: Arc<Mutex<VecDeque<SolverEvent>>>,
    solver_interrupt: raphael_solver::AtomicFlag,
) {
    let events = solver_events.clone();
    let solution_callback = move |actions: &[raphael_sim::Action]| {
        let event = SolverEvent::Actions(actions.to_vec());
        events.lock().unwrap().push_back(event);
    };
    let events = solver_events.clone();
    let progress_callback = move |progress: usize| {
        let event = SolverEvent::NodesVisited(progress);
        events.lock().unwrap().push_back(event);
    };
    rayon::spawn(move || {
        let solver_settings = raphael_solver::SolverSettings { simulator_settings };
        log::debug!("Spawning solver: {solver_settings:?}");
        let mut macro_solver = raphael_solver::MacroSolver::new(
            solver_settings,
            Box::new(solution_callback),
            Box::new(progress_callback),
            solver_interrupt,
        );
        match macro_solver.solve() {
            Ok(actions) => {
                let mut solver_events = solver_events.lock().unwrap();
                solver_events.push_back(SolverEvent::Actions(actions));
                solver_events.push_back(SolverEvent::Finished(None));
            }
            Err(exception) => solver_events
                .lock()
                .unwrap()
                .push_back(SolverEvent::Finished(Some(exception))),
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn fetch_latest_version(latest_version: Arc<Mutex<semver::Version>>) {
    #[derive(Deserialize)]
    struct ApiResponse {
        tag_name: String,
    }
    let request =
        ehttp::Request::get("https://api.github.com/repos/KonaeAkira/raphael-rs/releases/latest");
    ehttp::fetch(
        request,
        move |result: ehttp::Result<ehttp::Response>| match result {
            Ok(response) => match response.json::<ApiResponse>() {
                Ok(data) => match semver::Version::parse(data.tag_name.trim_start_matches('v')) {
                    Ok(version) => {
                        log::debug!("Latest version: {}", version);
                        *latest_version.lock().unwrap() = version;
                    }
                    Err(err) => log::error!("{err}"),
                },
                Err(err) => log::error!("{err}"),
            },
            Err(err) => log::error!("{err}"),
        },
    );
}
