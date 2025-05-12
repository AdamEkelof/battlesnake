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

mod simple;
mod mm_search;

use log::info;
use rand::seq::SliceRandom;
use serde_json::{json, Value};
use std::{
    collections::{HashMap, VecDeque},
    hash::{Hash, Hasher},
};

use crate::{Battlesnake, Board, Coord, Game, GameInfo};

use mm_search::search;
use simple::{SimpleBoard, SimpleSnake};

// info is called when you create your Battlesnake on play.battlesnake.com
// and controls your Battlesnake's appearance
// TIP: If you open your Battlesnake URL in a browser you should see this data
pub fn info() -> Value {
    info!("INFO");

    return json!({
        "apiversion": "1",
        "author": "Group 18", // TODO: Your Battlesnake Username
        "color": "#e83d84", // TODO: Choose color
        "head": "tiger-king", // TODO: Choose head
        "tail": "coffee", // TODO: Choose tail
    });
}

// start is called when your Battlesnake begins a game
pub fn start(_game: &Game, _turn: &i32, _board: &Board, _you: &Battlesnake) {
    // create team mate pairs
    // store timeout
    info!("GAME START");
}

// end is called when your Battlesnake finishes a game
pub fn end(_game: &Game, _turn: &i32, _board: &Board, _you: &Battlesnake) {
    info!("GAME OVER");
}

// move is called on every turn and returns your next move
// Valid moves are "up", "down", "left", or "right"
// See https://docs.battlesnake.com/api/example-move for available data
pub fn get_move(
    _game: &Game,
    turn: &i32,
    _board: &Board,
    you: &Battlesnake,
    game_info: &mut GameInfo,
) -> Value {
    let my_id = you.id.clone();
    let team_idx = game_info
        .agent_ids
        .iter()
        .position(|x| x == &my_id)
        .expect("Agent ID not found");
    let simple_snakes: Vec<SimpleSnake> = _board.snakes.iter().map(|snake| SimpleSnake::from(snake, game_info.agent_ids.contains(&snake.id))).collect();
    let simple_board = SimpleBoard::from(_board, simple_snakes);
    
    if game_info.agent_moves[team_idx].len() == *turn as usize + 1 {
        return json!({ "move": game_info.agent_moves[team_idx][*turn as usize] });
    }

    let mut temp_ids: [String; 2] = ["".to_string(), "".to_string()];
    let enemy_ids: Vec<String> = game_info
        .agent_ids
        .iter()
        .filter(|&x| !game_info.agent_ids.contains(&x))
        .cloned()
        .collect();
    for (i, id) in enemy_ids.iter().enumerate() {
        temp_ids[i] = id.clone();
    }

    let moves = search(_board, &game_info.agent_ids, &temp_ids, game_info.timeout);
    for i in 0..2 {
        game_info.agent_moves[i].push(moves[i].clone());
    }


    let chosen = moves[team_idx].clone();
    info!("MOVE {}: {}", turn, chosen);
    // store down for team mate
    return json!({ "move": chosen });
}

pub fn get_safe_moves<'a>(board: &'a Board, you: &'a Battlesnake) -> Vec<&'a str> {
    let mut is_move_safe: HashMap<_, _> = vec![
        ("up", true),
        ("down", true),
        ("left", true),
        ("right", true),
    ]
    .into_iter()
    .collect();

    let my_head = &you.body[0];
    let my_neck = &you.body[1];

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

    for m in is_move_safe.clone().keys() {
        if !is_move_safe[m] {
            continue;
        }
        is_move_safe.insert(
            m,
            !out_of_bounds(my_head, board, m)
                && !collision_with_body(my_head, &you.body[2..you.body.len() - 1], m)
                && !collision_with_snakes(my_head, board, you.id.clone(), m),
        );
    }

    is_move_safe
        .iter()
        .filter(|(_, &v)| v)
        .map(|(k, _)| *k)
        .collect()
}

fn new_position(position: &Coord, m: &str) -> Coord {
    let mut new_position = *position;
    if m == "" {
        // No move
    } else if m == "up" {
        new_position.y += 1;
    } else if m == "down" {
        new_position.y -= 1;
    } else if m == "left" {
        new_position.x -= 1;
    } else if m == "right" {
        new_position.x += 1;
    }
    new_position
}

fn out_of_bounds(poistion: &Coord, board: &Board, m: &str) -> bool {
    if m == "up" {
        return poistion.y == board.height as i32 - 1;
    } else if m == "down" {
        return poistion.y == 0;
    } else if m == "left" {
        return poistion.x == 0;
    } else if m == "right" {
        return poistion.x == board.width - 1;
    }
    return false;
}

fn collision_with_body(position: &Coord, body: &[Coord], m: &str) -> bool {
    let next_position: Coord = new_position(position, m);

    //info!("Checking collision {} ({}) with body: {:?}", position, m, body);

    for part in body.iter() {
        if part.x == next_position.x && part.y == next_position.y {
            return true;
        }
    }
    return false;
}

fn collision_with_snakes(position: &Coord, board: &Board, id: String, m: &str) -> bool {
    let next_position: Coord = new_position(position, m);

    for snake in board.snakes.iter() {
        if snake.id == id {
            continue;
        }
        if collision_with_body(&next_position, &snake.body[1..snake.body.len() - 1], "") {
            return true;
        }
    }
    return false;
}

impl Coord {
    fn manhattan(&self, other: &Self) -> u32 {
        ((self.x - other.x).abs() + (self.y - other.y).abs()) as u32
    }
}

fn select_played_out<'a>(
    snake: &Battlesnake,
    board: &'a Board,
    depth: u32,
) -> Vec<&'a Battlesnake> {
    let mut p_played = Vec::new();
    for other_snake in board.snakes.iter() {
        if snake == other_snake {
            continue;
        }
        if snake.head.manhattan(&other_snake.head) <= 2 * depth {
            p_played.push(other_snake);
        } else {
            for part in other_snake.body.iter() {
                if snake.head.manhattan(part) <= 2 * depth {
                    p_played.push(other_snake);
                    break;
                }
            }
        }
    }
    p_played
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

impl Hash for Board {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for food in self.food.iter() {
            food.hash(state);
        }
        for snake in self.snakes.iter() {
            snake.hash(state);
        }
    }
}

impl Hash for Coord {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.hash(state);
        self.y.hash(state);
    }
}

impl Hash for Battlesnake {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        for part in self.body.iter() {
            part.hash(state);
        }
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
