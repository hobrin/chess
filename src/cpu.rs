use crate::{*, util::BitIter};
use std::hash;
use std::time::{SystemTime, UNIX_EPOCH};
use rand::rngs::ThreadRng;
use rand::Rng;


pub fn calculate_best_move(board: &mut Board, evaluations: &mut usize, depth: usize, max_depth: usize, norm: i32, is_cpu_white: bool, beta: bool, rng: &mut ThreadRng) -> (i32, (usize, usize)) {
    *evaluations += 1;
    let mut move_score = board.rate_board();
    move_score *= (board.is_whites_turn as i32)*2-1;
    if depth <= 0 || max_depth <= 0 || (move_score < norm && depth < 3) {
        // move_score += 100*max_depth as i32; //To make it prefer short term things.
        return (move_score, (64,64));
    }
    let mut best_move: (usize, usize) = (64, 64);
    let mut best_move_score: i32 = std::i32::MIN / 10;

    let mut total_options: u32 = 0;
    
    for pos in util::BitIter::new(board.get_friendly_pieces_for(board.is_whites_turn)) {
        let options_timer = profiler::start_timing("options_searching");
        let options = board.get_moveable_squares(pos);
        options_timer.stop();
        total_options += options.count_ones();
        for target in util::BitIter::new(options) {
            let clone_timer = profiler::start_timing("clone");
            let mut board_2 = board.clone();
            board_2.score = (board.score as f32 * 1.01) as i32;
            clone_timer.stop();

            //check for draws.
            let hash_board = board_2.hash_board();
            let mut is_game_finished: bool = false;
            // for i in 0..=board_2.check_for_draws_idx {
            //     if board_2.check_for_draws[i] == hash_board {
            //         is_game_finished = true;
            //         board_2.score = 0;
            //     }
            // }

            let nothing = profiler::start_timing("nothing");
            nothing.stop();

            let move_timer = profiler::start_timing("move_piece");
            let is_capture = board_2.move_square(pos, target);
            move_timer.stop();

            if !is_game_finished {
                let (mut move_score, _) = 
                    calculate_best_move(&mut board_2, evaluations, depth-(!is_capture as usize), max_depth-1, norm*-1, is_cpu_white, beta, rng);
                move_score *= -1;
            }
            if move_score > best_move_score || (crate::RANDOM && move_score == best_move_score && rng.gen_bool(0.5)){
                best_move = (pos, target);
                best_move_score = move_score;
            }
        }
    }
    // if max_depth > depth {
    //     let mut board_2 = board.clone();
    //     board_2.score = (board.score as f32 * 1.01) as i32;
    //     board_2.move_square(best_move.0, best_move.1);
    //     (move_score, _) = calculate_best_move(&mut &mut board_2, depth, max_depth-1, norm*-1, is_cpu_white);
    // }
    
    (best_move_score + total_options as i32, best_move)
}

pub fn make_bot_move(game: &mut crate::render::Game, beta: bool) {
    let mut RNG = rand::thread_rng();
    let total_timer = profiler::start_timing("total");
    let is_cpu_white = game.board.is_whites_turn;
    let mut evaluations: usize = 0;
    let (mut norm, _) = cpu::calculate_best_move(&mut game.board, &mut evaluations, crate::NORM_EXPLR_DEPTH, crate::NORM_EXPLR_DEPTH, -100_000, is_cpu_white, beta, &mut RNG);
    norm -= crate::NORM;
    // println!("{}", norm);
    let mut depth = crate::DEPTH;
    let mut max_depth = crate::MAX_DEPTH;
    let mut score ;
    let mut from;
    let mut to;
    loop {
        let bef = SystemTime::now();
        (score, (from, to)) = cpu::calculate_best_move(&mut game.board, &mut evaluations, depth, max_depth, norm, is_cpu_white, beta, &mut RNG);
        if bef.elapsed().unwrap().as_millis() > 100 {
            break;
        }
        depth += 1;
        max_depth += 1;
    }    
    println!("Depth: {}. Evaluations: {}M", depth, evaluations as f32 / 1_000_000.0);

    total_timer.stop();
    println!("CPU score is: {}", score);
    game.move_square(from, to);
    println!("CPU score rn is: {}", game.board.rate_board());
    profiler::print();
}