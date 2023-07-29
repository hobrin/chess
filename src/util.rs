pub const window_width: f32 = 400.0;
use ggez::conf;
use ggez::graphics::StrokeOptions;
use ggez::mint::{self, Vector2};
use ggez::event::MouseButton;
use ggez::input::mouse::button_pressed;
use ggez::filesystem;
use ggez::GameError;

use crate::util;
use crate::cpu;
extern crate lazy_static;
use lazy_static::lazy_static;

extern crate bitintr;
use bitintr::*;

use std::arch::asm;
use std::borrow::BorrowMut;
use std::sync::RwLock;
use std::sync::Once;
use std::sync::Arc;

// https://github.com/ggez/ggez/tree/master/examples
use ggez::{
    event,
    glam::*,
    graphics::{self, Color},
    Context, GameResult,
};
use std::ops::{Add, Mul};
use std::{env, path};

pub const DIRECTIONS: [Vec2; 4] = [Vec2{x: 1.0, y: 0.0}, Vec2{x: -1.0, y: 0.0}, Vec2{x: 0.0, y: 1.0}, Vec2{x: 0.0, y: -1.0}];
pub const DIAGONALS: [Vec2; 4] = [Vec2{x: 1.0, y: 1.0}, Vec2{x: -1.0, y: -1.0}, Vec2{x: -1.0, y: 1.0}, Vec2{x: 1.0, y: -1.0}];
pub const PIECE_VALUES: [i32; 13] = [
    0, //nothing
    100_000, //white king
    900, //white queen
    320, //white bishop
    300, //white knight
    500, //white rook
    100, //white pawn
    -100_000, //black king
    -900, //black queen
    -320, //black bishop
    -300, //black knight
    -500, //black rook
    -100, //black pawn
];

pub const PIECE_VALUES_POSITION: [[i32; 64]; 13] = [[ //no nothing
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
], [ //white king
    100_000+-30, 100_000+-40, 100_000+-40, 100_000+-50, 100_000+-50, 100_000+-40, 100_000+-40, 100_000+-30,
    100_000+-30, 100_000+-40, 100_000+-40, 100_000+-50, 100_000+-50, 100_000+-40, 100_000+-40, 100_000+-30,
    100_000+-30, 100_000+-40, 100_000+-40, 100_000+-50, 100_000+-50, 100_000+-40, 100_000+-40, 100_000+-30,
    100_000+-30, 100_000+-40, 100_000+-40, 100_000+-50, 100_000+-50, 100_000+-40, 100_000+-40, 100_000+-30,
    100_000+-20, 100_000+-30, 100_000+-30, 100_000+-40, 100_000+-40, 100_000+-30, 100_000+-30, 100_000+-20,
    100_000+-10, 100_000+-20, 100_000+-20, 100_000+-20, 100_000+-20, 100_000+-20, 100_000+-20, 100_000+-10,
    100_000+20,  100_000+20,  100_000+ 0,  100_000+ 0,  100_000+ 0,  100_000+ 0,  100_000+20,  100_000+20,
    100_000+20,  100_000+30,  100_000+10,  100_000+ 0,  100_000+ 0,  100_000+10,  100_000+30,  100_000+20,
], [ //white queen
    900+-20, 900+-10, 900+-10,  900+-5,  900+-5, 900+-10, 900+-10, 900+-20,
    900+-10, 900+  0, 900+  0,  900+ 0,  900+ 0, 900+  0, 900+  0, 900+-10,
    900+-10, 900+  0, 900+  5,  900+ 5,  900+ 5, 900+  5, 900+  0, 900+-10,
    900+ -5, 900+  0, 900+  5,  900+ 5,  900+ 5, 900+  5, 900+  0, 900+ -5,
    900+  0, 900+  0, 900+  5,  900+ 5,  900+ 5, 900+  5, 900+  0, 900+ -5,
    900+-10, 900+  5, 900+  5,  900+ 5,  900+ 5, 900+  5, 900+  0, 900+-10,
    900+-10, 900+  0, 900+  5,  900+ 0,  900+ 0, 900+  0, 900+  0, 900+-10,
    900+-20, 900+-10, 900+-10,  900+50,  900+-5, 900+-10, 900+-10, 900+-20,
], [ //white bishop
    320+-20, 320+-10, 320+-10, 320+-10, 320+-10, 320+-10, 320+-10, 320+-20,
    320+-10, 320+  0, 320+  0, 320+  0, 320+  0, 320+  0, 320+  0, 320+-10,
    320+-10, 320+  0, 320+  5, 320+ 10, 320+ 10, 320+  5, 320+  0, 320+-10,
    320+-10, 320+  5, 320+  5, 320+ 10, 320+ 10, 320+  5, 320+  5, 320+-10,
    320+-10, 320+  0, 320+ 10, 320+ 10, 320+ 10, 320+ 10, 320+  0, 320+-10,
    320+-10, 320+ 10, 320+ 10, 320+ 10, 320+ 10, 320+ 10, 320+ 10, 320+-10,
    320+-10, 320+  5, 320+  0, 320+  0, 320+  0, 320+  0, 320+  5, 320+-10,
    320+-20, 320+-10, 320+-10, 320+-10, 320+-10, 320+-10, 320+-10, 320+-20,
],
[ //white knight
    300+-50, 300+-40, 300+-30, 300+-30, 300+-30, 300+-30, 300+-40, 300+-50,
    300+-40, 300+-20, 300+  0, 300+  0, 300+  0, 300+  0, 300+-20, 300+-40,
    300+-30, 300+  0, 300+ 10, 300+ 15, 300+ 15, 300+ 10, 300+  0, 300+-30,
    300+-30, 300+  5, 300+ 15, 300+ 20, 300+ 20, 300+ 15, 300+  5, 300+-30,
    300+-30, 300+  0, 300+ 15, 300+ 20, 300+ 20, 300+ 15, 300+  0, 300+-30,
    300+-30, 300+  5, 300+ 10, 300+ 15, 300+ 15, 300+ 10, 300+  5, 300+-30,
    300+-40, 300+-20, 300+  0, 300+  5, 300+  5, 300+  0, 300+-20, 300+-40,
    300+-50, 300+-40, 300+-30, 300+-30, 300+-30, 300+-30, 300+-40, 300+-50,
],[ //white rook
   500+ 0, 500+ 0, 500+ 0, 500+ 0, 500+ 0, 500+ 0, 500+ 0, 500+ 0,
   500+ 5, 500+10, 500+10, 500+10, 500+10, 500+10, 500+10, 500+ 5,
   500+-5, 500+ 0, 500+ 0, 500+ 0, 500+ 0, 500+ 0, 500+ 0, 500+-5,
   500+-5, 500+ 0, 500+ 0, 500+ 0, 500+ 0, 500+ 0, 500+ 0, 500+-5,
   500+-5, 500+ 0, 500+ 0, 500+ 0, 500+ 0, 500+ 0, 500+ 0, 500+-5,
   500+-5, 500+ 0, 500+ 0, 500+ 0, 500+ 0, 500+ 0, 500+ 0, 500+-5,
   500+-5, 500+ 0, 500+ 0, 500+ 0, 500+ 0, 500+ 0, 500+ 0, 500+-5,
   500+ 0, 500+ 0, 500+ 0, 500+ 5, 500+ 5, 500+ 0, 500+ 0, 500+ 0,
],
[ //white pawn
   100+ 0, 100+ 0,100+  0,100+  0,100+  0,100+  0,100+  0,100+  0,
   100+50, 100+50,100+ 50,100+ 50,100+ 50,100+ 50,100+ 50,100+ 50,
   100+10, 100+10,100+ 20,100+ 30,100+ 30,100+ 20,100+ 10,100+ 10,
   100+ 5, 100+ 5,100+ 10,100+ 25,100+ 25,100+ 10,100+  5,100+  5,
   100+ 0, 100+ 0,100+  0,100+ 20,100+ 20,100+  0,100+  0,100+  0,
   100+ 5, 100+-5,100+-10,100+  0,100+  0,100+-10,100+ -5,100+  5,
   100+ 5, 100+10,100+ 10,100+-20,100+-20,100+ 10,100+ 10,100+  5,
   100+ 0, 100+ 0,100+  0,100+  0,100+  0,100+  0,100+  0,100+  0,
],
[ //black king
    -100_000-20,  -100_000-30,  -100_000-10,  -100_000- 0,  -100_000- 0,  -100_000-10,  -100_000-30,  -100_000-20,
    -100_000-20,  -100_000-20,  -100_000- 0,  -100_000- 0,  -100_000- 0,  -100_000- 0,  -100_000-20,  -100_000-20,
    -100_000--10, -100_000--20, -100_000--20, -100_000--20, -100_000--20, -100_000--20, -100_000--20, -100_000--10,
    -100_000--20, -100_000--30, -100_000--30, -100_000--40, -100_000--40, -100_000--30, -100_000--30, -100_000--20,
    -100_000--20, -100_000--40, -100_000--40, -100_000--50, -100_000--50, -100_000--40, -100_000--40, -100_000--20,
    -100_000--20, -100_000--40, -100_000--40, -100_000--50, -100_000--50, -100_000--40, -100_000--40, -100_000--20,
    -100_000--30, -100_000--40, -100_000--40, -100_000--50, -100_000--50, -100_000--40, -100_000--40, -100_000--30,
    -100_000--30, -100_000--40, -100_000--40, -100_000--50, -100_000--50, -100_000--40, -100_000--40, -100_000--30,
], [ //black queen
    -900--20, -900--10, -900--10,  -900-40,  -900--5, -900--10, -900--10, -900--20,
    -900--10, -900-  0, -900-  0,  -900- 0,  -900- 0, -900-  0, -900-  0, -900--10,
    -900--10, -900-  0, -900-  5,  -900- 5,  -900- 5, -900-  5, -900-  0, -900--10,
    -900- -5, -900-  0, -900-  5,  -900- 5,  -900- 5, -900-  5, -900-  0, -900- -5,
    -900-  0, -900-  0, -900-  5,  -900- 5,  -900- 5, -900-  5, -900-  0, -900- -5,
    -900--10, -900-  5, -900-  5,  -900- 5,  -900- 5, -900-  5, -900-  0, -900--10,
    -900--10, -900-  0, -900-  5,  -900- 0,  -900- 0, -900-  0, -900-  0, -900--10,
    -900--20, -900--10, -900--10,  -900--5,  -900--5, -900--10, -900--10, -900--20,
], [ //black knight
    -300--50, -300--40, -300--30, -300--30, -300--30, -300--30, -300--40, -300--50,
    -300--40, -300--20, -300-  0, -300-  5, -300-  5, -300-  0, -300--20, -300--40,
    -300--30, -300-  5, -300- 10, -300- 15, -300- 15, -300- 10, -300-  5, -300--30,
    -300--30, -300-  0, -300- 15, -300- 20, -300- 20, -300- 15, -300-  0, -300--30,
    -300--30, -300-  5, -300- 15, -300- 20, -300- 20, -300- 15, -300-  5, -300--30,
    -300--30, -300-  0, -300- 10, -300- 15, -300- 15, -300- 10, -300-  0, -300--30,
    -300--40, -300--20, -300-  0, -300-  0, -300-  0, -300-  0, -300--20, -300--40,
    -300--50, -300--40, -300--30, -300--30, -300--30, -300--30, -300--40, -300--50,
], [ //black bishop
    -320--20, -320--10, -320--10, -320--10, -320--10, -320--10, -320--10, -320--20,
    -320--10, -320-  5, -320-  0, -320-  0, -320-  0, -320-  0, -320-  5, -320--10,
    -320--10, -320- 10, -320- 10, -320- 10, -320- 10, -320- 10, -320- 10, -320--10,
    -320--10, -320-  0, -320- 10, -320- 10, -320- 10, -320- 10, -320-  0, -320--10,
    -320--10, -320-  5, -320-  5, -320- 10, -320- 10, -320-  5, -320-  5, -320--10,
    -320--10, -320-  0, -320-  5, -320- 10, -320- 10, -320-  5, -320-  0, -320--10,
    -320--10, -320-  0, -320-  0, -320-  0, -320-  0, -320-  0, -320-  0, -320--10,
    -320--20, -320--10, -320--10, -320--10, -320--10, -320--10, -320--10, -320--20,
], [ //black rook
    -500- 0, -500- 0, -500- 0, -500- 5, -500- 5, -500- 0, -500- 0, -500- 0,
    -500--5, -500- 0, -500- 0, -500- 0, -500- 0, -500- 0, -500- 0, -500--5,
    -500--5, -500- 0, -500- 0, -500- 0, -500- 0, -500- 0, -500- 0, -500--5,
    -500--5, -500- 0, -500- 0, -500- 0, -500- 0, -500- 0, -500- 0, -500--5,
    -500--5, -500- 0, -500- 0, -500- 0, -500- 0, -500- 0, -500- 0, -500--5,
    -500--5, -500- 0, -500- 0, -500- 0, -500- 0, -500- 0, -500- 0, -500--5,
    -500- 5, -500-10, -500-10, -500-10, -500-10, -500-10, -500-10, -500- 5,
    -500- 0, -500- 0, -500- 0, -500- 0, -500- 0, -500- 0, -500- 0, -500- 0,
], [ //black pawn
   -100- 0, -100- 0,-100-  0,-100-  0,-100-  0,-100-  0,-100-  0,-100-  0,
   -100- 5, -100-10,-100- 10,-100--20,-100--20,-100- 10,-100- 10,-100-  5,
   -100- 5, -100--5,-100--10,-100-  0,-100-  0,-100--10,-100- -5,-100-  5,
   -100- 0, -100- 0,-100-  0,-100- 20,-100- 20,-100-  0,-100-  0,-100-  0,
   -100- 5, -100- 5,-100- 10,-100- 25,-100- 25,-100- 10,-100-  5,-100-  5,
   -100-10, -100-10,-100- 20,-100- 30,-100- 30,-100- 20,-100- 10,-100- 10,
   -100-50, -100-50,-100- 50,-100- 50,-100- 50,-100- 50,-100- 50,-100- 50,
   -100- 0, -100- 0,-100-  0,-100-  0,-100-  0,-100-  0,-100-  0,-100-  0,
],];


lazy_static! {
    pub static ref KNIGHT_MOVES: [u64; 64] = {
        let mut moves = [0u64; 64];
        for pos in 0..64 { let (x, y) = util::pos_to_xy(pos);
            let (x, y) = (x as isize, y as isize);
            let thin_template: u64 = util::fix_shl(0b00001010, x-2);
            let wide_template: u64 = util::fix_shl(0b00010001, x-2);
            let row_template: u64 = 0b11111111;
            let moveable: u64 = 
                (util::fix_shl(thin_template, (y-2)*8) & fix_shl(row_template, (y-2)*8)) |
                (util::fix_shl(wide_template, (y-1)*8) & fix_shl(row_template, (y-1)*8)) |
                (util::fix_shl(wide_template, (y+1)*8) & fix_shl(row_template, (y+1)*8)) |
                (util::fix_shl(thin_template, (y+2)*8) & fix_shl(row_template, (y+2)*8));
            moves[pos] = moveable;
        }
        moves
    };
    pub static ref KING_MOVES: [u64; 64] = {
        let mut moves = [0u64; 64];
        for pos in 0..64 {
            let template: u64 = 0b00000111;
            let row_template: u64 = 0b11111111;

            let (x, y) = pos_to_xy(pos);
            let (x, y) = (x as isize, y as isize);
            let y_at = (y*8);
            let y_before= ((y-1)*8);
            let y_after = ((y+1)*8);

            let in_row = fix_shl(template, x-1) & row_template;
            let value = (fix_shl(in_row, y_at)) |
                (fix_shl(in_row, y_before)) |
                (fix_shl(in_row, y_after));
            moves[pos] = value;
        }
        moves
    };
    pub static ref ROOK_MOVES: [u64; 64] = {
        let mut moves = [0u64; 64];
        for pos in 0..64 {
            let (x, y) = pos_to_xy(pos);
            let mut cur_move: u64 = 0;
            for dir_dir in DIRECTIONS.iter() {
                let mut dir: Vec2 = Vec2 {x: 6.9, y: 6.9}; //Never actually gets run, shitcode
                let mut i = 1;
                loop {
                    dir = pos_to_vec(pos).add(dir_dir.mul(i as f32));
                    if dir.x < 0.0 || dir.y < 0.0 || dir.x > 7.0 || dir.y > 7.0 {
                        break;
                    }
                    cur_move |= 1 << (dir.x as usize + (dir.y as usize*8));
                    i+=1;
                }
            }
            moves[pos] = cur_move;
        }
        moves
    };
    pub static ref ROOK_OBSTRUCTION_SELF_MAP: Vec<Vec<u64>> = {
        let mut obstruct_map: Vec<Vec<u64>> = vec![];
        for pos in 0..64 {
            obstruct_map.push(vec![]);
            let (x, y) = util::pos_to_xy(pos);
            for obstruction_idx  in 0..(1u64<<14) {
                let mut cur_move: u64 = 0;
                let bounds_rev: Vec<_> = vec![
                    (0, y),
                    (y, (y+x)),
                ];
                let bounds = vec![
                    ((y+x), (y+7)),
                    ((y+7), 14),
                ];
                for i in 0..2 {
                    for cur_bit in (bounds_rev[i].0..bounds_rev[i].1).rev() {
                        if obstruction_idx & (1<<cur_bit) != 0 {
                            break; }
                        cur_move |= (1<<(cur_bit)).pdep(util::ROOK_MOVES[pos]);
                    }
                }
                for i in 0..2 {
                    for cur_bit in bounds[i].0..bounds[i].1 {
                        if obstruction_idx & (1<<cur_bit) != 0 {
                            break;
                        }
                        cur_move |= (1<<(cur_bit)).pdep(util::ROOK_MOVES[pos]);
                    }
                }
                obstruct_map[pos].push(cur_move);
                // moves[pos][obstruction_idx as usize] = cur_move;
            }
        }
        println!("initialised rook self obstruction!");
        obstruct_map
    };
    pub static ref ROOK_OBSTRUCTION_OPPONENT_MAP: Vec<Vec<u64>> = {
        let mut obstruct_map: Vec<Vec<u64>> = vec![];
        for pos in 0..64 {
            obstruct_map.push(vec![]);
            let (x, y) = util::pos_to_xy(pos);
            for obstruction_idx  in 0..(1u64<<14) {
                let mut cur_move: u64 = 0;
                let bounds_rev: Vec<_> = vec![
                    (0, y),
                    (y, (y+x)),
                ];
                let bounds = vec![
                    ((y+x), (y+7)),
                    ((y+7), 14),
                ];
                for i in 0..2 {
                    for cur_bit in (bounds_rev[i].0..bounds_rev[i].1).rev() {
                        cur_move |= (1<<(cur_bit)).pdep(util::ROOK_MOVES[pos]);
                        if obstruction_idx & (1<<cur_bit) != 0 {
                            break;
                        }
                    }
                }
                for i in 0..2 {
                    for cur_bit in bounds[i].0..bounds[i].1 {
                        cur_move |= (1<<(cur_bit)).pdep(util::ROOK_MOVES[pos]);
                        if obstruction_idx & (1<<cur_bit) != 0 {
                            break;
                        }
                    }
                }
                obstruct_map[pos].push(cur_move);
                // moves[pos][obstruction_idx as usize] = cur_move;
            }
        }
        println!("initialised rook obstruction!");
        obstruct_map
    };
    pub static ref BISHOP_MOVES: [u64; 64] = {
        let mut moves = [0u64; 64];
        for pos in 0..64 {
            let (x, y) = pos_to_xy(pos);
            let mut cur_move: u64 = 0;
            for dir_dir in DIAGONALS.iter() {
                let mut dir: Vec2 = Vec2 {x: 6.9, y: 6.9}; //Never actually gets run, shitcode
                let mut i = 1;
                loop {
                    dir = pos_to_vec(pos).add(dir_dir.mul(i as f32));
                    if dir.x < 0.0 || dir.y < 0.0 || dir.x > 7.0 || dir.y > 7.0 {
                        break;
                    }
                    cur_move |= 1 << (dir.x as usize + (dir.y as usize*8));
                    i+=1;
                }
            }
            moves[pos] = cur_move;
        }
        moves
    };
    pub static ref BISHOP_OBSTRUCTION_SELF_MAP: Vec<Vec<u64>> = {
        let mut obstruct_map: Vec<Vec<u64>> = vec![];
        for pos in 0..64 {
            obstruct_map.push(vec![]);
            let (x, y) = util::pos_to_xy(pos);
            let count = BISHOP_MOVES[pos].count_ones();
            for obstruction_idx  in 0..(1u64<<count) {
                let obstuction_reconstructed = obstruction_idx.pdep(BISHOP_MOVES[pos]);
                let mut cur_move: u64 = 0;
                for dir_dir in DIAGONALS.iter() {
                    let mut look_pos: Vec2 = Vec2 {x: 6.9, y: 6.9}; //Never actually gets run, shitcode
                    let mut i = 1;
                    loop {
                        look_pos = pos_to_vec(pos).add(dir_dir.mul(i as f32));
                        if look_pos.x < 0.0 || look_pos.y < 0.0 || look_pos.x > 7.0 || look_pos.y > 7.0 {
                            break;
                        }
                        if 0 != (obstuction_reconstructed & (1<<(look_pos.x as usize + (look_pos.y as usize*8)))) {
                            break;
                        }
                        cur_move |= 1 << (look_pos.x as usize + (look_pos.y as usize*8));
                        i+=1;
                    }
                }
                obstruct_map[pos].push(cur_move);
            }
        }
        println!("initialised bishop self obstruction!");
        obstruct_map
    };
    pub static ref BISHOP_OBSTRUCTION_OPPONENT_MAP: Vec<Vec<u64>> = {
        let mut obstruct_map: Vec<Vec<u64>> = vec![];
        for pos in 0..64 {
            obstruct_map.push(vec![]);
            let (x, y) = util::pos_to_xy(pos);
            let count = BISHOP_MOVES[pos].count_ones();
            for obstruction_idx  in 0..(1u64<<count) {
                let obstuction_reconstructed = obstruction_idx.pdep(BISHOP_MOVES[pos]);
                let mut cur_move: u64 = 0;
                for dir_dir in DIAGONALS.iter() {
                    let mut look_pos: Vec2 = Vec2 {x: 6.9, y: 6.9}; //Never actually gets run, shitcode
                    let mut i = 1;
                    loop {
                        look_pos = pos_to_vec(pos).add(dir_dir.mul(i as f32));
                        if look_pos.x < 0.0 || look_pos.y < 0.0 || look_pos.x > 7.0 || look_pos.y > 7.0 {
                            break;
                        }
                        cur_move |= 1 << (look_pos.x as usize + (look_pos.y as usize*8));
                        if 0 != (obstuction_reconstructed & (1<<(look_pos.x as usize + (look_pos.y as usize*8)))) {
                            break;
                        }
                        i+=1;
                    }
                }
                obstruct_map[pos].push(cur_move);
            }
        }
        println!("initialised bishop obstruction!");
        obstruct_map
    };
    pub static ref WHITE_PAWN_CAPTURES: [u64; 64] = {
        let mut moves = [0u64; 64];
        let forward_dir: Vec2 = Vec2 {x: 0.0, y: -1.0};
        let sides = vec![Vec2{x: 1.0, y: 0.0}, Vec2 {x: -1.0, y: 0.0}];
        for pos in 0..64 {
            let mut cur_move: u64 = 0;
            let pos_vec = pos_to_vec(pos);
            let forward = pos_vec.add(forward_dir);
            if forward.y < 0.0 || forward.y > 7.0 {
                continue;
            }
            for side in sides.iter() {
                let capture_spot = forward.add(*side);
                if capture_spot.x < 0.0 || capture_spot.x > 7.0 {
                    continue;
                }
                cur_move |= 1u64 << (capture_spot.x as usize + (capture_spot.y as usize *8));
            }
            moves[pos] = cur_move;
        }
        moves
    };
    pub static ref BLACK_PAWN_CAPTURES: [u64; 64] = {
        let mut moves = [0u64; 64];
        let forward_dir: Vec2 = Vec2 {x: 0.0, y: 1.0};
        let sides = vec![Vec2{x: 1.0, y: 0.0}, Vec2 {x: -1.0, y: 0.0}];
        for pos in 0..64 {
            let mut cur_move: u64 = 0;
            let pos_vec = pos_to_vec(pos);
            let forward = pos_vec.add(forward_dir);
            if forward.y < 0.0 || forward.y > 7.0 {
                continue;
            }
            for side in sides.iter() {
                let capture_spot = forward.add(*side);
                if capture_spot.x < 0.0 || capture_spot.x > 7.0 {
                    continue;
                }
                cur_move |= 1u64 << (capture_spot.x as usize + (capture_spot.y as usize *8));
            }
            moves[pos] = cur_move;
        }
        moves
    };
}


const POS_TO_XY: [(usize, usize); 64] = [
    (0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0), (7, 0), 
    (0, 1), (1, 1), (2, 1), (3, 1), (4, 1), (5, 1), (6, 1), (7, 1), 
    (0, 2), (1, 2), (2, 2), (3, 2), (4, 2), (5, 2), (6, 2), (7, 2), 
    (0, 3), (1, 3), (2, 3), (3, 3), (4, 3), (5, 3), (6, 3), (7, 3), 
    (0, 4), (1, 4), (2, 4), (3, 4), (4, 4), (5, 4), (6, 4), (7, 4), 
    (0, 5), (1, 5), (2, 5), (3, 5), (4, 5), (5, 5), (6, 5), (7, 5), 
    (0, 6), (1, 6), (2, 6), (3, 6), (4, 6), (5, 6), (6, 6), (7, 6), 
    (0, 7), (1, 7), (2, 7), (3, 7), (4, 7), (5, 7), (6, 7), (7, 7), 
];
pub fn pos_to_xy(pos: usize) -> (usize, usize) {
    (pos % 8, pos / 8)
    // POS_TO_XY[pos]
}

pub fn pos_to_vec(pos: usize) -> Vec2 {
    let (x, y) = pos_to_xy(pos);
    Vec2 {
        x: x as f32,
        y: y as f32,
    }
}

pub fn is_enemy(piece: usize, white: bool) -> bool {
    if white {
        piece > 6
    } else {
        piece > 0 && piece < 7
    }
}

pub fn is_piece_white(piece: usize) -> bool {
    piece <= 6
}

pub const PIECE_TO_COLOURLESS: [usize; 13] = [0, 1, 2, 3, 4, 5, 6, 1, 2, 3, 4, 5, 6];

pub fn fix_shl(a: u64, amount: isize) -> u64 {
    ((amount >= 0) as u64 * a.wrapping_shl(amount as u32) & std::u64::MAX*((amount<64) as u64)) |
    ((amount < 0) as u64 * a.wrapping_shr(amount.abs() as u32) & std::u64::MAX*((amount>-64) as u64))
}
pub fn fix_shr(a: u64, amount: u32) -> u64 {
    a.wrapping_shr(amount) & std::u64::MAX*((amount<64) as u64)
}

fn get_bsf(bitboard: u64) -> usize {
    #[cfg(target_arch = "x86_64")] {
        unsafe {
            let nr: usize;
            asm!(
                "bsf {0}, {1}",
                out(reg) nr,
                in(reg) bitboard
            );
            nr
        }
    }
}

pub fn bitboard_to_vec(mut board: u64) -> Vec<usize> {
    let mut res = vec![0usize; board.count_ones() as usize];
    let mut prev_shift = 0;
    let mut i: usize = 0;
    loop {
        if board == 0 {
            break res;
        }
        let idx = get_bsf(board);
        res[i] = idx + prev_shift;
        board = fix_shr(board, idx as u32+1);
        prev_shift += idx+1;
        i+=1;
    }
}
pub struct BitIter {
    board: u64,
    prev_shift: usize,
}
impl BitIter {
    pub fn new(board: u64) -> Self {
        Self {
            board,
            prev_shift: 0,
        }
    }
}
impl Iterator for BitIter {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        if self.board == 0 {
            return None;
        }
        let idx = get_bsf(self.board);
        self.prev_shift += idx+1;
        self.board = fix_shr(self.board, idx as u32+1);
        Some(self.prev_shift-1)
    }
}

pub fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + f32::exp(-x))
}