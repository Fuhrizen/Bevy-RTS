//! Engine-independent rules and coordinates for fast Rust unit tests.

pub mod grid {
    pub const COLUMNS: u32 = 18;
    pub const ROWS: u32 = 12;
    pub const CELL_SIZE: f32 = 50.0;

    /// Convert a grid coordinate into a world-space 2D position.
    pub fn tile_position(column: u32, row: u32) -> [f32; 2] {
        [
            (column as f32 - (COLUMNS - 1) as f32 / 2.0) * CELL_SIZE,
            (row as f32 - (ROWS - 1) as f32 / 2.0) * CELL_SIZE,
        ]
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn grid_is_centred_around_origin() {
            assert_eq!(tile_position(0, 0), [-425.0, -275.0]);
            assert_eq!(tile_position(COLUMNS - 1, ROWS - 1), [425.0, 275.0]);
        }

        #[test]
        fn neighbouring_tiles_are_spaced_evenly() {
            let [left_x, left_y] = tile_position(3, 5);
            let [right_x, right_y] = tile_position(4, 5);
            assert_eq!((right_x - left_x, right_y - left_y), (CELL_SIZE, 0.0));
        }
    }
}
