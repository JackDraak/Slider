/// Represents the visual content of a tile
#[derive(Debug, Clone, PartialEq)]
pub enum TileContent {
    /// Numeric label for the tile
    Numeric(u32),
    /// Placeholder for future image support
    Image(ImageData),
}

/// Placeholder for future image data
#[derive(Debug, Clone, PartialEq)]
pub struct ImageData {
    // Future: image path, bitmap data, etc.
}

/// Represents a single tile in the puzzle
#[derive(Debug, Clone, PartialEq)]
pub struct Tile {
    /// The visual content of the tile
    pub content: TileContent,
    /// The tile's "home" position in the solved state (row, col)
    pub home_position: (usize, usize),
}

impl Tile {
    /// Creates a new tile with numeric content
    pub fn new_numeric(value: u32, home_position: (usize, usize)) -> Self {
        Self {
            content: TileContent::Numeric(value),
            home_position,
        }
    }

    /// Returns the numeric value if this is a numeric tile
    pub fn numeric_value(&self) -> Option<u32> {
        match self.content {
            TileContent::Numeric(n) => Some(n),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_numeric_tile() {
        let tile = Tile::new_numeric(5, (1, 2));
        assert_eq!(tile.numeric_value(), Some(5));
        assert_eq!(tile.home_position, (1, 2));
    }

    #[test]
    fn test_tile_content_equality() {
        let tile1 = Tile::new_numeric(3, (0, 0));
        let tile2 = Tile::new_numeric(3, (0, 0));
        assert_eq!(tile1, tile2);
    }
}