use slider::run_gui;

fn main() -> eframe::Result<()> {
    // Default to 4x4 grid
    let grid_size = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(4);

    if grid_size < 3 || grid_size > 15 {
        eprintln!("Grid size must be between 3 and 15");
        std::process::exit(1);
    }

    run_gui(grid_size)
}
