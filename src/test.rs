use crate::*;
use std::time::{SystemTime, UNIX_EPOCH};
use rand::Rng;

pub fn benchmark_performance(game: &mut render::Game) {
    println!("Going to run all tests in test folder.");
    let bef = SystemTime::now();
    let mut evaluations: usize = 0;
    let runs_on_tests: usize = 10;
    let mut RNG = rand::thread_rng();
    for i in 0..runs_on_tests {
        let dir = "test/";
        match std::fs::read_dir(dir) {
            Ok(files) => {
                for file in files {
                    if let Ok(file) = file {
                        game.load_game(format!("{}{}", dir, file.file_name().to_str().unwrap()).as_str());
                        let is_cpu_white = game.board.is_whites_turn;
                        cpu::calculate_best_move(&mut game.board, &mut evaluations, crate::DEPTH, crate::MAX_DEPTH, -100_000, is_cpu_white, true, &mut RNG);
                    }
                }

            }
            Err(e) => {},
        }
        println!("Finished {}% of tests.", (1+i)*100 / runs_on_tests);
    }
    println!("Run all test games, took {} seconds!", bef.elapsed().unwrap().as_secs());
}

pub fn benchmark_quality(game: &mut render::Game) {
    let mut wins_new = 0;
    let mut beta = true;
    let total_games: usize = 100;
    for i in 0..total_games {
        game.load_game("test/start.txt");
        loop {
            cpu::make_bot_move(game, beta);
            if game.board.rate_board().abs() > 80_000 { //checkmate
                if beta {
                    wins_new += 1;
                }
                break;
            }
            beta = !beta;
        }
        println!("Completed {}/{} games.", i+1, total_games);
    }
    println!("Beta won {}/{} games.", wins_new, total_games);
}