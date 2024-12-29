use egui_plot::PlotPoints;
use simulator::*;

use crate::config::QualityTarget;

use super::util;

pub struct SolutionAnalysis<'a> {
    settings: Settings,
    initial_quality: u16,
    target_quality: u16,
    actions: &'a [Action],
    is_expert: bool,
}

impl<'a> SolutionAnalysis<'a> {
    pub fn new(
        settings: Settings,
        initial_quality: u16,
        target_quality: u16,
        actions: &'a [Action],
        is_expert: bool,
    ) -> Self {
        Self {
            settings: Settings {
                adversarial: false,
                ..settings
            },
            initial_quality,
            target_quality,
            actions,
            is_expert,
        }
    }
}

impl egui::Widget for SolutionAnalysis<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 3.0);

            let mut collapsed = false;
            let distribution = simulator::quality_probability_distribution(
                self.settings,
                self.actions,
                self.initial_quality,
            );

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    collapsed = util::collapse_button(ui, egui::Id::new("analysis_collapsed"));
                    ui.label(egui::RichText::new("Analysis").strong());
                    if self.is_expert {
                        ui.label("N/A (Expert recipes not supported)");
                    } else if self.actions.is_empty() {
                        ui.label("N/A (No macro to analyze)");
                    } else {
                        ui.label(format!(
                            "{:.2}% chance to reach target Quality ({})",
                            distribution.at_least(self.target_quality) * 100.0,
                            self.target_quality
                        ));
                    }
                });
                if collapsed || self.is_expert {
                    return;
                }
                ui.separator();

                let plot_max_quality =
                    std::cmp::max(self.settings.max_quality, self.target_quality);
                let mut prob_acc = distribution.at_least(plot_max_quality) as f64;

                let mut plot_points = vec![
                    [plot_max_quality as f64, 0.0],
                    [plot_max_quality as f64, prob_acc],
                ];
                for value in distribution
                    .into_iter()
                    .rev()
                    .filter(|value| value.quality < plot_max_quality)
                {
                    plot_points.push([value.quality as f64, prob_acc]);
                    prob_acc = (prob_acc + value.probability as f64).clamp(0.0, 1.0);
                    plot_points.push([value.quality as f64, prob_acc]);
                }
                plot_points.push([0.0, prob_acc]);
                plot_points.push([0.0, 1.0]);
                let line_chart = egui_plot::Line::new(PlotPoints::from(plot_points));

                let grid_marks = vec![
                    egui_plot::GridMark {
                        value: 0.0,
                        step_size: self.settings.max_quality as f64,
                    },
                    egui_plot::GridMark {
                        value: QualityTarget::CollectableT1.get_target(self.settings.max_quality)
                            as f64,
                        step_size: self.settings.max_quality as f64,
                    },
                    egui_plot::GridMark {
                        value: QualityTarget::CollectableT2.get_target(self.settings.max_quality)
                            as f64,
                        step_size: self.settings.max_quality as f64,
                    },
                    egui_plot::GridMark {
                        value: QualityTarget::CollectableT3.get_target(self.settings.max_quality)
                            as f64,
                        step_size: self.settings.max_quality as f64,
                    },
                    egui_plot::GridMark {
                        value: self.settings.max_quality as f64,
                        step_size: self.settings.max_quality as f64,
                    },
                ];

                egui_plot::Plot::new("quality_probability_distribution")
                    .x_grid_spacer(|_| grid_marks.clone())
                    .height(240.0)
                    .clamp_grid(true)
                    .allow_drag(false)
                    .allow_scroll(false)
                    .allow_boxed_zoom(false)
                    .allow_zoom(false)
                    .show(ui, |plot_ui| {
                        plot_ui.line(line_chart);
                    });
            });
        })
        .response
    }
}
