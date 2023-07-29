pub const window_width: f32 = 400.0;
use ggez::conf;
use ggez::graphics::StrokeOptions;
use ggez::mint::{self, Vector2};
use ggez::event::MouseButton;
use ggez::input::mouse::button_pressed;
use ggez::filesystem;
use ggez::GameError;

use crate::util::{self, bitboard_to_vec, BLACK_PAWN_CAPTURES};
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
extern crate bitintr;
use bitintr::*;
use std::sync::Once;

pub struct Board {
    pub is_whites_turn: bool,
    pub has_moved_king_white: bool,
    pub has_moved_king_black: bool,
    pub en_passant: usize,

    pub board_pos: [usize; 64],
    pub white_bitboard: u64,
    pub black_bitboard: u64,
    // pub pieces_bitboard: [u64; 13],
    pub score: i32,
    pub check_for_draws: [u64; 75], //this works for 75 moves.
    pub check_for_draws_idx: usize,
}
impl Clone for Board {
    fn clone(&self) -> Board {
        Board {
            is_whites_turn: self.is_whites_turn,
            has_moved_king_white: self.has_moved_king_white,
            has_moved_king_black: self.has_moved_king_black,
            en_passant: self.en_passant,
            board_pos: self.board_pos,
            score: self.score,
            white_bitboard: self.white_bitboard,
            black_bitboard: self.black_bitboard,
            check_for_draws: self.check_for_draws,
            check_for_draws_idx: self.check_for_draws_idx,
            // pieces_bitboard: self.pieces_bitboard,
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
        let mut pieces_bitboard: [u64; 13] = [0; 13];
        for i in 0..64 {
            pieces_bitboard[board_pos[i]] |= 1<<i;
        }
        let white_bitboard = (1..=6).map(|i| pieces_bitboard[i]).reduce(|a, b| a|b).unwrap();
        let black_bitboard = (7..=12).map(|i| pieces_bitboard[i]).reduce(|a, b| a|b).unwrap();

        Board {
            board_pos,
            has_moved_king_white: false,
            has_moved_king_black: false,
            en_passant: 0,
            is_whites_turn: true,
            score: 0,
            white_bitboard,
            black_bitboard,
            check_for_draws: [0u64; 75],
            check_for_draws_idx: 0usize,
            // pieces_bitboard,
        }
    }

    pub fn get_friendly_pieces_for_mut(&mut self, is_white: bool) -> &mut u64 {
        if is_white {
            &mut self.white_bitboard
        } else {
            &mut self.black_bitboard
        }
    }
    pub fn get_friendly_pieces_for(&self, is_white: bool) -> u64 {
        if is_white {
            self.white_bitboard
        } else {
            self.black_bitboard
        }
    }
    pub fn hash_board(&self) -> u64 {
        self.black_bitboard & self.white_bitboard
    }
    pub fn move_square(self: &mut Board, old: usize, new: usize) -> bool {
        let mut piece = self.board_pos[old];
        self.score -= util::PIECE_VALUES_POSITION[piece][old];
        let is_white = util::is_piece_white(piece);
        let mut is_reversible_move = true;
        self.en_passant = 0;
        match util::PIECE_TO_COLOURLESS[piece] {
            crate::KING => {
                is_reversible_move &= !self.has_king_moved(is_white);
                *self.has_king_moved_mut(is_white) = true;
                if util::pos_to_xy(old).0.abs_diff(util::pos_to_xy(new).0) == 2 {
                    let rook_x = if util::pos_to_xy(old).0 < util::pos_to_xy(new).0 {
                        7usize
                    } else {
                        0usize
                    };
                    let rook_y = util::pos_to_xy(old).1;
                    let rook_pos = rook_x + rook_y*8;
                    self.move_square(rook_pos, (old+new)/2);
                    self.is_whites_turn = !self.is_whites_turn; //fixed a bug where castling would cause yes.
                }
            }
            crate::PAWN => {
                is_reversible_move = false;
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
        let captured_piece = self.board_pos[new];
        let is_capture = captured_piece > 0;
        is_reversible_move &= !is_capture;
        self.score -= util::PIECE_VALUES_POSITION[captured_piece][new];
        self.score += util::PIECE_VALUES_POSITION[piece][new];
        // self.pieces_bitboard[captured_piece] ^= 1<<new;
        // self.pieces_bitboard[piece] ^= 1<<old;
        // self.pieces_bitboard[piece] |= 1<<new;
        *self.get_friendly_pieces_for_mut(!is_white) &= !(1<<new); //register opponent gone
        *self.get_friendly_pieces_for_mut(is_white) ^= 1<<old; //register piece himself no longer there
        *self.get_friendly_pieces_for_mut(is_white) |= 1<<new; //register piece at new location

        self.check_for_draws[self.check_for_draws_idx] = self.hash_board();
        self.check_for_draws_idx += 1;
        if !is_reversible_move {
            self.check_for_draws_idx = 0;
        }

        self.board_pos[new] = piece;
        self.board_pos[old] = 0;
        self.is_whites_turn = !self.is_whites_turn;
        is_capture
    }

    pub fn has_king_moved_mut(self: &mut Board, is_white: bool) -> &mut bool {
        if is_white {
            &mut self.has_moved_king_white
        } else {
            &mut self.has_moved_king_black }
    }
    pub fn has_king_moved(self: &Board, is_white: bool) -> &bool {
        if is_white {
            &self.has_moved_king_white
        } else {
            &self.has_moved_king_black
        }
    }

    pub fn get_moveable_pawn(self: &Board, mut pos: usize, white: bool) -> u64 {
        let offset: isize = (white as isize * -8) | ((!white) as isize * 8);
        let can_double_push: u64 = (pos/8 == (if white {6} else {1})) as u64;
        let colour_pawn_captures: &[u64; 64] = if white {&*util::WHITE_PAWN_CAPTURES} else {&*util::BLACK_PAWN_CAPTURES};
        //don't ask me about the &*. I think it is some special type unwrapping.
        let nothing_board = !(self.white_bitboard | self.black_bitboard);
        let mut moveable: u64 = util::fix_shl(1, pos as isize + offset) & nothing_board; //single forward
        moveable |= can_double_push * (util::fix_shl(moveable, offset) & nothing_board); //double push.
        moveable |= colour_pawn_captures[pos] & self.get_friendly_pieces_for(!white);
        moveable
    }
    pub fn get_moveable_knight(self: &Board, mut pos: usize, white: bool) -> u64 {
        let mut moveable = util::KNIGHT_MOVES[pos];
        moveable &= !self.get_friendly_pieces_for(white);
        moveable
    }

    pub fn get_moveable_rook(self: &Board, mut pos: usize, white: bool) -> u64 {
        let moveable: u64 = util::ROOK_MOVES[pos];
        let entry_friendly = self.get_friendly_pieces_for(white).pext(moveable) as usize;
        let entry_enemy = self.get_friendly_pieces_for(!white).pext(moveable) as usize;
        let mut moveable = util::ROOK_OBSTRUCTION_SELF_MAP[pos][entry_friendly];
        moveable &= util::ROOK_OBSTRUCTION_OPPONENT_MAP[pos][entry_enemy];
        moveable
    }
    pub fn get_moveable_bishop(self: &Board, mut pos: usize, white: bool) -> u64 {
        let mut moveable: u64 = util::BISHOP_MOVES[pos];
        let entry_friendly = self.get_friendly_pieces_for(white).pext(moveable) as usize;
        let entry_enemy = self.get_friendly_pieces_for(!white).pext(moveable) as usize;
        let mut moveable = util::BISHOP_OBSTRUCTION_SELF_MAP[pos][entry_friendly];
        moveable &= util::BISHOP_OBSTRUCTION_OPPONENT_MAP[pos][entry_enemy];
        moveable
    }

    pub fn get_moveable_queen(self: &Board, mut pos: usize, white: bool) -> u64 {
        self.get_moveable_bishop(pos, white) | self.get_moveable_rook(pos, white)
    }

    pub fn get_moveable_king(self: &Board, mut pos: usize, white: bool) -> u64 {
        let (x, y) = util::pos_to_xy(pos);
        let pos_vec = util::pos_to_vec(pos);

        let mut mov_bits: u64 = util::KING_MOVES[pos];
        mov_bits &= !self.get_friendly_pieces_for(white);

        if !self.has_king_moved(white) {
            let row_offset: isize = if white {7*8} else {0*8};
            let left_castle:  u64 = util::fix_shl(0b00001110, row_offset);
            let right_castle: u64 = util::fix_shl(0b01100000, row_offset);
            let can_left_castle = (left_castle & self.get_friendly_pieces_for(white)) == 0 &&
                util::PIECE_TO_COLOURLESS[self.board_pos[0+row_offset as usize]] == crate::ROOK;
            let can_right_castle = (right_castle & self.get_friendly_pieces_for(white)) == 0 &&
                util::PIECE_TO_COLOURLESS[self.board_pos[7+row_offset as usize]] == crate::ROOK;
            mov_bits |= (can_left_castle as u64) << (pos as u64 - 2);
            mov_bits |= (can_right_castle as u64) << (pos as u64 + 2);
        }
        mov_bits
    }

    pub fn get_moveable_squares(self: &Board, pos: usize) -> u64 {
        let piece = self.board_pos[pos];
        let white = util::is_piece_white(piece);
        let piece = util::PIECE_TO_COLOURLESS[piece];
        let bit_moveable: u64 = match piece {
            crate::NOTHING => 0u64,
            crate::KING => self.get_moveable_king(pos, white),
            crate::QUEEN => self.get_moveable_queen(pos, white),
            crate::BISHOP => self.get_moveable_bishop(pos, white),
            crate::KNIGHT => self.get_moveable_knight(pos, white),
            crate::ROOK => self.get_moveable_rook(pos, white),
            crate::PAWN => self.get_moveable_pawn(pos, white),
            _ => 0u64, // unreachable code.
        };
        bit_moveable
    }
    pub fn get_moveable_squares_with_checks(self: &Board, pos: usize) -> u64 {
        let mut moveable = self.get_moveable_squares(pos);
        // let is_white = self.white_bitboard & (1 << pos) > 0;
        // if util::PIECE_TO_COLOURLESS[self.board_pos[pos]] == crate::KING {
            // moveable &= !(
                // util::BitIter::new(self.get_friendly_pieces_for(!is_white))
                // .fold(0u64, |old, pos| old|self.get_moveable_squares(pos))
            // );
        // } else {
            // let king_pos = (0..64usize).filter(|&pos| {
                // let piece = self.board_pos[pos];
                // util::PIECE_TO_COLOURLESS[self.board_pos[pos]] == crate::KING
                // && (util::is_piece_white(piece) == is_white)
            // }).collect::<Vec<usize>>()[0];
            // let is_in_check = util::BitIter::new(self.get_friendly_pieces_for(is_white))
                // .any(|pos| (self.get_moveable_squares(pos) & (1<<king_pos)) > 0);
            // if is_in_check {
                // moveable = 0;
            // }
        // }
        moveable
    }
    pub fn rate_board(&self) -> i32 {
        self.score
    }
}