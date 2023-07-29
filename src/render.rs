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

use std::fs::File;
use std::io::prelude::*;

pub const BORDER_SIZE: f32 = 10.0;
pub const DX: f32 = BORDER_SIZE+40.0+BORDER_SIZE+20.0+BORDER_SIZE;
pub const DY: f32 = 0.0+BORDER_SIZE;
pub const WINDOW_WIDTH: f32 = 600.0 + DX+BORDER_SIZE;
pub const WINDOW_HEIGHT: f32 = 600.0 + DY + BORDER_SIZE;
pub const SQUARE_SIZE: f32 = (WINDOW_WIDTH-DX-BORDER_SIZE) / 8.0;

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
    pub history: Vec<(usize, usize)>,
}

impl event::EventHandler<ggez::GameError> for Game {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if *self.get_current_player_type() == Player::BOT {
            cpu::make_bot_move(self, false);
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));

        for x in 0..8 {
            for y in 0..8 {
                let square = if (x+y)%2==0 {&self.square_light} else {&self.square_dark};
                canvas.draw(square, Vec2::new((x as f32)*SQUARE_SIZE+DX, (y as f32)*SQUARE_SIZE+DY));
            }
        }
        for x in 0..8 {
            for y in 0..8 {
                let piece = self.board.board_pos[x+y*8];
                if piece == 0 {continue;}
                let sprite = &self.pieces[piece-1];
                let scale = mint::Vector2 {x: SQUARE_SIZE / (sprite.width() as f32), y: SQUARE_SIZE / (sprite.height() as f32)};
                let dest = mint::Vector2 {x: (x as f32)*SQUARE_SIZE+DX, y: (y as f32)*SQUARE_SIZE+DY};
                let draw_param = graphics::DrawParam::new()
                    .dest(dest)
                    .scale(scale);
                canvas.draw(sprite, draw_param);
            }
        }
        if self.selected_square.is_some() {
            let x = self.selected_square.unwrap() % 8;
            let y = self.selected_square.unwrap() / 8;
            canvas.draw(&self.square_highlight, Vec2::new((x as f32)*SQUARE_SIZE+DX, (y as f32)*SQUARE_SIZE+DY));

            let pos = self.selected_square.unwrap();
            let mut moveable = self.board.get_moveable_squares_with_checks(pos);
            for p in util::BitIter::new(moveable) {
                let (x, y) = util::pos_to_xy(p);
                canvas.draw(&self.square_moveable, Vec2::new((x as f32)*SQUARE_SIZE+DX, (y as f32)*SQUARE_SIZE+DY));
            }
        }
        
        let board_rating = util::sigmoid(self.board.rate_board() as f32 / -300.0);
        let white = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(0.0,0.0, 20.0, 8.0*SQUARE_SIZE),
            Color::WHITE,
        )?;
        let black = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(0.0,0.0, 20.0, 8.0*SQUARE_SIZE*board_rating),
            Color::BLACK,
        )?;
        canvas.draw(&white, Vec2::new(DX-BORDER_SIZE-20.0, BORDER_SIZE));
        canvas.draw(&black, Vec2::new(DX-BORDER_SIZE-20.0, BORDER_SIZE));
        let button = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(0.0,0.0, 40.0, 20.0),
            Color::from_rgb(00, 00, 00),
        )?;
        canvas.draw(&button, Vec2::new(BORDER_SIZE, BORDER_SIZE));
        let mut text = graphics::Text::new("Save");
        text.set_bounds(Vec2::new(400.0, f32::INFINITY))
            .set_layout(graphics::TextLayout {
                h_align: graphics::TextAlign::Middle,
                v_align: graphics::TextAlign::Middle,
            });
        canvas.draw(&text, Vec2::new(BORDER_SIZE+20.0, BORDER_SIZE+10.0));
        let button = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(0.0,0.0, 40.0, 20.0),
            Color::from_rgb(00, 00, 00),
        )?;
        canvas.draw(&button, Vec2::new(BORDER_SIZE, 2.0*BORDER_SIZE+20.0));
        let mut text = graphics::Text::new("Load");
        text.set_bounds(Vec2::new(400.0, f32::INFINITY))
            .set_layout(graphics::TextLayout {
                h_align: graphics::TextAlign::Middle,
                v_align: graphics::TextAlign::Middle,
            });
        canvas.draw(&text, Vec2::new(BORDER_SIZE+20.0, 2.0*BORDER_SIZE+30.0));
        canvas.finish(ctx)?;

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
            if x >= DX {
                // Store the clicked position
                let clicked_pos = (((x-DX) / SQUARE_SIZE).floor() + 8.0*((y-DY)/SQUARE_SIZE).floor()) as usize;
                if *self.get_current_player_type() == Player::HUMAN {
                    if self.selected_square.is_some() {
                        if (self.board.board_pos[self.selected_square.unwrap()] < 7) == self.board.is_whites_turn {
                            let mut moveable = self.board.get_moveable_squares(self.selected_square.unwrap());
                            for p in util::BitIter::new(moveable) {
                                if p == clicked_pos {
                                    self.move_square(self.selected_square.unwrap(), clicked_pos);
                                }
                            }
                        }
                    }
                }
                if clicked_pos < 64 { //unsigned, so can't be bellow 0.
                    self.selected_square = Some(clicked_pos);
                }
            } else {
                if x > BORDER_SIZE && y > BORDER_SIZE && x < BORDER_SIZE+40.0 && y < BORDER_SIZE+20.0 {
                    self.save_game("games/lagame.txt");
                }
                if x > BORDER_SIZE && y > 20.0+2.0*BORDER_SIZE && x < BORDER_SIZE+40.0 && y < 2.0*BORDER_SIZE+40.0 {
                    self.load_game("games/lagame.txt")
                }
            }
        }
        return Ok(());
    }
    fn mouse_button_up_event(
            &mut self,
            ctx: &mut Context,
            button: MouseButton,
            x: f32,
            y: f32,
        ) -> Result<(), ggez::GameError> {
        self.mouse_button_down_event(ctx, button, x, y)
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

    let mut cb = ggez::ContextBuilder::new("Chess", "hobrin");
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

    let mut state = Game::new(&mut ctx, player_white, player_black)?;
    if args.len() >= 2 && args[1] == "test" {
        test::benchmark_performance(&mut state);
    }
    if args.len() >= 2 && args[1] == "test_rating" {
        test::benchmark_quality(&mut state);
    }
    event::run(ctx, event_loop, state) // Dereference event_loop
}

impl Game {
    pub fn save_game(&mut self, file_path: &str) {
        println!("Saving game!");
        let mut file = match File::create(file_path) {
            Ok(f) => f,
            Err(e) => panic!("Failed to create file: {}", e),
        };

        let mut data = "".to_string();
        for (from, to) in self.history.iter() {
            data += format!("{} {}\n", from, to).as_str();
        }
        let _ = file.write_all(data.as_bytes());
        let _ = file.flush();
    }
    pub fn load_game(&mut self, file_path: &str) {
        println!("Loading game!");
        let mut file = match File::open(file_path) {
            Ok(f) => f,
            Err(e) => panic!("Failed to open file: {}", e),
        };

        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Ok(_) => {
                let mut history: Vec<(usize, usize)> = vec![];
                for line in contents.split("\n").into_iter() {
                    if line.is_empty() {continue;}
                    let move_: Vec<&str> = line.split(" ").collect();
                    let from: usize;
                    let to: usize;
                    match move_[0].to_string().parse::<usize>() {
                        Ok(parsed_move) => {
                            from = parsed_move;
                        }
                        Err(e) => {
                            panic!("Corrupted file!: {}", e);
                        },
                    }
                    match move_[1].to_string().parse::<usize>() {
                        Ok(parsed_move) => {
                            to = parsed_move;
                        }
                        Err(e) => {
                            panic!("Corrupted file!: {}", e);
                        },
                    }
                    history.push((from, to));
                }
                self.board = Board::new();
                self.history = vec![];
                for (from, to) in history {
                    self.move_square(from, to);
                }
            },
            Err(e) => panic!("Failed to read the file: {}", e),
        }
    }
    pub fn move_square(&mut self, old: usize, new: usize) {
        self.history.push((old, new));
        self.board.move_square(old, new);
    }
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
            history: vec![],
        })
    }

}