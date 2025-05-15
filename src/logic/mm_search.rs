use crate::logic::{flood_fill, get_safe_moves};
use crate::{Battlesnake, Board, Coord, GameInfo};
use log::info;
use std::{collections::HashMap, convert::TryInto, time::Instant};

use crate::board_rep::{check_deaths, move_snake};
use crate::logic::simple::{Movement, SimpleBoard};

pub fn search(board: &Board, game_info: &GameInfo, timeout: u32) -> (Movement, Movement) {
    let simple_board = SimpleBoard::from(board, game_info);
    let mut values = Vec::new();
    let mut moves = Vec::new();

    let mut best_value = i32::MIN;
    let simulations = simple_board.simulate_move(true);
    for (move_pair, next_board) in simulations {
        // minmax on enemies since this outer loop is on friendly
        let value = minmax_simple(
            &next_board,
            1,
            false,
            best_value,
            i32::MAX,
            5,
            15,
            timeout as i32,
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
        .expect("No best move found");
    moves[idx]
}

// pub fn search(
//     _board: &Board,
//     team_ids: &[String; 2],
//     enemy_ids: &[String; 2],
//     timeout: u32,
// ) -> [String; 2] {
//     let joint_moves: Vec<[&str; 2]> = get_joint_moves(_board, team_ids);
//     let mut values: Vec<i32> = Vec::new();
//     let mut moves: Vec<[&str; 2]> = Vec::new();

//     let mut best_value = i32::MIN;

//     for move_pair in joint_moves {
//         let mut new_board = _board.clone();
//         for (i, id) in team_ids.iter().enumerate() {
//             let m: &str = move_pair[i];
//             new_board = move_snake(&new_board, id, m);
//         }
//         new_board = check_deaths(&new_board, false);
//         let value = minimax(
//             &new_board,
//             team_ids,
//             enemy_ids,
//             1,
//             false,
//             best_value,
//             i32::MAX,
//             5,
//             15,
//             timeout as i32,
//         );
//         best_value = best_value.max(value);
//         values.push(value);
//         moves.push(move_pair);
//     }

//     info!("Values: {:?} Moves: {:?}", values, moves);

//     let mut best_value = i32::MIN;
//     let mut best_move = &["down", "down"]; // Default move
//     for (i, value) in values.iter().enumerate() {
//         if *value >= best_value {
//             best_value = *value;
//             best_move = &moves[i];
//         }
//     }

//     best_move
//         .iter()
//         .map(|&s| s.to_string())
//         .collect::<Vec<String>>()
//         .try_into()
//         .unwrap_or_else(|_| ["down".to_string(), "down".to_string()])
// }

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
        return board.heuristic();
    }

    let mut simulations = board.simulate_move(our_team);
    if our_team {
        simulations.sort_by_key(|s| -s.1.heuristic());
    } else {
        simulations.sort_by_key(|s| s.1.heuristic());
    }
    // Detta är värre än det som finns i den tidigare i att den beräknar heuristic en extra gång för en sim men kanske sparas temporärt i cache och mer lättläst
    if let Some(sim) = simulations.first() {
        if our_team && sim.1.heuristic() == i32::MAX {
            return i32::MAX;
        } else if !our_team && sim.1.heuristic() == i32::MIN {
            return i32::MIN;
        }
    }

    let mut best_value = if our_team { i32::MIN } else { i32::MAX };

    for (idx, (_, next_board)) in simulations.iter().enumerate() {
        let time_left = timeout - start.elapsed().as_millis() as i32 - return_time;
        if time_left <= 0 {
            info!("Ran out of time at depth {}", depth);
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

fn minimax(
    _board: &Board,
    team_ids: &[String; 2],
    enemy_ids: &[String; 2],
    depth: i32,
    is_maximizing: bool,
    mut alpha: i32,
    mut beta: i32,
    return_time: i32,
    heuristic_time: i32,
    timeout: i32,
) -> i32 {
    let start = Instant::now();
    if depth == 100 || heuristic_time + return_time >= timeout {
        return heuristic(_board, team_ids, enemy_ids);
    }

    let joint_moves: Vec<[&str; 2]> = get_joint_moves(_board, team_ids);
    let mut new_boards = vec![_board.clone(); joint_moves.len()];
    let mut heuristic_values: Vec<i32> = vec![0; joint_moves.len()];
    for (m_idx, move_pair) in joint_moves.iter().enumerate() {
        for (i, id) in team_ids.iter().enumerate() {
            let m: &str = move_pair[i];
            new_boards[m_idx] = move_snake(&new_boards[m_idx], id, m);
        }
        new_boards[m_idx] = check_deaths(&new_boards[m_idx], !is_maximizing);
        heuristic_values[m_idx] = heuristic(&new_boards[m_idx], team_ids, enemy_ids);
        if is_maximizing && heuristic_values[m_idx] == i32::MAX {
            return heuristic_values[m_idx];
        } else if !is_maximizing && heuristic_values[m_idx] == i32::MIN {
            return heuristic_values[m_idx];
        }
    }

    // Move ordering: sort joint_moves and new_boards by heuristic_values
    let mut combined: Vec<(_, _, _)> = joint_moves
        .into_iter()
        .zip(new_boards.into_iter())
        .zip(heuristic_values.into_iter())
        .map(|((jm, nb), hv)| (jm, nb, hv))
        .collect();

    if is_maximizing {
        combined.sort_by(|a, b| b.2.cmp(&a.2)); // Descending order for maximizing
    } else {
        combined.sort_by(|a, b| a.2.cmp(&b.2)); // Ascending order for minimizing
    }

    let (joint_moves, new_boards): (Vec<_>, Vec<_>) =
        combined.into_iter().map(|(jm, nb, _)| (jm, nb)).unzip();

    let mut best_value = if is_maximizing { i32::MIN } else { i32::MAX };

    for (m_idx, move_pair) in joint_moves.iter().enumerate() {
        let time_left: i32 = timeout - start.elapsed().as_millis() as i32 - return_time;
        if time_left <= 0 {
            info!("Ran out of time at depth {}", depth);
            return best_value;
        }

        let iterations_left: i32 = joint_moves.len() as i32 - m_idx as i32;
        let time_per_move: i32 = time_left / iterations_left;
        let value = minimax(
            &new_boards[m_idx],
            team_ids,
            enemy_ids,
            depth + 1,
            !is_maximizing,
            alpha,
            beta,
            return_time,
            heuristic_time,
            time_per_move,
        );
        if is_maximizing {
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

fn heuristic(_board: &Board, team_ids: &[String; 2], enemy_ids: &[String; 2]) -> i32 {
    if _board.snakes.len() == 0 {
        return 0; // No snakes on the board (tie)
    }
    let ff = flood_fill(_board);
    let mut value = 0;
    let mut no_team_snakes = true;
    for id in team_ids {
        if let Some(snake) = _board.snakes.iter().find(|s| s.id == *id) {
            let flood = ff
                .get(snake)
                .expect("No such snake in flood fill map")
                .len() as i32
                >> 2;
            value += flood;
            value += snake.length;
            if snake.health < 50 {
                value -= 1; // Penalize for low health
            }
            no_team_snakes = false;
        }
    }
    if no_team_snakes {
        return i32::MIN; // No team snakes left (loss)
    }
    let mut no_enemy_snakes = true;
    for id in enemy_ids {
        if let Some(snake) = _board.snakes.iter().find(|s| s.id == *id) {
            let flood = ff
                .get(snake)
                .expect("No such snake in flood fill map")
                .len() as i32
                >> 2;
            value -= flood;
            value -= snake.length;
            if snake.health < 50 {
                value += 1; // Reward for low health
            }
            no_enemy_snakes = false;
        }
    }
    if no_enemy_snakes {
        return i32::MAX; // No enemy snakes left (win)
    }
    value
}

fn get_joint_moves<'a>(_board: &'a Board, team_ids: &'a [String; 2]) -> Vec<[&'a str; 2]> {
    let snake_map: HashMap<String, &Battlesnake> =
        _board.snakes.iter().map(|s| (s.id.clone(), s)).collect();

    let mut team_moves: Vec<Vec<&str>> = vec![Vec::new(); 2];
    for (i, id) in team_ids.iter().enumerate() {
        let snake: &Battlesnake;
        if let Some(temp_snake) = snake_map.get(id) {
            snake = temp_snake;
        } else {
            team_moves[i] = vec!["down"]; // Default move
            continue; // Snake not found, skip to next
        }

        let safe_moves = get_safe_moves(_board, snake);
        if safe_moves.len() == 0 {
            team_moves[i] = vec!["down"]; // Default move
            continue; // No safe moves, skip to next
        }
        team_moves[i] = safe_moves;
    }

    let mut team_moves_combinations: Vec<[&str; 2]> = Vec::new();
    for m1 in &team_moves[0] {
        for m2 in &team_moves[1] {
            let mut move_pair: [&str; 2] = ["down", "down"];
            move_pair[0] = m1;
            move_pair[1] = m2;
            team_moves_combinations.push(move_pair);
        }
    }
    team_moves_combinations
}
