// // Export all engine types at the top level
// mod types;
// pub use types::*;
// mod engine;
// pub use engine::Engine;

// pub mod render;
// pub mod input;
// pub mod image;

// mod util;

pub struct Player {
    pub name: String,
    pub is_cpu: bool,
    pub is_turn: bool,
    pub points: i32,
    pub inventory: Vec<String>,
    pub current_move: Option<Move>, 
}

pub enum Outcomes {
    Win,
    Lose,
    Draw,
}

//how can we make this lib.rs file take any type 
pub struct Move {
    pub move_type: <T>, 
    pub wins: <T>, 
    pub loses: <T>,
    // pub cost: u32,
}

// player1: Move{Rock, Scissor, Paper}
// enemy.current_move : Move{Paper, Rock, Scissor}

impl Move {

}

impl Player {
    pub fn new(name: String, is_cpu:bool, is_turn:bool) -> Player {
        Player{name: name, is_cpu: is_cpu, is_turn: is_turn, points:0, inventory: Vec<String>::new(), current_move: None}
    }
    
    fn finished_turn(&self) {
        self.is_turn = !self.is_turn;
    }
    fn execute_move(&self, enemy: Player) -> Outcomes {
        //double check current move
        if enemy.current_move.is_some() && self.current_move.is_some() {
            let enemy_move = enemy.current_move.unwrap();
            let our_move = &self.current_move.unwrap();

            //need to make a to string method for Move
            println!("You play: {}", our_move.to_string()); 

            
            if &our_move.wins == enemy_move.move_type {
                return Outcomes::Win;
            } else if &our_move.loses == enemy_move.move_type{
                return Outcomes::Lose;
            } else {
                return Outcomes::Draw;
            }
        
        }
    }
}



pub struct Room {
    pub name: String, // E.g. "Antechamber"
    pub desc: String, // E.g. "Dark wood paneling covers the walls.  The gilded northern doorway lies open."
    pub doors: Vec<Door>,
}
pub struct Door {
    pub target: RoomID,          // More about this in a minute
    pub triggers: Vec<String>,   // e.g. "go north", "north"
    pub message: Option<String>, // What message, if any, to print when the doorway is traversed
    // Any other info about the door would go here
    pub condition: Option<Item>,
}

#[derive(Debug, PartialEq)]
pub enum Item {
    Key,
    PaintingAdjuster,
}

impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Key => write!(f, "Mysterious Key"),
            Self::PaintingAdjuster => write!(f, "A pole which you can tell adjusts things."),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct RoomID(pub usize);
