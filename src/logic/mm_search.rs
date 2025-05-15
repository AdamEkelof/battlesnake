use crate::{Board, /*Coord,*/ GameInfo};
use log::info;
use std::time::Instant;

use crate::logic::simple::{Movement, SimpleBoard};

pub fn search(board: &Board, game_info: &GameInfo) -> [Movement; 2] {
    let start = Instant::now();
    let simple_board = SimpleBoard::from(board, game_info);
    let timeout: i32 = game_info.timeout as i32 * 1_000_000; // Convert milliseconds to nanoseconds
    let mut values = Vec::new();
    let mut moves = Vec::new();

    let mut best_value = i32::MIN;
    let simulations = simple_board.simulate_move(true);
    for (i, (move_pair, next_board)) in simulations.iter().enumerate() {
        let time: i32 = (timeout - start.elapsed().as_nanos() as i32) / (simulations.len() as i32 - i as i32);
        info!("Move {} time: {} (timeout: {} elapsed: {})", i, time, timeout, start.elapsed().as_nanos());
        // minmax on enemies since this outer loop is on friendly
        let value = minmax_simple(
            &next_board,
            1,
            false,
            best_value,
            i32::MAX,
            1,
            1,
            time,
        );
        best_value = best_value.max(value);
        values.push(value);
        moves.push(move_pair);
    }
    let idx = values
        .iter()
        .enumerate()
        .max_by(|(_, v), (_, v2)| v.cmp(v2))
        .map(|(i, _)| i)
        .expect(&format!("No best move found in values: {:?} for {} moves", values, simulations.len()));
    *moves[idx]
}

fn minmax_simple(
    board: &SimpleBoard,
    depth: i32,
    our_team: bool,
    mut alpha: i32,
    mut beta: i32,
    heuristic_time: i32,
    return_time: i32,
    timeout: i32,
) -> i32 {
    let start = Instant::now();
    if depth == 100 || heuristic_time + return_time >= timeout {
        //info!("Depth {} reached", depth);
        return board.heuristic();
    }

    let mut simulations = board.simulate_move(our_team);
    if our_team {
        simulations.sort_by_key(|s| -s.1.heuristic());
    } else {
        simulations.sort_by_key(|s| s.1.heuristic());
    }
    
    if let Some(sim) = simulations.first() {
        let h = sim.1.heuristic();
        if our_team && h == i32::MAX {
            return i32::MAX;
        } else if !our_team && h == i32::MIN {
            return i32::MIN;
        }
    }

    let mut best_value = if our_team { i32::MIN } else { i32::MAX };

    for (idx, (_, next_board)) in simulations.iter().enumerate() {
        let time_left = timeout - start.elapsed().as_nanos() as i32 - return_time;
        if time_left <= 0 {
            //info!("Ran out of time at depth {}", depth);
            return best_value;
        }

        let iterations_left = simulations.len() as i32 - idx as i32;
        let time_per_move = time_left / iterations_left;
        let value = minmax_simple(
            &next_board,
            depth + 1,
            !our_team,
            alpha,
            beta,
            heuristic_time,
            return_time,
            time_per_move,
        );
        if our_team {
            best_value = best_value.max(value);
            alpha = alpha.max(best_value);
            if best_value >= beta {
                break;
            }
        } else {
            best_value = best_value.min(value);
            beta = beta.min(best_value);
            if best_value <= alpha {
                break;
            }
        }
    }

    best_value
}