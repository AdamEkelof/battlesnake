use crate::{Battlesnake, Board, Coord};
use log::info;

pub fn move_snake(
    board: &Board,
    id: &str,
    m: &str, // Assumed to be safe move
) -> Board {
    let mut new_board = board.clone();
    let mut snake: &mut Battlesnake;
    if let Some(s) = new_board.snakes.iter_mut().find(|s| s.id == id) {
        snake = s;
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
        new_board.food.retain(|f| *f != new_head);
        snake.health = 100;
        snake.length += 1;
    } else {
        // Remove the last segment of the snake's body
        snake.body.pop();
        snake.health -= 1;
        if snake.health <= 0 {
            // If the snake's health is 0, remove it from the board
            new_board.snakes.retain(|s| s.id != id);
        }
    }

    info!("Board after move: {}", new_board);

    new_board
}

pub fn check_deaths(board: &Board, full_turn: bool) -> Board {
    let mut new_board = board.clone();

    // Check for head-on-head collisions
    for (i, snake) in board.snakes[..board.snakes.len() - 1].iter().enumerate() {
        for other_snake in board.snakes[i+1..].iter() {
            info!("Checking head-on-head collision between {} and {}: {} {}", snake.id, other_snake.id, snake.body[0], other_snake.body[0]);
            if snake.body[0] == other_snake.body[0] {
                if snake.length > other_snake.length {
                    new_board.snakes.retain(|s| s.id != other_snake.id);
                    info!("Snake {} died in head-on-head collision", other_snake.id);
                } else if snake.length < other_snake.length {
                    new_board.snakes.retain(|s| s.id != snake.id);
                    info!("Snake {} died in head-on-head collision", snake.id);
                } else {
                    // Both snakes die
                    new_board.snakes.retain(|s| s.id != snake.id && s.id != other_snake.id);
                    info!("Both snakes died in head-on-head collision");
                }
            }
        }
    }

    let mut new_board_snakes = new_board.snakes.clone();

    // Check for snake collisions with other snakes
    for i in 0..new_board.snakes.len() {
        let snake_id = new_board.snakes[i].id.clone();
        let snake_head = new_board.snakes[i].body[0];
        for j in 0..new_board.snakes.len() {
            if i == j {
                continue; // Skip self-collision, handled by safe moves
            }
            let other_snake = &new_board.snakes[j];
            let other_snake_body: Vec<Coord>;
            if full_turn {
                other_snake_body = other_snake.body.clone();
            } else {
                other_snake_body = other_snake.body[..other_snake.body.len() - 1].iter().map(|c| *c).collect();
            }
            if other_snake_body.contains(&snake_head) {
                new_board_snakes.retain(|s| s.id != snake_id);
            }
        }
    }

    new_board.snakes = new_board_snakes;

    new_board
}