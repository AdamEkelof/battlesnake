use crate::{Battlesnake, Board, Coord, Game, GameInfo};
use std::{
    collections::{HashMap, VecDeque},
    hash::Hash,
};
use log::info;

pub fn move_snake(
    board: &Board,
    id: &str,
    m: &str, // Assumed to be safe and valid
) -> Board {
    let mut new_board = board.clone();
    let mut snake: Battlesnake;
    if let Some(s) = new_board.snakes.iter_mut().find(|s| s.id == id) {
        snake = s.clone();
    } else {
        return new_board; // Snake not found, return the board unchanged
    }
    let head = snake.body[0];
    let mut new_head = head;

    match m {
        "up" => new_head.y += 1,
        "down" => new_head.y -= 1,
        "left" => new_head.x -= 1,
        "right" => new_head.x += 1,
        _ => panic!("Invalid move"),
    }

    // Move the snake's body
    snake.body.insert(0, new_head);
    if new_board.food.contains(&new_head) {
        // If the snake eats food, don't remove the last segment
        new_board.food.retain(|f| f != &new_head);
        snake.health = 100;
        snake.length += 1;
    } else { // May need to check for death if safe assumption is not addequate
        // Remove the last segment of the snake's body
        snake.body.pop();
        snake.health -= 1;
        if snake.health <= 0 {
            // If the snake's health is 0, remove it from the board
            new_board.snakes.retain(|s| s.id != id);
        }
    }

    new_board
}

pub fn kill_snake(
    board: &Board,
    id: &str,
) -> Board {
    let mut new_board = board.clone();
    new_board.snakes.retain(|s| s.id != id);
    new_board
}