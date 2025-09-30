//! Presenter layer handling GUI rendering and user input.
//!
//! This module contains the egui-based graphical user interface implementation,
//! including tile rendering and visual feedback for player interactions.

pub mod gui_presenter;
pub mod tile_renderer;

pub use gui_presenter::{run_gui, GuiPresenter};
pub use tile_renderer::TileRenderer;