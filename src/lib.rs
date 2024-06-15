pub mod analizer;
pub mod input_stream;
pub mod output_stream;
pub mod visualizer;

pub const MARGIN: usize = 8;
pub const WIDTH: usize = 512 + 2 * MARGIN;
pub const HEIGHT: usize = WIDTH;
pub const DELTA: f32 = 2.0;
pub const I_BUFF: usize = 2014;
pub const FF_BUFF: usize = 128;
pub const SCALE_FACTOR: usize = 2;
