use engine::*;
use std::fmt::{self, Display, Formatter};

//we would have RPSTypes and Outcomes in the main of the rock paper scissors main.rs
pub enum RPSType {
    Rock,
    Paper,
    Scissors,
}

pub enum Outcomes {
    Win,
    Lose,
    Draw,
}


//need to create a list of what beats what 
//needs to create something that ends in an outcome

// struct Room {
//     name: String, // E.g. "Antechamber"
//     desc: String, // E.g. "Dark wood paneling covers the walls.  The gilded northern doorway lies open."
//     doors: Vec<Door>,
// }
// struct Door {
//     target: RoomID,          // More about this in a minute
//     triggers: Vec<String>,   // e.g. "go north", "north"
//     message: Option<String>, // What message, if any, to print when the doorway is traversed
//     // Any other info about the door would go here
//     condition: Option<Item>,
// }

// #[derive(Debug, PartialEq)]
// enum Item {
//     Key,
//     PaintingAdjuster,
// }

// impl Display for Item {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         match self {
//             Self::Key => write!(f, "Mysterious Key"),
//             Self::PaintingAdjuster => write!(f, "A pole which you can tell adjusts things."),
//         }
//     }
// }

// #[derive(PartialEq, Eq, Clone, Copy)]
// struct RoomID(usize);

fn main() {
    use std::io;
    // We need the Write trait so we can flush stdout
    use std::io::Write;

    let moves = [
        Move {
            move_type: Rock,
            wins: Scissors,
            loses: Paper,
        },
        Move {
            move_type: Scissors,
            wins: Paper,
            loses: Rock,
        },
        Move {
            move_type: Paper,
            wins: Rock,
            loses: Scissors,
        },
    ];

    let p1 = Play

    // let rooms = [
    //     Room {
    //         name: "Foyer".into(), // Turn a &'static string (string constant) into a String
    //         desc: "This beautifully decorated foyer beckons you further into the mansion.  There is a door to the north. \nThere is a key in the room.".into(),
    //         doors: vec![Door{target:RoomID(1), triggers:vec!["door".into(), "north".into(), "go north".into()], message:None, condition: Some(Item::Key)}]
    //     },
    //     Room {
    //         name: "Antechamber".into(),
    //         desc: "Dark wood paneling covers the walls.  An intricate painting of a field mouse hangs slightly askew on the wall (it looks like you could fix it). The gilded northern doorway lies open to a shadowy parlour.  You can return to the foyer to the southern door.\nThere is a painting adjuster in the room you can take.".into(),
    //         doors: vec![
    //             Door{target:RoomID(0), triggers:vec!["door".into(), "south".into(), "go south".into(), "foyer".into()], message:None, condition: None},
    //             Door{target:RoomID(2), triggers:vec!["north".into(), "doorway".into(), "go north".into()], message:None, condition: None},
    //             Door{target:RoomID(3), triggers:vec!["painting".into(), "mouse".into(), "fix painting".into()], message:Some("As you adjust the painting, a trap-door opens beneath your feet!".into()), condition: Some(Item::PaintingAdjuster)}
    //         ]
    //     },
    //     Room {
    //         name: "A Room Full of Snakes!".into(),
    //         desc: "The shadows wriggle and shift as you enter the parlour.  The floor is covered in snakes!  The walls are covered in snakes!  The ceiling is covered in snakes!  You are also covered in snakes!\n\nBAD END".into(),
    //         doors:vec![]
    //     },
    //     Room {
    //         name: "The Vault".into(),
    //         desc: "When you regain consciousness, you feel a stabbing sensation in your lower back.  Reaching beneath you, you discover a massive diamond!  This room is full of gold and jewels, and a convenient ladder leading back outdoors!\n\nYou win!".into(),
    //         doors:vec![]
    //     }
    // ];
    // let end_rooms = [RoomID(2), RoomID(3)];
    // let mut inventory: Vec<Item> = Vec::new();

    // let mut room_items: Vec<Vec<Item>> = Vec::new();
    // room_items.push(vec![Item::Key]);
    // room_items.push(vec![Item::PaintingAdjuster]);
    // room_items.push(vec![]);
    // room_items.push(vec![]);

    // let mut input = String::new();

    // let mut at = RoomID(0);
    // println!("The Spooky Mansion Adventure");
    // println!("============================");
    // println!();
    // println!("You've been walking for hours in the countryside, and have finally stumbled on the spooky mansion you read about in the tour guide.");
    // loop {
    //     // We don't want to move out of rooms, so we take a reference
    //     let here = &rooms[at.0];
    //     println!("{}\n{}", here.name, here.desc,);
    //     if end_rooms.contains(&at) {
    //         break;
    //     }
    //     loop {
    //         print!("Inventory: ");
    //         let mut in_string = String::new();
    //         for item in inventory.iter() {
    //             in_string.push_str(&item.to_string());
    //         }
    //         print!("{}\n", in_string);
    //         println!("Enter 'get item' to take an item from the room, if available");
    //         io::stdout().flush().unwrap();
    //         input.clear();
    //         io::stdin().read_line(&mut input).unwrap();
    //         let input = input.trim();
    //         if "get item" == input {
    //             let item = room_items[at.0].pop();
    //             if let None = item {
    //                 print!("No item available to take");
    //             } else {
    //                 inventory.push(item.unwrap());
    //             }
    //         } else if let Some(door) = here
    //             .doors
    //             .iter()
    //             .find(|d| d.triggers.iter().any(|t| *t == input))
    //         {
    //             if let Some(item) = &door.condition {
    //                 if inventory.contains(&item) {
    //                     if let Some(msg) = &door.message {
    //                         println!("{}", msg);
    //                     }
    //                     at = door.target;
    //                     break;
    //                 } else {
    //                     print!("You don't have the item required to do that.\n");
    //                 }
    //             } else {
    //                 if let Some(msg) = &door.message {
    //                     println!("{}", msg);
    //                 }
    //                 at = door.target;
    //                 break;
    //             }
    //         } else {
    //             println!("You can't do that!");
    //         }
    //     }
    // }
}
