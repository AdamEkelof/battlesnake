// Welcome to
// __________         __    __  .__                               __
// \______   \_____ _/  |__/  |_|  |   ____   ______ ____ _____  |  | __ ____
//  |    |  _/\__  \\   __\   __\  | _/ __ \ /  ___//    \\__  \ |  |/ // __ \
//  |    |   \ / __ \|  |  |  | |  |_\  ___/ \___ \|   |  \/ __ \|    <\  ___/
//  |________/(______/__|  |__| |____/\_____>______>___|__(______/__|__\\_____>
//
// This file can be a nice home for your Battlesnake logic and helper functions.
//
// To get you started we've included code to prevent your Battlesnake from moving backwards.
// For more info see docs.battlesnake.com

use log::info;
use rand::seq::SliceRandom;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::{Battlesnake, Board, Coord, Game};

// info is called when you create your Battlesnake on play.battlesnake.com
// and controls your Battlesnake's appearance
// TIP: If you open your Battlesnake URL in a browser you should see this data
pub fn info() -> Value {
    info!("INFO");

    return json!({
        "apiversion": "1",
        "author": "", // TODO: Your Battlesnake Username
        "color": "#888888", // TODO: Choose color
        "head": "default", // TODO: Choose head
        "tail": "default", // TODO: Choose tail
    });
}

// start is called when your Battlesnake begins a game
pub fn start(_game: &Game, _turn: &i32, _board: &Board, _you: &Battlesnake) {
    info!("GAME START");
}

// end is called when your Battlesnake finishes a game
pub fn end(_game: &Game, _turn: &i32, _board: &Board, _you: &Battlesnake) {
    info!("GAME OVER");
}

// move is called on every turn and returns your next move
// Valid moves are "up", "down", "left", or "right"
// See https://docs.battlesnake.com/api/example-move for available data
pub fn get_move(_game: &Game, turn: &i32, _board: &Board, you: &Battlesnake) -> Value {
    let mut is_move_safe: HashMap<_, _> = vec![
        ("up", true),
        ("down", true),
        ("left", true),
        ("right", true),
    ]
    .into_iter()
    .collect();

    // We've included code to prevent your Battlesnake from moving backwards
    let my_head = &you.body[0]; // Coordinates of your head
    let my_neck = &you.body[1]; // Coordinates of your "neck"

    if my_neck.x < my_head.x {
        // Neck is left of head, don't move left
        is_move_safe.insert("left", false);
    } else if my_neck.x > my_head.x {
        // Neck is right of head, don't move right
        is_move_safe.insert("right", false);
    } else if my_neck.y < my_head.y {
        // Neck is below head, don't move down
        is_move_safe.insert("down", false);
    } else if my_neck.y > my_head.y {
        // Neck is above head, don't move up
        is_move_safe.insert("up", false);
    }

    // TODO: Step 1 - Prevent your Battlesnake from moving out of bounds
    // let board_width = &board.width;
    // let board_height = &board.height;

    // TODO: Step 2 - Prevent your Battlesnake from colliding with itself
    // let my_body = &you.body;

    // TODO: Step 3 - Prevent your Battlesnake from colliding with other Battlesnakes
    // let opponents = &board.snakes;

    // Are there any safe moves left?
    let safe_moves = is_move_safe
        .into_iter()
        .filter(|&(_, v)| v)
        .map(|(k, _)| k)
        .collect::<Vec<_>>();

    // Choose a random move from the safe ones
    let chosen = safe_moves.choose(&mut rand::thread_rng()).unwrap();

    // TODO: Step 4 - Move towards food instead of random, to regain health and survive longer
    // let food = &board.food;

    info!("MOVE {}: {}", turn, chosen);
    return json!({ "move": chosen });
}

// This function finds which snake is closest to each point on the map through a flood fill
// fn flood_fill(_board: &Board) -> Vec<Vec<String>> {
//     // Skapar en klon för vet inte hur man slipper det på ett bra vis.
//     let mut snakes = _board.snakes.to_vec();
//     snakes.sort_by(|a, b| a.length.cmp(&b.length));
//     let mut queue = Vec::new();
//     for (i, s) in snakes.iter().enumerate() {
//         queue.push((i, s.head));
//     }
//     let mut visited = vec![vec![false; _board.height as usize]; _board.width as usize];
//     while let Some(current) = queue.pop() {
//         let (i, coord) = current;
//         if visited[coord.x as usize][coord.y as usize] {
//             continue;
//         }
//     }
// }

fn get_neighbors(x: i32, y: i32, height: i32, width: i32) -> Vec<(i32, i32)> {
    let mut neighbors = Vec::new();
    let directions = [(1, 0), (0, 1), (-1, 0), (0, -1)];
    for (dx, dy) in directions.iter() {
        let nx = x + dx;
        let ny = y + dy;
        if nx > 0 && nx < width && ny > 0 && ny < height {
            neighbors.push((x, y));
        }
    }
    neighbors
}

// sker sjukt många string.clone() i den här funktionen, hade varit nice att ta bort.
fn flood_fill(_board: &Board) -> Vec<Option<String>> {
    let mut snakes = Vec::new();
    for s in _board.snakes.iter() {
        snakes.push((s.id.clone(), s.head.x, s.head.y, s.length));
    }
    snakes.sort_by(|a, b| a.3.cmp(&b.3));
    let width = _board.width as i32;
    let height = _board.height as i32;
    let mut visited = vec![None; (height * width) as usize];
    let mut queue = Vec::new();
    for s in snakes.iter() {
        queue.push((s.0.clone(), s.1, s.2))
    }
    while let Some((id, x, y)) = queue.pop() {
        if visited[(y * height + x) as usize].is_some() {
            continue;
        }
        visited[(y * height + x) as usize] = Some(id.clone());
        for (nx, ny) in get_neighbors(x, y, height, width) {
            queue.push((id.clone(), nx, ny));
        }
    }
    visited
}
