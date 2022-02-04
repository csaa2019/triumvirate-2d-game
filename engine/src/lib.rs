// // Export all engine types at the top level
// mod types;
// pub use types::*;
// mod engine;
// pub use engine::Engine;

// pub mod render;
// pub mod input;
// pub mod image;

// mod util;

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