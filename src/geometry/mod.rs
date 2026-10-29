pub mod alphabet;
pub mod matrix;

pub use alphabet::{
    angle_to_letter, index_to_angle, letter_to_angle, ALPHABET_SIZE, DEGREES_PER_LETTER,
};
pub use matrix::{
    calculate_grid_dimension, decode_matrix, render_matrix, MatrixConfig, CELL_SIZE,
    QUIET_ZONE_CELLS,
};
