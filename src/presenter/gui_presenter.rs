use crate::controller::GameController;
use crate::model::{Difficulty, PerformanceMetrics, Position};
use crate::presenter::tile_renderer::TileRenderer;
use eframe::egui;
use egui::{CentralPanel, Context, Pos2, TopBottomPanel};
use std::time::Instant;

/// Animation state for a sliding tile
#[derive(Debug, Clone)]
struct TileAnimation {
    tile_pos: Position,
    from_pos: Position,
    to_pos: Position,
    start_time: Instant,
    duration_ms: u64,
}

impl TileAnimation {
    fn new(tile_pos: Position, from_pos: Position, to_pos: Position, duration_ms: u64) -> Self {
        Self {
            tile_pos,
            from_pos,
            to_pos,
            start_time: Instant::now(),
            duration_ms,
        }
    }

    fn progress(&self) -> f32 {
        let elapsed = self.start_time.elapsed().as_millis() as f32;
        let duration = self.duration_ms as f32;
        (elapsed / duration).min(1.0)
    }

    fn is_complete(&self) -> bool {
        self.progress() >= 1.0
    }

    fn current_pos(&self) -> (f32, f32) {
        let t = self.progress();
        // Ease-out cubic for smooth deceleration
        let t = 1.0 - (1.0 - t).powi(3);

        let (from_row, from_col) = self.from_pos;
        let (to_row, to_col) = self.to_pos;

        let row = from_row as f32 + (to_row as f32 - from_row as f32) * t;
        let col = from_col as f32 + (to_col as f32 - from_col as f32) * t;

        (row, col)
    }
}

/// Main GUI presenter using egui
pub struct GuiPresenter {
    controller: GameController,
    renderer: TileRenderer,
    difficulty: Difficulty,
    show_performance: bool,
    animation: Option<TileAnimation>,
}

impl GuiPresenter {
    pub fn new(grid_size: usize) -> Result<Self, crate::model::PuzzleError> {
        let tile_size = 80.0;
        let gap = 5.0;

        Ok(Self {
            controller: GameController::new(grid_size)?,
            renderer: TileRenderer::new(tile_size, gap),
            difficulty: Difficulty::Medium,
            show_performance: false,
            animation: None,
        })
    }
}

impl eframe::App for GuiPresenter {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Check if animation is complete
        if let Some(ref anim) = self.animation {
            if anim.is_complete() {
                self.animation = None;
            } else {
                ctx.request_repaint(); // Continue animating
            }
        }

        // Update auto-solve state (execute moves at 0.7 second intervals)
        self.controller.update_auto_solve();

        // Request repaint for smooth animation
        if self.controller.is_auto_solving() || self.animation.is_some() {
            ctx.request_repaint();
        }

        // Top panel with controls
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Slider Puzzle");

                ui.separator();

                // Difficulty selection
                ui.label("Difficulty:");
                ui.radio_value(&mut self.difficulty, Difficulty::Easy, "Easy");
                ui.radio_value(&mut self.difficulty, Difficulty::Medium, "Medium");
                ui.radio_value(&mut self.difficulty, Difficulty::Hard, "Hard");

                ui.separator();

                // New game button
                if ui.button("New Game").clicked() {
                    self.controller.new_game(self.difficulty);
                }

                // Reset button
                if ui.button("Reset").clicked() {
                    self.controller.reset();
                }

                ui.separator();

                // Auto-solve button
                if self.controller.is_auto_solving() {
                    if ui.button("Stop Solve").clicked() {
                        self.controller.stop_auto_solve();
                    }
                    if let Some((current, total)) = self.controller.auto_solve_progress() {
                        ui.label(format!("{}/{}", current, total));
                    }
                } else if ui.button("Auto Solve").clicked() {
                    self.controller.start_auto_solve();
                }

                ui.separator();

                // Move counter
                ui.label(format!("Moves: {}", self.controller.move_count()));

                ui.separator();

                // Entropy metrics display
                let metrics = self.controller.all_entropy_metrics();

                if self.show_performance {
                    // Detailed view with performance metrics (cached - shows calc time, not frame time)
                    ui.label(format!("Manhattan: {}", metrics.manhattan_distance));
                    ui.label(format!("Heuristic: {}", metrics.shortest_path_heuristic));

                    if metrics.actual_solution_length < 999 {
                        ui.label(format!(
                            "Actual: {} (calc: {})",
                            metrics.actual_solution_length,
                            PerformanceMetrics::format_duration(metrics.performance.actual_time_micros)
                        ));
                    } else {
                        ui.label("Actual: --");
                    }
                } else {
                    // Compact view without timing
                    ui.label(format!("Manhattan: {}", metrics.manhattan_distance));
                    ui.label(format!("Heuristic: {}", metrics.shortest_path_heuristic));

                    if metrics.actual_solution_length < 999 {
                        ui.label(format!("Actual: {}", metrics.actual_solution_length));
                    } else {
                        ui.label("Actual: --");
                    }
                }

                ui.separator();

                // Performance toggle
                ui.checkbox(&mut self.show_performance, "Show Performance");

                // Solved indicator
                if self.controller.is_solved() {
                    ui.separator();
                    ui.colored_label(egui::Color32::GREEN, "SOLVED!");
                }
            });
        });

        // Central panel with puzzle grid
        CentralPanel::default().show(ctx, |ui| {
            let grid_size = self.renderer.grid_size(self.controller.state().size());

            // Center the grid
            let available = ui.available_size();
            let top_left = Pos2::new(
                (available.x - grid_size.x) / 2.0 + ui.min_rect().left(),
                (available.y - grid_size.y) / 2.0 + ui.min_rect().top(),
            );

            // Collect clicked position before modifying state
            let mut clicked_pos = None;

            // Only allow clicks if no animation is running
            let can_interact = self.animation.is_none();

            // Render all tiles (with animation if active)
            for (pos, tile) in self.controller.state().tiles() {
                let render_pos = if let Some(ref anim) = self.animation {
                    // Check if this is the animating tile
                    if pos == anim.tile_pos {
                        let (row, col) = anim.current_pos();
                        (row, col)
                    } else {
                        (pos.0 as f32, pos.1 as f32)
                    }
                } else {
                    (pos.0 as f32, pos.1 as f32)
                };

                let clicked = self.renderer.render_tile_at(ui, tile, pos, render_pos, top_left);
                if clicked && can_interact {
                    clicked_pos = Some(pos);
                }
            }

            // Render empty cell
            let empty_pos = self.controller.state().empty_position();
            self.renderer.render_empty(ui, empty_pos, top_left);

            // Handle click after rendering (start animation)
            if let Some(pos) = clicked_pos {
                let old_empty = self.controller.state().empty_position();
                if self.controller.handle_click(pos) {
                    // Move was successful - start animation
                    let new_empty = self.controller.state().empty_position();
                    // The tile that moved is now at the old empty position
                    self.animation = Some(TileAnimation::new(
                        old_empty,     // tile current position (moved to old empty)
                        new_empty,     // from position (where it was)
                        old_empty,     // to position (where it is now)
                        200,           // 0.2 second animation
                    ));
                }
            }
        });
    }
}

pub fn run_gui(grid_size: usize) -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("Slider Puzzle"),
        ..Default::default()
    };

    eframe::run_native(
        "Slider Puzzle",
        options,
        Box::new(|_cc| {
            GuiPresenter::new(grid_size)
                .map(|p| Box::new(p) as Box<dyn eframe::App>)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        }),
    )
}