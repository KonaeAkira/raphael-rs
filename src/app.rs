use raphael_solver::SolverException;
use raphael_translations::{t, t_format};

use egui::{Align, CursorIcon, Layout, TextStyle};
use raphael_data::{Locale, action_name, get_job_name};

use raphael_sim::{Action, ActionImpl, HeartAndSoul, Manipulation, QuickInnovation};

use crate::config::{QualitySource, QualityTarget};
use crate::context::AppContext;
use crate::fonts::FontLoadingState;
use crate::solve::{LastSolveInfo, RunningSolveInfo, SolveState};
use crate::{
    elements::{panels::*, widgets::*},
    thread_pool,
};

pub struct MacroSolverApp {
    app_context: AppContext,

    stats_edit_window_open: bool,
    saved_rotations_window_open: bool,
    missing_stats_error_window_open: bool,

    solve_state: SolveState,

    font_loading_state: FontLoadingState,

    #[cfg(any(debug_assertions, feature = "dev-panel"))]
    render_info: RenderInfo,
}

impl MacroSolverApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let app_context = AppContext::new(cc);
        cc.egui_ctx
            .set_zoom_factor(f32::from(app_context.app_config.zoom_percentage) * 0.01);

        cc.egui_ctx.all_styles_mut(|style| {
            style.visuals.interact_cursor = Some(CursorIcon::PointingHand);
            style.url_in_tooltip = true;
            style.always_scroll_the_only_direction = false;
            style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        });
        // Force scroll area state to be effectively temporary
        cc.egui_ctx
            .data_mut(egui::util::IdTypeMap::remove_by_type::<egui::scroll_area::State>);

        let font_loading_state = FontLoadingState::new(&cc.egui_ctx, app_context.locale);

        #[cfg(not(target_arch = "wasm32"))]
        crate::update::check_for_update();

        Self {
            app_context,

            stats_edit_window_open: false,
            saved_rotations_window_open: false,
            missing_stats_error_window_open: false,

            solve_state: SolveState::default(),

            font_loading_state,

            #[cfg(any(debug_assertions, feature = "dev-panel"))]
            render_info: RenderInfo::default(),
        }
    }
}

impl eframe::App for MacroSolverApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let locale = self.app_context.locale;

        self.solve_state
            .process_solver_events(&mut self.app_context);

        #[cfg(not(target_arch = "wasm32"))]
        crate::update::show_dialogues(ui, locale);

        if self.missing_stats_error_window_open {
            egui::Modal::new(egui::Id::new("min_stats_warning")).show(ui, |ui| {
                let req_cms = self.app_context.recipe_config.recipe().req_craftsmanship;
                let req_ctrl = self.app_context.recipe_config.recipe().req_control;
                ui.style_mut().spacing.item_spacing = egui::vec2(3.0, 3.0);
                ui.label(egui::RichText::new("Error").strong());
                ui.separator();
                ui.label(t!(
                    locale,
                    "Your stats are below the minimum requirement for this recipe."
                ));
                ui.label(t_format!(
                    locale,
                    "Requirement: {req_cms} Craftsmanship, {req_ctrl} Control."
                ));
                ui.separator();
                ui.vertical_centered_justified(|ui| {
                    if ui.button(t!(locale, "Close")).clicked() {
                        self.missing_stats_error_window_open = false;
                    }
                });
            });
        }

        if let Some(error) = self.solve_state.solver_error().cloned() {
            egui::Modal::new(egui::Id::new("solver_error")).show(ui, |ui| {
                ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 3.0);
                ui.set_width(480.0f32.min(ui.content_rect().width() - 32.0));
                match error {
                    SolverException::NoSolution => {
                        ui.label(egui::RichText::new(t!(locale, "No solution")).strong());
                        ui.separator();
                        ui.label(t!(locale, "Cannot complete synthesis."));
                        self.solve_state.reset_actions();
                        if self.app_context.solver_config.must_reach_target_quality
                            && self.app_context.game_settings().max_quality != 0
                        {
                            ui.label(t!(locale, "Try lowering target quality."));
                        }
                    }
                    SolverException::Interrupted => {
                        self.solve_state.resolve_error();
                    }
                    SolverException::SearchQueueCapacityExceeded => {
                        ui.label(
                            egui::RichText::new(t!(locale, "Search Queue Capacity Exceeded"))
                                .strong(),
                        );
                        ui.separator();
                        ui.label(t!(locale, "The number of nodes in the search queue exceeded the limit of the 32-bit web version."));
                        ui.label(t!(locale, "Solving this configuration requires a 64-bit version of Raphael."));
                        ui.add(egui::Hyperlink::from_label_and_url(
                            egui::RichText::new(t!(locale, "Download latest release from GitHub"))
                                .small(),
                            "https://github.com/KonaeAkira/raphael-rs/releases/latest",
                        ));
                    }
                    SolverException::InternalError(message) => {
                        ui.label(egui::RichText::new(t!(locale, "Internal Solver Error")).strong());
                        ui.separator();
                        ui.add(
                            MultilineMonospace::new("SOLVER_ERROR".into(), message)
                                .max_height(320.0)
                                .scrollable(true),
                        );
                    }
                }
                ui.separator();
                ui.vertical_centered_justified(|ui| {
                    if ui.button(t!(locale, "Close")).clicked() {
                        self.solve_state.resolve_error();
                    }
                });
            });
        }

        if let Some(RunningSolveInfo {
            start_time,
            solver_progress,
            ..
        }) = self.solve_state.running_solve_info()
        {
            let running_duration = start_time.elapsed().as_secs_f32();
            let solver_progress = *solver_progress;
            #[cfg(target_arch = "wasm32")]
            if crate::OOM_PANIC_OCCURED.load(std::sync::atomic::Ordering::Relaxed) {
                eframe::wasm_bindgen::throw_val("OOM panic".into());
            }
            let interrupt_pending = self.solve_state.interrupted();
            egui::Modal::new(egui::Id::new("solver_busy")).show(ui, |ui| {
                ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 3.0);
                ui.set_width(180.0);
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(if interrupt_pending {
                                    t!(locale, "Cancelling ...")
                                } else {
                                    t!(locale, "Solving ...")
                                })
                                .strong(),
                            );
                            ui.label(format!("({:.2}s)", running_duration));
                        });
                        if solver_progress == 0 {
                            ui.label(t!(locale, "Computing ..."));
                        } else {
                            // format with thousands separator
                            let num = solver_progress
                                .to_string()
                                .as_bytes()
                                .rchunks(3)
                                .rev()
                                .map(std::str::from_utf8)
                                .collect::<Result<Vec<&str>, _>>()
                                .unwrap()
                                .join(",");
                            ui.label(t_format!(locale, "{num} nodes visited"));
                        }
                    });
                });

                ui.vertical_centered_justified(|ui| {
                    ui.separator();
                    let response =
                        ui.add_enabled(!interrupt_pending, egui::Button::new(t!(locale, "Cancel")));
                    if response.clicked() {
                        self.solve_state.interrupt();
                    }
                });
            });
        }

        egui::Panel::top("top_panel").show_inside(ui, |ui| {
            egui::ScrollArea::horizontal()
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .show(ui, |ui| {
                    egui::containers::menu::MenuBar::new().ui(ui, |ui| {
                        ui.label(egui::RichText::new("Raphael  |  FFXIV Crafting Solver").strong());
                        ui.label(format!("v{}", env!("CARGO_PKG_VERSION")));
                        self.draw_app_config_menu_button(ui);

                        let selectable_locales = [
                            Locale::EN,
                            Locale::DE,
                            Locale::FR,
                            Locale::JP,
                            Locale::CN,
                            Locale::KR,
                            Locale::TW,
                        ];
                        ui.add(DropDown::new(
                            "LOCALE",
                            &mut self.app_context.locale,
                            selectable_locales,
                            Locale::short_code,
                        ));
                        let locale = self.app_context.locale;
                        if self.font_loading_state.loaded_fonts_for_locale != locale {
                            self.font_loading_state.load_fonts(ui.ctx(), locale, false);
                            if self.font_loading_state.loaded_fonts_for_locale == locale {
                                ui.ctx().request_discard("font change");
                            }
                        }

                        ui.add(
                            egui::Hyperlink::from_label_and_url(
                                t!(locale, "View source on GitHub"),
                                "https://github.com/KonaeAkira/raphael-rs",
                            )
                            .open_in_new_tab(true),
                        );
                        ui.label("/");
                        ui.add(
                            egui::Hyperlink::from_label_and_url(
                                t!(locale, "Join Discord"),
                                "https://discord.com/invite/m2aCy3y8he",
                            )
                            .open_in_new_tab(true),
                        );
                        ui.label("/");
                        ui.add(
                            egui::Hyperlink::from_label_and_url(
                                t!(locale, "Support me on Ko-fi"),
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
                                .selectable_label(self.render_info.state.shown, "Dev Panel")
                                .clicked()
                            {
                                self.render_info.state.shown = !self.render_info.state.shown;
                            }
                            egui::warn_if_debug_build(ui);
                            ui.separator();
                        });
                    });
                });
        });

        #[cfg(any(debug_assertions, feature = "dev-panel"))]
        if self.render_info.state.shown {
            egui::Panel::right("dev_panel")
                .resizable(true)
                .show_inside(ui, |ui| {
                    ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 3.0);
                    self.render_info.ui(ui, _frame);
                });
        }

        egui::CentralPanel::default().show_inside(ui, |ui| {
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
            });
        });

        let maximum_visible_window_size =
            (ui.content_rect().size() - egui::Vec2::new(14.0, 45.0)).max(egui::Vec2::ZERO);
        let stats_edit_window_size = maximum_visible_window_size.min(egui::Vec2::new(412.0, 650.0));
        egui::Window::new(
            egui::RichText::new(t!(locale, "Crafter stats"))
                .strong()
                .text_style(TextStyle::Body),
        )
        .id(egui::Id::new("STATS_EDIT"))
        .open(&mut self.stats_edit_window_open)
        .collapsible(false)
        .resizable(false)
        .min_size(stats_edit_window_size)
        .max_size(stats_edit_window_size)
        .show(ui, |ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 3.0);
            ui.add(StatsEdit::new(&mut self.app_context));
        });

        egui::Window::new(
            egui::RichText::new(t!(locale, "Saved macros & solve history"))
                .strong()
                .text_style(TextStyle::Body),
        )
        .id(egui::Id::new("SAVED_ROTATIONS"))
        .open(&mut self.saved_rotations_window_open)
        .collapsible(false)
        .default_size((400.0, 600.0))
        .show(ui, |ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 3.0);
            ui.add(SavedRotationsWidget::new(
                &mut self.app_context,
                self.solve_state.actions_mut(),
            ));
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.app_context.save(storage);
    }

    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(1)
    }
}

impl MacroSolverApp {
    fn draw_app_config_menu_button(&mut self, ui: &mut egui::Ui) {
        let locale = self.app_context.locale;
        ui.add_enabled_ui(true, |ui| {
            ui.reset_style();
            egui::containers::menu::MenuButton::new(t!(locale, "⚙ Settings"))
                .config(
                    egui::containers::menu::MenuConfig::default()
                        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside),
                )
                .ui(ui, |ui| {
                    ui.reset_style();
                    ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 3.0);
                    ui.horizontal(|ui| {
                        ui.label(t!(locale, "Zoom"));

                        let mut zoom_percentage = (ui.zoom_factor() * 100.0).round() as u16;
                        ui.horizontal(|ui| {
                            ui.style_mut().spacing.item_spacing.x = 4.0;
                            ui.add_enabled_ui(zoom_percentage > 50, |ui| {
                                if ui.button(egui::RichText::new("-").monospace()).clicked() {
                                    zoom_percentage -= 10;
                                }
                            });
                            ui.add_enabled_ui(zoom_percentage != 100, |ui| {
                                if ui.button(t!(locale, "Reset")).clicked() {
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

                        self.app_context.app_config.zoom_percentage = zoom_percentage;
                        ui.set_zoom_factor(f32::from(zoom_percentage) * 0.01);
                    });

                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label(t!(locale, "Theme"));
                        egui::global_theme_preference_buttons(ui);
                    });
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label(t!(locale, "Max solver threads"));
                        ui.add_enabled_ui(!thread_pool::initialization_attempted(), |ui| {
                            let mut auto_thread_count =
                                self.app_context.app_config.num_threads.is_none();
                            if ui
                                .checkbox(&mut auto_thread_count, t!(locale, "Auto"))
                                .changed()
                            {
                                if auto_thread_count {
                                    self.app_context.app_config.num_threads = None;
                                } else {
                                    self.app_context.app_config.num_threads =
                                        Some(thread_pool::default_thread_count());
                                }
                            }
                            if thread_pool::is_initialized() {
                                ui.add_enabled(
                                    false,
                                    egui::DragValue::new(&mut rayon::current_num_threads()),
                                );
                            } else if let Some(num_threads) =
                                self.app_context.app_config.num_threads.as_mut()
                            {
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
                        let app_restart_text =
                            t!(locale, "Reload the page to change max solver threads.");
                        #[cfg(not(target_arch = "wasm32"))]
                        let app_restart_text =
                            t!(locale, "Restart the app to change max solver threads.");
                        ui.label(
                            egui::RichText::new(t!(
                                locale,
                                "⚠ Unavailable after the solver was started."
                            ))
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
        ui.add(Simulator::new(&self.app_context, &self.solve_state));
    }

    fn draw_list_select_widgets(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.add(RecipeSelect::new(&mut self.app_context));
            ui.add(FoodSelect::new(&mut self.app_context));
            ui.add(PotionSelect::new(&mut self.app_context));
        });
    }

    fn draw_config_and_results_widget(&mut self, ui: &mut egui::Ui) {
        let locale = self.app_context.locale;
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
                        let text_color = ui.global_style().visuals.selection.stroke.color;
                        let text = egui::RichText::new(t!(locale, "Solve")).color(text_color);
                        let fill_color = ui.global_style().visuals.selection.bg_fill;
                        let solve_pending = self.solve_state.pending();
                        let button = ui
                            .add_enabled(!solve_pending, egui::Button::new(text).fill(fill_color));
                        if button.clicked() || solve_pending {
                            self.on_solve_initiated();

                            ui.ctx().request_repaint();
                        }
                    });
                });
                if let Some(LastSolveInfo {
                    duration,
                    loaded_from_history,
                    ..
                }) = self.solve_state.last_solve_info()
                {
                    ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                        if *loaded_from_history {
                            ui.label(t!(locale, "Loaded from saved rotations"));
                        } else {
                            ui.label(t_format!(
                                locale,
                                "Elapsed time: {dur:.2}s",
                                dur = duration.as_secs_f32()
                            ));
                        }
                    });
                }
                // fill the remaining space
                ui.with_layout(Layout::bottom_up(Align::LEFT), |_| {});
            });
        });
    }

    fn draw_configuration_widget(&mut self, ui: &mut egui::Ui) {
        let AppContext {
            locale,
            selected_food,
            selected_potion,
            ..
        } = self.app_context;
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(t!(locale, "Configuration")).strong());
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.style_mut().spacing.item_spacing = [4.0, 4.0].into();
                if ui.button("✏").clicked() {
                    self.stats_edit_window_open = true;
                }
                ui.add(DropDown::new(
                    "SELECTED_JOB",
                    self.app_context.selected_job_mut(),
                    [0, 1, 2, 3, 4, 5, 6, 7],
                    |job_id: u8| get_job_name(job_id, locale),
                ));
            });
        });
        ui.separator();

        const BUFFED_STAT_BG_COLOR: egui::Color32 =
            egui::Color32::from_rgba_unmultiplied_const(144, 238, 144, 128);
        let consumables = &[selected_food, selected_potion];
        ui.label(egui::RichText::new(t!(locale, "Crafter stats")).strong());
        ui.horizontal(|ui| {
            ui.label(t!(locale, "Craftsmanship"));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.style_mut().spacing.item_spacing.x = 5.0;

                let cms_base = &mut self.app_context.active_stats_mut().craftsmanship;
                ui.scope(|ui| {
                    let cms_bonus = raphael_data::craftsmanship_bonus(*cms_base, consumables);
                    if cms_bonus != 0 {
                        ui.visuals_mut().widgets.inactive.weak_bg_fill = BUFFED_STAT_BG_COLOR;
                        ui.visuals_mut().widgets.hovered.weak_bg_fill = BUFFED_STAT_BG_COLOR;
                        ui.visuals_mut().widgets.active.weak_bg_fill = BUFFED_STAT_BG_COLOR;
                    }

                    let buffed = *cms_base + cms_bonus;
                    let mut final_value = buffed;
                    ui.add(
                        egui::DragValue::new(&mut final_value)
                            .range(0..=9999)
                            .update_while_editing(false),
                    );
                    if final_value != buffed
                        && let Some(unbuffed) =
                            raphael_data::craftsmanship_unbuffed(final_value, consumables)
                    {
                        *cms_base = unbuffed;
                    }
                });
                ui.label("➡");
                ui.add(egui::DragValue::new(cms_base).range(0..=9000));
            });
        });
        ui.horizontal(|ui| {
            ui.label(t!(locale, "Control"));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.style_mut().spacing.item_spacing.x = 5.0;

                let control_base = &mut self.app_context.active_stats_mut().control;
                ui.scope(|ui| {
                    let control_bonus = raphael_data::control_bonus(*control_base, consumables);
                    if control_bonus != 0 {
                        ui.visuals_mut().widgets.inactive.weak_bg_fill = BUFFED_STAT_BG_COLOR;
                        ui.visuals_mut().widgets.hovered.weak_bg_fill = BUFFED_STAT_BG_COLOR;
                        ui.visuals_mut().widgets.active.weak_bg_fill = BUFFED_STAT_BG_COLOR;
                    }

                    let buffed = *control_base + control_bonus;
                    let mut final_value = buffed;
                    ui.add(
                        egui::DragValue::new(&mut final_value)
                            .range(0..=9999)
                            .update_while_editing(false),
                    );
                    if final_value != buffed
                        && let Some(unbuffed) =
                            raphael_data::control_unbuffed(final_value, consumables)
                    {
                        *control_base = unbuffed;
                    }
                });
                ui.label("➡");
                ui.add(egui::DragValue::new(control_base).range(0..=9000));
            });
        });
        ui.horizontal(|ui| {
            ui.label(t!(locale, "CP"));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.style_mut().spacing.item_spacing.x = 5.0;

                let cp_base = &mut self.app_context.active_stats_mut().cp;
                ui.scope(|ui| {
                    let cp_bonus = raphael_data::cp_bonus(*cp_base, consumables);
                    if cp_bonus != 0 {
                        ui.visuals_mut().widgets.inactive.weak_bg_fill = BUFFED_STAT_BG_COLOR;
                        ui.visuals_mut().widgets.hovered.weak_bg_fill = BUFFED_STAT_BG_COLOR;
                        ui.visuals_mut().widgets.active.weak_bg_fill = BUFFED_STAT_BG_COLOR;
                    }

                    let buffed = *cp_base + cp_bonus;
                    let mut final_value = buffed;
                    ui.add(
                        egui::DragValue::new(&mut final_value)
                            .range(0..=9999)
                            .update_while_editing(false),
                    );
                    if final_value != buffed
                        && let Some(unbuffed) = raphael_data::cp_unbuffed(final_value, consumables)
                    {
                        *cp_base = unbuffed;
                    }
                });
                ui.label("➡");
                ui.add(egui::DragValue::new(cp_base).range(0..=9000));
            });
        });
        ui.horizontal(|ui| {
            ui.label(t!(locale, "Job level"));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add(
                    egui::DragValue::new(&mut self.app_context.active_stats_mut().level)
                        .range(1..=100),
                );
            });
        });
        ui.separator();

        ui.label(egui::RichText::new(t!(locale, "HQ materials")).strong());
        let mut has_hq_ingredient = false;
        let recipe_ingredients = self.app_context.recipe_config.recipe().ingredients;
        if let QualitySource::HqMaterialList(provided_ingredients) =
            &mut self.app_context.recipe_config.quality_source
        {
            for (index, ingredient) in recipe_ingredients.into_iter().enumerate() {
                if ingredient.item_id == 0 {
                    continue;
                }
                has_hq_ingredient = true;
                ui.horizontal(|ui| {
                    ui.add(GameDataNameLabel::new(&ingredient, locale));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui: &mut egui::Ui| {
                        let mut max_placeholder = ingredient.amount;
                        ui.add_enabled(false, egui::DragValue::new(&mut max_placeholder));
                        ui.monospace("/");
                        ui.add(
                            egui::DragValue::new(&mut provided_ingredients[index])
                                .range(0..=ingredient.amount),
                        );
                    });
                });
            }
        }
        if !has_hq_ingredient {
            ui.label(t!(locale, "None"));
        }
        ui.separator();

        ui.label(egui::RichText::new(t!(locale, "Actions")).strong());
        if self.app_context.active_stats().level >= Manipulation::LEVEL_REQUIREMENT {
            ui.add(egui::Checkbox::new(
                &mut self.app_context.active_stats_mut().manipulation,
                action_name(Action::Manipulation, locale),
            ));
        } else {
            ui.add_enabled(
                false,
                egui::Checkbox::new(&mut false, action_name(Action::Manipulation, locale)),
            );
        }
        if self.app_context.active_stats().level >= HeartAndSoul::LEVEL_REQUIREMENT {
            ui.add(egui::Checkbox::new(
                &mut self.app_context.active_stats_mut().heart_and_soul,
                action_name(Action::HeartAndSoul, locale),
            ));
        } else {
            ui.add_enabled(
                false,
                egui::Checkbox::new(&mut false, action_name(Action::HeartAndSoul, locale)),
            );
        }
        if self.app_context.active_stats().level >= QuickInnovation::LEVEL_REQUIREMENT {
            ui.add(egui::Checkbox::new(
                &mut self.app_context.active_stats_mut().quick_innovation,
                action_name(Action::QuickInnovation, locale),
            ));
        } else {
            ui.add_enabled(
                false,
                egui::Checkbox::new(&mut false, action_name(Action::QuickInnovation, locale)),
            );
        }
        let mut max_stellar_steady_hand_charges = self
            .app_context
            .recipe_config
            .recipe_source
            .max_stellar_steady_hand_charges();
        if max_stellar_steady_hand_charges > 0 {
            ui.horizontal(|ui| {
                ui.label(action_name(Action::StellarSteadyHand, locale));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui: &mut egui::Ui| {
                    ui.add_enabled(
                        false,
                        egui::DragValue::new(&mut max_stellar_steady_hand_charges),
                    );
                    ui.monospace("/");
                    ui.add(
                        egui::DragValue::new(
                            &mut self.app_context.solver_config.stellar_steady_hand_charges,
                        )
                        .range(0..=max_stellar_steady_hand_charges),
                    );
                });
            });
        } else {
            self.app_context.solver_config.stellar_steady_hand_charges = 0;
        }
        let heart_and_soul_enabled = self.app_context.active_stats().level
            >= HeartAndSoul::LEVEL_REQUIREMENT
            && self.app_context.active_stats().heart_and_soul;
        let quick_innovation_enabled = self.app_context.active_stats().level
            >= QuickInnovation::LEVEL_REQUIREMENT
            && self.app_context.active_stats().quick_innovation;
        if heart_and_soul_enabled || quick_innovation_enabled {
            #[cfg(not(target_arch = "wasm32"))]
            ui.label(
                egui::RichText::new(t!(
                    locale,
                    "⚠ Specialist actions substantially increase solve time and memory usage."
                ))
                .small()
                .color(ui.visuals().warn_fg_color),
            );
            #[cfg(target_arch = "wasm32")]
            {
                ui.label(
                    egui::RichText::new(
                        t!(locale, "⚠ Specialist actions substantially increase solve time and memory usage. It is recommended that you download and use the native version if you want to enable specialist actions."),
                    )
                    .small()
                    .color(ui.visuals().warn_fg_color),
                );
                ui.add(egui::Hyperlink::from_label_and_url(
                    egui::RichText::new(t!(locale, "Download latest release from GitHub")).small(),
                    "https://github.com/KonaeAkira/raphael-rs/releases/latest",
                ));
            }
        }
        ui.separator();

        ui.label(egui::RichText::new(t!(locale, "Solver settings")).strong());
        ui.horizontal(|ui| {
            ui.label(t!(locale, "Target quality"));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.style_mut().spacing.item_spacing = [4.0, 4.0].into();
                let game_settings = self.app_context.game_settings();
                let mut current_value = self
                    .app_context
                    .solver_config
                    .quality_target
                    .get_target(game_settings.max_quality);
                match &mut self.app_context.solver_config.quality_target {
                    QualityTarget::Custom(value) => {
                        ui.add(egui::DragValue::new(value));
                    }
                    _ => {
                        ui.add_enabled(false, egui::DragValue::new(&mut current_value));
                    }
                }
                let selectable_targets = [
                    QualityTarget::Zero,
                    QualityTarget::Half,
                    QualityTarget::CollectableT1,
                    QualityTarget::CollectableT2,
                    QualityTarget::CollectableT3,
                    QualityTarget::Full,
                    QualityTarget::Custom(current_value),
                ];
                ui.add(DropDown::new(
                    "TARGET_QUALITY",
                    &mut self.app_context.solver_config.quality_target,
                    selectable_targets,
                    |value: QualityTarget| value.display(locale),
                ));
            });
        });

        ui.horizontal(|ui| {
            ui.checkbox(
                &mut self
                    .app_context
                    .solver_config
                    .must_reach_target_quality,
                t!(locale, "Solution must reach target quality"),
            );
            ui.add(HelpText::new(t!(locale, "Reduce memory usage by skipping candidate solutions that cannot reach the target quality. Basically, you either get a solution that reaches the target quality or you get no solution at all. If you want to know how close you are to reaching the target quality, keep this option turned off.")));
        });

        ui.horizontal(|ui| {
            ui.checkbox(
                &mut self.app_context.solver_config.backload_progress,
                t!(locale, "Backload progress"),
            );
            ui.add(HelpText::new(t!(locale, "Find a rotation that only uses Progress-increasing actions at the end of the rotation.\n  - May decrease achievable Quality.\n  - May increase macro duration.")));
        });

        if self.app_context.recipe_config.recipe().is_expert {
            self.app_context.solver_config.adversarial = false;
        }
        ui.horizontal(|ui| {
            ui.add_enabled(
                !self.app_context.recipe_config.recipe().is_expert,
                egui::Checkbox::new(
                    &mut self.app_context.solver_config.adversarial,
                    t!(locale, "Ensure 100% reliability"),
                ),
            );
            ui.add(HelpText::new(t!(locale, "Find a rotation that can reach the target quality no matter how unlucky the random conditions are.\n  - May decrease achievable Quality.\n  - May increase macro duration.\n  - Much longer solve time.\nThe solver never tries to use Tricks of the Trade to \"eat\" Excellent quality procs, so in some cases this option does not produce the optimal macro.")));
        });
        if self.app_context.solver_config.adversarial {
            ui.label(
                egui::RichText::new(Self::experimental_warning_text(locale))
                    .small()
                    .color(ui.visuals().warn_fg_color),
            );
        }
    }

    fn on_solve_initiated(&mut self) {
        let craftsmanship_req = self.app_context.recipe_config.recipe().req_craftsmanship;
        let control_req = self.app_context.recipe_config.recipe().req_control;
        let active_stats = self.app_context.active_stats();
        let craftsmanship_bonus = raphael_data::craftsmanship_bonus(
            active_stats.craftsmanship,
            &[
                self.app_context.selected_food,
                self.app_context.selected_potion,
            ],
        );
        let control_bonus = raphael_data::control_bonus(
            active_stats.control,
            &[
                self.app_context.selected_food,
                self.app_context.selected_potion,
            ],
        );
        if active_stats.craftsmanship + craftsmanship_bonus >= craftsmanship_req
            && active_stats.control + control_bonus >= control_req
        {
            self.solve_state.solve(&self.app_context);
        } else {
            self.missing_stats_error_window_open = true;
        }
    }

    fn draw_macro_output_widget(&mut self, ui: &mut egui::Ui) {
        ui.add(MacroView::new(
            &mut self.app_context,
            self.solve_state.actions_mut(),
        ));
    }

    fn experimental_warning_text(locale: Locale) -> &'static str {
        #[cfg(not(target_arch = "wasm32"))]
        return t!(
            locale,
            "⚠ EXPERIMENTAL FEATURE\nThis option may use a lot of memory (sometimes well above 4GB) which may cause your system to run out of memory."
        );
        #[cfg(target_arch = "wasm32")]
        return t!(
            locale,
            "⚠ EXPERIMENTAL FEATURE\nMay crash the solver due to reaching the 4GB memory limit of 32-bit web assembly, causing the UI to get stuck in the \"solving\" state indefinitely."
        );
    }
}
