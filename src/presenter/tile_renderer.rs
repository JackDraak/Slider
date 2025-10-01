use crate::model::{Tile, TileContent};
use egui::{Color32, FontId, Pos2, Rect, Sense, Ui, Vec2};

/// Renders individual tiles in the GUI
pub struct TileRenderer {
    tile_size: f32,
    gap: f32,
}

impl TileRenderer {
    pub fn new(tile_size: f32, gap: f32) -> Self {
        Self { tile_size, gap }
    }

    /// Renders a single tile at the given grid position
    /// Returns true if the tile was clicked
    pub fn render_tile(
        &self,
        ui: &mut Ui,
        tile: &Tile,
        grid_pos: (usize, usize),
        top_left: Pos2,
    ) -> bool {
        self.render_tile_at(ui, tile, grid_pos, (grid_pos.0 as f32, grid_pos.1 as f32), top_left)
    }

    /// Renders a single tile at a specific rendered position (supports fractional positions for animation)
    /// Returns true if the tile was clicked
    pub fn render_tile_at(
        &self,
        ui: &mut Ui,
        tile: &Tile,
        grid_pos: (usize, usize),
        render_pos: (f32, f32),
        top_left: Pos2,
    ) -> bool {
        let (row, col) = render_pos;
        let x = top_left.x + col * (self.tile_size + self.gap);
        let y = top_left.y + row * (self.tile_size + self.gap);

        let rect = Rect::from_min_size(
            Pos2::new(x, y),
            Vec2::new(self.tile_size, self.tile_size),
        );

        let response = ui.allocate_rect(rect, Sense::click());

        // Calculate Manhattan distance from home position
        let manhattan_distance =
            (tile.home_position.0 as i32 - grid_pos.0 as i32).abs() +
            (tile.home_position.1 as i32 - grid_pos.1 as i32).abs();

        // Determine color based on Manhattan distance
        // Distance 0 (home) = light sky blue (135, 206, 235)
        // Distance 6 (max for 4x4) = pale red (255, 160, 160)
        // Interpolate between these colors
        let max_distance = 6.0; // Maximum possible Manhattan distance in 4x4 grid
        let ratio = (manhattan_distance as f32 / max_distance).min(1.0);

        let blue_r = 135;
        let blue_g = 206;
        let blue_b = 235;
        let red_r = 255;
        let red_g = 160;
        let red_b = 160;

        let r = (blue_r as f32 + ratio * (red_r as f32 - blue_r as f32)) as u8;
        let g = (blue_g as f32 + ratio * (red_g as f32 - blue_g as f32)) as u8;
        let b = (blue_b as f32 + ratio * (red_b as f32 - blue_b as f32)) as u8;

        let color = Color32::from_rgb(r, g, b);

        // Highlight on hover
        let color = if response.hovered() {
            Color32::from_rgb(255, 255, 150)
        } else {
            color
        };

        // Draw tile background
        ui.painter().rect_filled(rect, 5.0, color);

        // Draw border
        ui.painter()
            .rect_stroke(rect, 5.0, (2.0, Color32::from_rgb(80, 80, 80)));

        // Draw tile content
        match &tile.content {
            TileContent::Numeric(n) => {
                let text = format!("{}", n);
                let font = FontId::proportional(self.tile_size * 0.4);
                let galley = ui.painter().layout_no_wrap(text, font, Color32::BLACK);

                let text_pos = Pos2::new(
                    rect.center().x - galley.size().x / 2.0,
                    rect.center().y - galley.size().y / 2.0,
                );

                ui.painter().galley(text_pos, galley, Color32::BLACK);
            }
            TileContent::Image(_) => {
                // Placeholder for future image rendering
                let text = "IMG";
                let font = FontId::proportional(self.tile_size * 0.3);
                let galley = ui.painter().layout_no_wrap(text.to_string(), font, Color32::BLACK);

                let text_pos = Pos2::new(
                    rect.center().x - galley.size().x / 2.0,
                    rect.center().y - galley.size().y / 2.0,
                );

                ui.painter().galley(text_pos, galley, Color32::BLACK);
            }
        }

        response.clicked()
    }

    /// Renders the empty cell
    pub fn render_empty(&self, ui: &mut Ui, grid_pos: (usize, usize), top_left: Pos2) {
        let (row, col) = grid_pos;
        let x = top_left.x + col as f32 * (self.tile_size + self.gap);
        let y = top_left.y + row as f32 * (self.tile_size + self.gap);

        let rect = Rect::from_min_size(
            Pos2::new(x, y),
            Vec2::new(self.tile_size, self.tile_size),
        );

        // Draw empty cell with darker background
        ui.painter()
            .rect_filled(rect, 5.0, Color32::from_rgb(50, 50, 50));
    }

    /// Calculates the total size needed for the grid
    pub fn grid_size(&self, puzzle_size: usize) -> Vec2 {
        let total = puzzle_size as f32 * self.tile_size + (puzzle_size - 1) as f32 * self.gap;
        Vec2::new(total, total)
    }
}