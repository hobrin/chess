mod render;
mod board_slow;
mod board_fast;
mod util;
mod cpu;
mod profiler;
mod test;

// 1 king,
// 2 queen,
// 3 bishop,
// 4 knight,
// 5 rook,
// 6 pawn

pub type Board = board_fast::Board;
pub const DEPTH: usize = 4;
pub const MAX_DEPTH: usize = 5;
pub const NORM_EXPLR_DEPTH: usize = 2;
pub const NORM: i32 = 340;
pub const RANDOM: bool = true;

pub const NOTHING: usize = 0;
pub const KING: usize = 1;
pub const QUEEN: usize = 2;
pub const BISHOP: usize = 3;
pub const KNIGHT: usize = 4;
pub const ROOK: usize = 5;
pub const PAWN: usize = 6;

fn main() {
    render::main();
}