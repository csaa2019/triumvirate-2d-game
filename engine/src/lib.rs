// // Export all engine types at the top level
pub mod animation;
pub mod image;

use crate::image::Image;
use std::rc::Rc;

// mod types;
// pub use types::*;
// mod engine;
// pub use engine::Engine;

// pub mod render;
// pub mod input;
// mod util;

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub struct Vec2i {
    // Or Vec2f for floats?
    pub x: i32,
    pub y: i32,
}

impl Vec2i {
    pub fn new(x: i32, y: i32) -> Vec2i {
        Vec2i { x, y }
    }
    // Maybe add functions for e.g. the midpoint of two vecs, or...
}

pub struct Player<T: Copy + Eq + PartialEq> {
    pub name: String,
    pub is_cpu: bool,
    pub is_turn: bool,
    pub points: i32,
    pub inventory: Vec<String>,
    pub current_move: Option<Move<T>>,
}

impl<T: Copy + Eq + PartialEq> Player<T> {
    pub fn new(name: String, is_cpu: bool, is_turn: bool) -> Player<T> {
        Player {
            name: name,
            is_cpu: is_cpu,
            is_turn: is_turn,
            points: 0,
            inventory: Vec::<String>::new(),
            current_move: None,
        }
    }
    pub fn set_current_move(&mut self, chosen_move: Move<T>) {
        self.current_move = Some(chosen_move);
    }

    pub fn finished_turn(&mut self) {
        self.is_turn = !self.is_turn;
    }
    pub fn execute_move(&mut self, enemy: &Player<T>) -> Outcomes {
        //double check current move
        if enemy.current_move.is_some() && self.current_move.is_some() {
            let enemy_move = enemy.current_move.as_ref().unwrap();
            let our_move = &self.current_move.as_ref().unwrap();

            //need to make a to string method for Move
            // println!("You play: {}", our_move.to_string());

            if our_move.wins == enemy_move.move_type {
                return Outcomes::Win;
            } else if our_move.loses == enemy_move.move_type {
                return Outcomes::Lose;
            } else {
                return Outcomes::Draw;
            }
        } else {
            // What to return if invalid moves?
            return Outcomes::Draw;
        }
    }
}

pub struct Fighter<T: Copy + Eq + PartialEq> {
    pub name: String,
    pub is_cpu: bool,
    pub is_turn: bool,
    pub health: i32,
    pub mana: i32,
    pub move_inventory: Vec<FighterMove>,
    pub current_move: Option<FighterMove<T>>,
}

impl<T: Copy + Eq + PartialEq> Fighter<T> {
    pub fn new(
        name: String,
        is_cpu: bool,
        is_turn: bool,
        move_inventory: Vec<FighterMove>,
    ) -> Fighter<T> {
        Fighter {
            name: name,
            is_cpu: is_cpu,
            is_turn: is_turn,
            health: 100,
            mana: 100,
            move_inventory,
            current_move: None,
        }
    }
    pub fn set_current_move(&mut self, chosen_move: Move<T>) {
        self.current_move = Some(chosen_move);
    }

    pub fn finished_turn(&mut self) {
        self.is_turn = !self.is_turn;
    }

    //need to modify this execute_move function
    //do we still want this to return an outcome?
    pub fn execute_move(&mut self, enemy: &Fighter<T>) {
        //double check current move
        if enemy.current_move.is_some() && self.current_move.is_some() {
            let enemy_fighter_move = enemy.current_move.as_ref().unwrap();
            let self_fighter_move = &self.current_move.as_ref().unwrap();

            self.health += enemy_fighter_move.damage;
            enemy.mana += enemy_fighter_move.mana_cost;
            enemy.health += enemy_fighter_move.health_cost;

            enemy.health += self_fighter_move.damage;
            self.mana += self_fighter_move.mana_cost;
            self.health += self_fighter_move.health_cost;
        } else {
            //do nothing
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum Outcomes {
    Win,
    Lose,
    Draw,
}

//how can we make this lib.rs file take any type
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub struct Move<T: Copy + Eq + PartialEq> {
    pub move_type: T,
    pub wins: T,
    pub loses: T,
    // pub cost: u32,
}
pub struct FighterMove<T: Copy + Eq + PartialEq> {
    pub name: String,
    //damage to the other player, positive value
    pub damage: i32,
    //cost of player's mana, negative value
    pub mana_cost: i32,
    //cost of "health" positive value means it adds to players health (regenerative moves)
    //negative health would be just the case that a move takes away from player health
    pub health_cost: i32,
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum GameStates {
    //before the game begins: Rock Paper Scissor Game, by Chloe, Nate, and Grace with a play button
    MainScreen,

    //instructions for the game + start button + where to add Player name
    Instructions,

    //Pick your cards, go button
    PlayerPicking,

    //Screen with which round number (of 3) and countdown
    Countdown,

    //Screen that shows the CPU and your pick for that round
    ShowPick,

    //Screen that shows who wins
    FinalScreen,

    //gameStates unique to game-2

    //sort of like playerpicking, choose nate, grace, or chloe
    ChooseFighter,

    NateInfo,
    ChloeInfo,
    GraceInfo,

    //choose which move from each player
    ChooseMove,
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub struct Game {
    pub state: GameStates,
}

impl Game {
    pub fn new(state: GameStates) {
        Game { state };
    }
}
