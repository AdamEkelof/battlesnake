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
use std::{
    collections::{HashMap, VecDeque},
    hash::Hash,
};

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

fn get_neighbors(x: i32, y: i32, height: i32, width: i32) -> Vec<Coord> {
    let mut neighbors = Vec::new();
    let directions = [(1, 0), (0, 1), (-1, 0), (0, -1)];
    for (dx, dy) in directions.iter() {
        let nx = x + dx;
        let ny = y + dy;
        if 0 <= nx && nx < width && 0 <= ny && ny < height {
            neighbors.push(Coord { x: nx, y: ny });
        }
    }
    neighbors
}

impl Hash for Battlesnake {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Battlesnake {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Battlesnake {}

fn flood_fill(_board: &Board) -> HashMap<&Battlesnake, Vec<Coord>> {
    let mut mapping = HashMap::new();
    let mut queue: Vec<(&Battlesnake, Coord)> = _board
        .snakes
        .iter()
        .map(|x| (x, Coord { ..x.head }))
        .collect();
    queue.sort_by(|a, b| b.0.length.cmp(&a.0.length));
    let mut queue = VecDeque::from(queue);
    for snake in queue.iter() {
        mapping.insert(snake.0, Vec::new());
    }
    let h = _board.height as i32;
    let w = _board.width;
    let mut visited = vec![false; (h * w) as usize];
    while let Some(snake) = queue.pop_front() {
        let x = snake.1.x;
        let y = snake.1.y;
        if visited[(y * h + x) as usize] {
            continue;
        }
        // println!("New point in map {},{}", x, y);
        visited[(y * h + x) as usize] = true;
        mapping
            .get_mut(snake.0)
            .expect("What snake is this?")
            .push(Coord { x, y });
        for neighbor in get_neighbors(x, y, h, w) {
            queue.push_back((snake.0, neighbor));
        }
    }
    mapping
}

fn print_flood(mapping: &HashMap<&Battlesnake, Vec<Coord>>) {
    let mut v = [' '; 121];
    for (&key, val) in mapping {
        for pos in val.iter() {
            v[(pos.y * 11 + pos.x) as usize] = key.id.chars().next().unwrap();
        }
        v[(key.head.y * 11 + key.head.x) as usize] =
            key.id.chars().next().unwrap().to_ascii_lowercase();
    }
    for (i, n) in v.iter().enumerate() {
        print!("{n}");
        if (i + 1) % 11 == 0 {
            println!();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Battlesnake, Board, Coord};

    use super::{flood_fill, print_flood};

    impl Battlesnake {
        fn new(id: String, head: (i32, i32), length: i32) -> Self {
            Battlesnake {
                id,
                name: String::new(),
                health: 100,
                body: Vec::new(),
                head: Coord {
                    x: head.0,
                    y: head.1,
                },
                length,
                latency: String::new(),
                shout: None,
            }
        }
    }

    #[test]
    fn test_flood_fill() {
        let board = Board {
            height: 11,
            width: 11,
            food: vec![Coord { x: 1, y: 1 }],
            snakes: vec![
                Battlesnake::new(String::from("Adam"), (2, 2), 1),
                Battlesnake::new(String::from("Oliver"), (5, 6), 2),
            ],
            hazards: Vec::new(),
        };
        // a a a a a o o ...
        // a a a a a o o ...
        // a a a a o o o ...
        // a a a o o o o ...
        // ...
        let hm = flood_fill(&board);
        for (key, val) in &hm {
            println!("{}: {:?}", &key.id, val);
        }
        print_flood(&hm);
        let adam = hm
            .keys()
            .find(|s| s.id == String::from("Adam"))
            .expect("Adam does not exist");
        let oliver = hm
            .keys()
            .find(|s| s.id == String::from("Oliver"))
            .expect("Oliver does not exist");
        assert!(hm
            .get(adam)
            .unwrap()
            .iter()
            .find(|&x| x.x == 5 && x.y == 0)
            .is_some());
        assert!(hm
            .get(oliver)
            .unwrap()
            .iter()
            .find(|&x| x.x == 5 && x.y == 2)
            .is_some());
    }
}
