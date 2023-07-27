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

// https://github.com/ggez/ggez/tree/master/examples
use ggez::{
    event,
    glam::*,
    graphics::{self, Color},
    Context, GameResult,
};
use std::ops::{Add, Mul};
use std::{env, path};

pub struct Board {
    pub is_whites_turn: bool,
    pub has_moved_king_white: bool,
    pub has_moved_king_black: bool,
    pub en_passant: usize,

    pub board_pos: [usize; 64],
}
impl Clone for Board {
    fn clone(&self) -> Board {
        Board {
            is_whites_turn: self.is_whites_turn,
            has_moved_king_white: self.has_moved_king_white,
            has_moved_king_black: self.has_moved_king_black,
            en_passant: self.en_passant,
            board_pos: self.board_pos,
        }
    }
}

impl Board {
    pub fn new() -> Board {
        let board_pos: [usize; 64] = [
            11, 10,  9,  8,  7,  9, 10, 11,
            12, 12, 12, 12, 12, 12, 12, 12,
            0,  0,  0,  0,  0,  0,  0,  0,
            0,  0,  0,  0,  0,  0,  0,  0,
            0,  0,  0,  0,  0,  0,  0,  0,
            0,  0,  0,  0,  0,  0,  0,  0,
            6,  6,  6,  6,  6,  6,  6,  6,
            5,  4,  3,  2,  1,  3,  4,  5,
        ];
        Board {board_pos, has_moved_king_white: false, has_moved_king_black: false, en_passant: 0, is_whites_turn: true}
    }
    pub fn move_square(self: &mut Board, old: usize, new: usize) -> bool {
        let mut piece = self.board_pos[old];
        self.en_passant = 0;
        match util::PIECE_TO_COLOURLESS[piece] {
            1 => {
                *self.has_king_moved_mut(util::is_piece_white(piece)) = true;
                if util::pos_to_xy(old).0.abs_diff(util::pos_to_xy(new).0) == 2 {
                    let rook_x = if util::pos_to_xy(old).0 < util::pos_to_xy(new).0 {
                        7usize
                    } else {
                        0usize
                    };
                    let rook_y = util::pos_to_xy(old).1;
                    let rook_pos = rook_x + rook_y*8;
                    self.move_square(rook_pos, (old+new)/2);
                }
            }
            6 => {
                if util::pos_to_xy(old).1 % 5 == 1 { //Fancy schmanzy check if on second or seventh rank
                    if util::pos_to_xy(new).1 == 3 || util::pos_to_xy(new).1 == 4 {
                        self.en_passant = new;
                    }
                }
                if util::pos_to_xy(new).1 % 7 == 0 {
                    piece = 2;
                }
            }
            _ => {}
        }
        let is_capture = self.board_pos[new] > 0;
        self.board_pos[new] = piece;
        self.board_pos[old] = 0;
        self.is_whites_turn = !self.is_whites_turn;
        is_capture
    }

    pub fn has_king_moved_mut(self: &mut Board, is_white: bool) -> &mut bool {
        if is_white {
            &mut self.has_moved_king_white
        } else {
            &mut self.has_moved_king_black
        }
    }
    pub fn has_king_moved(self: &Board, is_white: bool) -> &bool {
        if is_white {
            &self.has_moved_king_white
        } else {
            &self.has_moved_king_black
        }
    }

    pub fn get_moveable_pawn(self: &Board, mut pos: usize, white: bool) -> Vec<usize> {
        let mut moveable: Vec<usize> = Vec::new();
        let dir: isize = if white {-8} else {8};
        let mut steps = 1;
        if (white && pos/8 == 6) || (!white && pos/8 == 1) {
            steps = 2;
        }
        if pos/8 != 0 && pos/8 != 7 {
            if pos%8!=0 {
                if util::is_enemy(self.board_pos[((pos as isize)+dir) as usize-1], white) {
                    moveable.push(((pos as isize)+dir-1) as usize);
                }
            }
            if pos%8!=7 {
                if util::is_enemy(self.board_pos[((pos as isize)+dir) as usize+1], white) {
                    moveable.push(((pos as isize)+dir) as usize+1);
                }
            }
        }
        for i in 0..steps {
            let intermediate = (pos as isize + dir);
            if intermediate < 0  || intermediate > 63{
                break;
            }
            if self.board_pos[intermediate as usize] != 0 {
                break;
            }
            pos = intermediate as usize;
            moveable.push(pos);
        }
        moveable
        
    }
 pub fn get_moveable_knight(self: &Board, mut pos: usize, white: bool) -> Vec<usize> {
        let mut moveable: Vec<usize> = Vec::new();
        let (x, y) = util::pos_to_xy(pos);

        for dir in util::DIRECTIONS {
            let intermediate = Vec2 {x: x as f32+dir.x*2.0, y: y as f32+dir.y*2.0};
            for i in [-1.0, 1.0] {
                let intermediate2 = Vec2{
                    x: intermediate.x + dir.y*i,
                    y: intermediate.y + dir.x*i,
                };
                if intermediate2.x > 7.0 || intermediate2.x < 0.0 || intermediate2.y < 0.0 || intermediate2.y > 7.0 {
                    continue;
                }
                let target_pos: usize = intermediate2.x as usize + (intermediate2.y as usize)*8;
                if !util::is_enemy(self.board_pos[target_pos], !white) {
                    moveable.push(target_pos);
                }
            }
        }

        moveable
    }

    pub fn get_moveable_rook(self: &Board, mut pos: usize, white: bool) -> Vec<usize> {
        let mut moveable: Vec<usize> = Vec::new();
        let pos_vec = util::pos_to_vec(pos);
        for dir in util::DIRECTIONS {
            for offset in 1..8 {
                let look_vec = pos_vec.add(dir.mul(offset as f32));
                if look_vec.x >= 0.0 && look_vec.y >= 0.0 && look_vec.x <= 7.0 && look_vec.y <= 7.0 {
                    let look_usize: usize = look_vec.x as usize + (look_vec.y as usize) * 8;
                    if util::is_enemy(self.board_pos[look_usize], !white) {
                        break;
                    }
                    moveable.push(look_usize);
                    if util::is_enemy(self.board_pos[look_usize], white) {
                        break;
                    }
                }
            }
        }
        moveable
    }
    pub fn get_moveable_bishop(self: &Board, mut pos: usize, white: bool) -> Vec<usize> {
        let mut moveable: Vec<usize> = Vec::new();
        let pos_vec = util::pos_to_vec(pos);
        for dir in util::DIAGONALS {
            for offset in 1..8 {
                let look_vec = pos_vec.add(dir.mul(offset as f32));
                if look_vec.x >= 0.0 && look_vec.y >= 0.0 && look_vec.x <= 7.0 && look_vec.y <= 7.0 {
                    let look_usize: usize = look_vec.x as usize + (look_vec.y as usize) * 8;
                    if util::is_enemy(self.board_pos[look_usize], !white) {
                        break;
                    }
                    moveable.push(look_usize);
                    if util::is_enemy(self.board_pos[look_usize], white) {
                        break;
                    }
                }
            }
        }
        moveable
    }

    pub fn get_moveable_queen(self: &Board, mut pos: usize, white: bool) -> Vec<usize> {
        let mut moveable = self.get_moveable_rook(pos, white);
        moveable.extend(self.get_moveable_bishop(pos, white));
        moveable
    }

    pub fn get_moveable_king(self: &Board, mut pos: usize, white: bool) -> Vec<usize> {
        let mut moveable = Vec::new();
        let pos_vec = util::pos_to_vec(pos);
        let all_dirs: Vec<Vec2> = util::DIRECTIONS.iter().chain(util::DIAGONALS.iter()).cloned().collect();
        for dir in all_dirs {
            let look_vec = pos_vec.add(dir);
            if look_vec.x >= 0.0 && look_vec.y >= 0.0 && look_vec.x <= 7.0 && look_vec.y <= 7.0 {
                let look_usize: usize = look_vec.x as usize + (look_vec.y as usize) * 8;
                if util::is_enemy(self.board_pos[look_usize], !white) {
                    continue;
                }
                moveable.push(look_usize);
            }
        }
        let (x, y) = util::pos_to_xy(pos);
        if !*self.has_king_moved(white) {
            // We know that the king is in its spot.
            let mut can_castle = false;
            for offset in 1..5 { //Castle right
                if x+offset > 7 {
                    break;
                }
                let look_usize = (x + offset) + y*8;
                if self.board_pos[look_usize] != 0 {
                    if util::PIECE_TO_COLOURLESS[self.board_pos[look_usize]] == 5 {
                        if (look_usize % 8) == 7 {
                            can_castle = true;
                            break;
                        }
                    }
                    can_castle = false;
                    break;
                }
            }
            if can_castle {
                moveable.push((x+2)+y*8);
            }
            can_castle = false;
            for offset in 1..5 { //Castle left
                if offset > x {
                    break;
                }
                let look_usize = (x - offset) + y*8;
                if self.board_pos[look_usize] != 0 {
                    if util::PIECE_TO_COLOURLESS[self.board_pos[look_usize]] == 5 {
                        if x - offset == 0 {
                            can_castle = true;
                            break;
                        }
                    }
                    can_castle = false;
                    break;
                }
            }
            if can_castle {
                moveable.push((x-2)+y*8);
            }
        }
        moveable
    }

    pub fn get_moveable_squares(self: &Board, pos: usize) -> Vec<usize> {
        let moveable: Vec<usize> = Vec::new();
        let piece = self.board_pos[pos];
        if piece == 0 {return moveable}
        let white = piece <= 6;
        let piece = util::PIECE_TO_COLOURLESS[piece];
        let moveable = match piece {
            1 => self.get_moveable_king(pos, white),
            2 => self.get_moveable_queen(pos, white),
            3 => self.get_moveable_bishop(pos, white),
            4 => self.get_moveable_knight(pos, white),
            5 => self.get_moveable_rook(pos, white),
            6 => self.get_moveable_pawn(pos, white),
            _ => moveable,
        };

        moveable
    }
    pub fn rate_board(&self) -> i32 {
        let mut score: i32 = 0;
        for pos in 0..64usize {
            let piece = self.board_pos[pos];
            score += util::PIECE_VALUES[piece];
        }
        score
    }
}