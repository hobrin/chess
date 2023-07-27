use ggez::conf;
use ggez::graphics::StrokeOptions;
use ggez::mint::{self, Vector2};
use ggez::event::MouseButton;
use ggez::input::mouse::button_pressed;
use ggez::filesystem;
use ggez::GameError;

use crate::*;

// https://github.com/ggez/ggez/tree/master/examples
use ggez::{
    event,
    glam::*,
    graphics::{self, Color},
    Context, GameResult,
};
use std::ops::{Add, Mul};
use std::{env, path};

pub const WINDOW_HEIGHT: f32 = 700.0;
pub const WINDOW_WIDTH: f32 = 700.0;
pub const SQUARE_SIZE: f32 = WINDOW_WIDTH / 8.0;

#[derive(PartialEq)]
pub enum Player {
    HUMAN,
    BOT,
}
pub struct Game {
    pub board: Board, pub square_light: graphics::Mesh,
    pub square_dark: graphics::Mesh,
    pub square_highlight: graphics::Mesh,
    pub square_moveable: graphics::Mesh,
    pub pieces: [graphics::Image; 12],

    pub selected_square: Option<usize>,
    pub player_white: Player,
    pub player_black: Player,
}

impl event::EventHandler<ggez::GameError> for Game {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        // self.pos_x = self.pos_x % 800.0 + 1.0;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));

        for x in 0..8 {
            for y in 0..8 {
                let square = if (x+y)%2==0 {&self.square_light} else {&self.square_dark};
                canvas.draw(square, Vec2::new((x as f32)*SQUARE_SIZE, (y as f32)*SQUARE_SIZE));
            }
        }
        for x in 0..8 {
            for y in 0..8 {
                let piece = self.board.board_pos[x+y*8];
                if piece == 0 {continue;}
                let sprite = &self.pieces[piece-1];
                let scale = mint::Vector2 {x: SQUARE_SIZE / (sprite.width() as f32), y: SQUARE_SIZE / (sprite.height() as f32)};
                let dest = mint::Vector2 {x: (x as f32)*SQUARE_SIZE, y: (y as f32)*SQUARE_SIZE};
                let draw_param = graphics::DrawParam::new()
                    .dest(dest)
                    .scale(scale);
                canvas.draw(sprite, draw_param);
            }
        }
        if self.selected_square.is_some() {
            let x = self.selected_square.unwrap() % 8;
            let y = self.selected_square.unwrap() / 8;
            canvas.draw(&self.square_highlight, Vec2::new((x as f32)*SQUARE_SIZE, (y as f32)*SQUARE_SIZE));

            let pos = self.selected_square.unwrap();
            let mut moveable = self.board.get_moveable_squares_with_checks(pos);
            for p in util::BitIter::new(moveable) {
                let (x, y) = util::pos_to_xy(p);
                canvas.draw(&self.square_moveable, Vec2::new((x as f32)*SQUARE_SIZE, (y as f32)*SQUARE_SIZE));
            }

        }

        canvas.finish(ctx)?;

        if *self.get_current_player_type() == Player::BOT {
            cpu::make_bot_move(self);
        }

        Ok(())
    }
    fn mouse_button_down_event(
        self: &mut Game,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> Result<(), GameError> {
        if button == MouseButton::Left {
            // Store the clicked position
            let clicked_pos = ((x / SQUARE_SIZE).floor() + 8.0*(y/SQUARE_SIZE).floor()) as usize;
            if *self.get_current_player_type() == Player::HUMAN {
                if self.selected_square.is_some() {
                    if (self.board.board_pos[self.selected_square.unwrap()] < 7) == self.board.is_whites_turn {
                        let mut moveable = self.board.get_moveable_squares(self.selected_square.unwrap());
                        for p in util::BitIter::new(moveable) {
                            if p == clicked_pos {
                                self.board.move_square(self.selected_square.unwrap(), clicked_pos);
                            }
                        }
                    }
                }
            }
            self.selected_square = Some(clicked_pos);
        }
        return Ok(());
    }
}

pub fn main() -> GameResult<()> {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let mut cb = ggez::ContextBuilder::new("super_simple", "ggez");
    let mut cb = cb.add_resource_path(resource_dir);
    let mut cb = cb.window_mode(ggez::conf::WindowMode::default().dimensions(WINDOW_WIDTH, WINDOW_HEIGHT)); // Set the window dimensions here
    let (mut ctx, mut event_loop) = cb.build()?;

    let args = env::args().collect::<Vec<String>>();
    let mut player_white = Player::HUMAN;
    let mut player_black = Player::BOT;
    if args.len() >= 2 && args[1] == "b" {
        player_white = Player::BOT;
    }
    if args.len() >= 3 && args[2] == "h" {
        player_black = Player::HUMAN;
    }

    let state = Game::new(&mut ctx, player_white, player_black)?;
    event::run(ctx, event_loop, state) // Dereference event_loop
}

impl Game {
    pub fn get_current_player_type(&self) -> &Player {
        if self.board.is_whites_turn {
            &self.player_white
        } else {
            &self.player_black
        }
    }
    pub fn new(ctx: &mut Context, player_white: Player, player_black: Player) -> GameResult<Game> {
        let mut pieces: Vec<graphics::Image> = Vec::with_capacity(12);
        for colour in 0..2 {
            let colour_label = if colour==0 {"w"} else {"b"};
            for piece_id in 0..6 {
                let piece_label = "kqbnrp".chars().nth(piece_id).unwrap();
                let piece = graphics::Image::from_path(ctx, format!("/{}{}.png", colour_label, piece_label))?;
                pieces.push(piece);
            }
        }

        let size = SQUARE_SIZE;
        let square_light = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(0.0,0.0,size,size),
            Color::WHITE,
        )?;
        let square_dark = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(0.0,0.0,size,size),
            Color::from_rgb(96, 65, 58),
        )?;

        let highlight = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::stroke(5.0),
            graphics::Rect::new(0.0,0.0,size,size),
            Color::BLACK
        )?;

        let square_moveable = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            Vec2{x: size/2.0, y: size/2.0},
            10.0,
            2.0,
            Color::BLACK
        )?;

        let pieces: Box<[graphics::Image; 12]> = match pieces.into_boxed_slice().try_into() {
            Ok(ba) => ba,
            Err(o) => panic!("Expected a Vec of length {} but it was {}", 12, o.len()),
        };

        let board: Board = Board::new();

        Ok(Game {
            board,
            square_light,
            square_dark,
            pieces: *pieces,
            selected_square: None,
            square_highlight: highlight,
            square_moveable,
            player_white: player_white,
            player_black,
        })
    }

}