use crate::{Battlesnake, Board, Coord};
use crate::logic::get_safe_moves;
use std::{
    collections::HashMap,
    convert::TryInto,
};
use log::info;

use crate::board_rep::{move_snake, check_deaths};

pub fn search(_board: &Board, team_ids: &[String; 2], enemy_ids: &[String; 2], timeout: u32) -> [String; 2] {
    let joint_moves: Vec<[&str; 2]> = get_joint_moves(_board, team_ids);
    let mut values: Vec<i32> = Vec::new();
    let mut moves: Vec<[&str; 2]> = Vec::new();

    for move_pair in joint_moves {
        let mut new_board = _board.clone();
        for (i, id) in team_ids.iter().enumerate() {
            let m: &str = move_pair[i];
            new_board = move_snake(&new_board, id, m);
        }
        new_board = check_deaths(&new_board, false);
        let value = heuristic(&new_board, team_ids, enemy_ids);
        values.push(value);
        moves.push(move_pair);
    }

    info!("Values: {:?} Moves: {:?}", values, moves);

    let mut best_value = i32::MIN;
    let mut best_move = &["down", "down"]; // Default move
    for (i, value) in values.iter().enumerate() {
        if *value > best_value {
            best_value = *value;
            best_move = &moves[i];
        }
    }

    best_move.iter().map(|&s| s.to_string()).collect::<Vec<String>>().try_into().unwrap_or_else(|_| ["down".to_string(), "down".to_string()])
}

fn heuristic(_board: &Board, team_ids: &[String; 2], enemy_ids: &[String; 2]) -> i32 {
    let mut value = 0;
    for id in team_ids {
        if let Some(snake) = _board.snakes.iter().find(|s| s.id == *id) {
            value += snake.length;
            if snake.health < 50 {
                value -= 1; // Penalize for low health
            }
        }
    }
    for id in enemy_ids {
        if let Some(snake) = _board.snakes.iter().find(|s| s.id == *id) {
            value -= snake.length;
            if snake.health < 50 {
                value += 1; // Reward for low health
            }
        }
    }
    value
}

fn get_joint_moves<'a>(_board: &'a Board, team_ids: &'a [String; 2]) -> Vec<[&'a str; 2]> {
    let snake_map: HashMap<String, &Battlesnake> = _board
        .snakes
        .iter()
        .map(|s| (s.id.clone(), s))
        .collect();

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
            move_pair[0] = m1.clone();
            move_pair[1] = m2.clone();
            team_moves_combinations.push(move_pair);
        }
    }
    team_moves_combinations
}