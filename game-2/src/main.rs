use engine::image::Vec2i;
use engine::*;
use kira::arrangement::{Arrangement, LoopArrangementSettings};
use kira::instance::InstanceSettings;
use kira::manager::{AudioManager, AudioManagerSettings};
use kira::sound::SoundSettings;
use rand;
use rand::Rng;
use std::rc::Rc;
use vulkano::buffer::TypedBufferAccess;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents};
use vulkano::pipeline::{Pipeline, PipelineBindPoint};
use vulkano::swapchain::{self, AcquireError, SwapchainCreationError};
use vulkano::sync::{self, FlushError, GpuFuture};
use winit::event::{Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

const WIDTH: usize = 320;
const HEIGHT: usize = 240;

fn main() {
    /*
    Stuff during initialization (once)
    Stuff after window resize
    Stuff during each event loop

    Stuff each call/draw operation
    */

    //Load audio
    let mut audio_manager = AudioManager::new(AudioManagerSettings::default()).unwrap();
    // let mut sound_handle_music = audio_manager
    //     .load_sound("content/RPS_tunes_loop.ogg", SoundSettings::default())
    //     .unwrap();
    // let mut arrangement_handle = audio_manager
    //     .add_arrangement(Arrangement::new_loop(
    //         &sound_handle_music,
    //         LoopArrangementSettings::default(),
    //     ))
    //     .unwrap();
    let mut sound_handle_music = audio_manager
        .load_sound("content/game2_tunes.ogg", SoundSettings::default())
        .unwrap();

    let mut arrangement_handle = audio_manager
        .add_arrangement(Arrangement::new_loop(
            &sound_handle_music,
            LoopArrangementSettings::default(),
        ))
        .unwrap();

    let mut coin_handle_click = audio_manager
        .load_sound("content/button-noise.ogg", SoundSettings::default())
        .unwrap();

    let mut click_handle_click = audio_manager
        .load_sound("content/click-noise.ogg", SoundSettings::default())
        .unwrap();

    let event_loop = EventLoop::new();
    let (vulkan_config, mut vulkan_state) = engine::vulkan_init(&event_loop);

    let mut playing_anim = false;

    //Image stuff
    let (mut titlefont, mut font) = engine::animation::get_fonts();

    let fontsize = font.font_width;
    let fontsize_h = font.font_height;
    let titlefont_size = titlefont.font_width;
    let titlefont_size_height = titlefont.font_height;

    let description_box_dim = Vec2i {
        x: 200,
        y: HEIGHT as i32 - (HEIGHT as i32 / 3),
    };
    let description_draw_to = Vec2i {
        x: ((WIDTH as i32) / 2) - (description_box_dim.x / 2),
        y: HEIGHT as i32 / 3,
    };

    let title_box_dim = Vec2i {
        x: 200,
        y: titlefont_size_height as i32 * 3,
    };
    let title_draw_to = Vec2i {
        x: ((WIDTH as i32) / 2) - (title_box_dim.x / 2),
        y: 10,
    };

    //GameState::ChooseFighter

    pub fn center_w(image_width: u32) -> u32 {
        let empty_space = WIDTH as u32 - image_width;
        return empty_space / 2;
    }

    pub fn center_h(image_height: u32) -> u32 {
        let empty_space = HEIGHT as u32 - image_height;
        return empty_space / 2;
    }

    let main_screen_w = 260;
    let main_screen_h = 180;
    let main_screen_rect = engine::image::Rect::new(0, 0, main_screen_w, main_screen_h);
    let main_screen = engine::image::Image::from_png_not_premultiplied(
        "content/main-screen-final.png",
        main_screen_w,
        main_screen_h,
    );
    let main_screen_draw_to = engine::image::Vec2i {
        x: center_w(main_screen_w) as i32,
        y: center_h(main_screen_h) as i32,
    };

    let header_rect_w = 80 as u32;
    let header_rect_h = 10 as u32;
    let header_rect = engine::image::Rect::new(0, 0, header_rect_w, header_rect_h);
    let header_player1 =
        engine::image::Image::from_png("content/player1-text.png", header_rect_w, header_rect_h);

    let header_player2 =
        engine::image::Image::from_png("content/player2-text.png", header_rect_w, header_rect_h);

    let wins_text =
        engine::image::Image::from_png("content/wins-text.png", header_rect_w, header_rect_h);

    let player_header_rect_draw_to = engine::image::Vec2i { x: 120, y: 12 };

    let header_choose_move_rect_w = 120 as u32;
    let header_choose_move_rect_h = 10 as u32;
    let header_choose_move_rect =
        engine::image::Rect::new(0, 0, header_choose_move_rect_w, header_choose_move_rect_h);
    let header_player1_choose_move_rect = engine::image::Image::from_png(
        "content/player1-choose-move-text.png",
        header_choose_move_rect_w,
        header_choose_move_rect_h,
    );

    let header_player2_choose_move_rect = engine::image::Image::from_png(
        "content/player2-choose-move-text.png",
        header_choose_move_rect_w,
        header_choose_move_rect_h,
    );
    let player_choose_move_draw_to = engine::image::Vec2i {
        x: center_w(header_choose_move_rect_w) as i32,
        y: 12,
    };

    let highlight_rect_w = 96 as u32;
    let highlight_rect_h = 130 as u32;
    let highlight_rect_rect = engine::image::Rect::new(0, 0, highlight_rect_w, highlight_rect_h);
    let highlight_rect = engine::image::Image::from_png_not_premultiplied(
        "content/pinkhighlight.png",
        highlight_rect_w,
        highlight_rect_h,
    );

    let highlight_rect_draw_to_1 = engine::image::Vec2i { x: 10, y: 30 };
    let highlight_rect_draw_to_2 = engine::image::Vec2i { x: 110, y: 30 };
    let highlight_rect_draw_to_3 = engine::image::Vec2i { x: 210, y: 30 };

    //temporary placeholder for our fighter images
    let fighter_rect_w = 86 as u32;
    let fighter_rect_h = 120 as u32;
    let fighter_rect_rect = engine::image::Rect::new(0, 0, fighter_rect_w, fighter_rect_h);
    let fighter_rect = engine::image::Image::from_png_not_premultiplied(
        "content/playerimagerect.png",
        fighter_rect_w,
        fighter_rect_h,
    );

    let nate_fighter_rect = engine::image::Image::from_png_not_premultiplied(
        "content/natefighter.png",
        fighter_rect_w,
        fighter_rect_h,
    );

    let chloe_fighter_rect = engine::image::Image::from_png_not_premultiplied(
        "content/chloefighter.png",
        fighter_rect_w,
        fighter_rect_h,
    );

    let grace_fighter_rect = engine::image::Image::from_png_not_premultiplied(
        "content/gracefighter.png",
        fighter_rect_w,
        fighter_rect_h,
    );

    let move_chill_pill = engine::image::Image::from_png_not_premultiplied(
        "content/move-chill-pill.png",
        fighter_rect_w,
        fighter_rect_h,
    );

    let move_guitar = engine::image::Image::from_png_not_premultiplied(
        "content/move-guitar.png",
        fighter_rect_w,
        fighter_rect_h,
    );

    let move_yoyo = engine::image::Image::from_png_not_premultiplied(
        "content/move-yoyo.png",
        fighter_rect_w,
        fighter_rect_h,
    );

    let move_hacker = engine::image::Image::from_png_not_premultiplied(
        "content/move-hacker.png",
        fighter_rect_w,
        fighter_rect_h,
    );

    let move_metaphysical_inquiry = engine::image::Image::from_png_not_premultiplied(
        "content/move-metaphysical-inquiry.png",
        fighter_rect_w,
        fighter_rect_h,
    );

    let move_curb_stomp = engine::image::Image::from_png_not_premultiplied(
        "content/move-curb-stomp.png",
        fighter_rect_w,
        fighter_rect_h,
    );

    let move_grilled_cheese = engine::image::Image::from_png_not_premultiplied(
        "content/move-grilled-cheese.png",
        fighter_rect_w,
        fighter_rect_h,
    );

    let move_prank = engine::image::Image::from_png_not_premultiplied(
        "content/move-prank.png",
        fighter_rect_w,
        fighter_rect_h,
    );

    let move_mystery_box = engine::image::Image::from_png_not_premultiplied(
        "content/move-mystery-box.png",
        fighter_rect_w,
        fighter_rect_h,
    );

    //a draw to for each chloe, nate, grace
    //for ease lets have 1 as chloe, 2 as nate, 3 as grace
    let fighter_rect_draw_to_1 = engine::image::Vec2i { x: 15, y: 35 };
    let fighter_rect_draw_to_2 = engine::image::Vec2i { x: 115, y: 35 };
    let fighter_rect_draw_to_3 = engine::image::Vec2i { x: 215, y: 35 };

    // modified this:
    // let p1_draw_to = engine::image::Vec2i { x: 215, y: 10 };
    // let p2_draw_to = engine::image::Vec2i { x: 15, y: HEIGHT as i32 - (fighter_rect_h as i32 + 10)};
    let p1_draw_to = engine::image::Vec2i {
        x: 15,
        y: HEIGHT as i32 - (fighter_rect_h as i32 + 10),
    };
    let p2_draw_to = engine::image::Vec2i { x: 215, y: 10 };

    let fighter_rect_clickable_rect_1 = engine::image::Rect::new(
        fighter_rect_draw_to_1.x,
        fighter_rect_draw_to_1.y,
        fighter_rect_w,
        fighter_rect_h,
    );

    let fighter_rect_clickable_rect_2 = engine::image::Rect::new(
        fighter_rect_draw_to_2.x,
        fighter_rect_draw_to_2.y,
        fighter_rect_w,
        fighter_rect_h,
    );

    let fighter_rect_clickable_rect_3 = engine::image::Rect::new(
        fighter_rect_draw_to_3.x,
        fighter_rect_draw_to_3.y,
        fighter_rect_w,
        fighter_rect_h,
    );

    //temporary placeholder for our "read info" images
    //maybe a better variable name for this would be "small button" but I'll change this later
    let fighter_info_w = 86 as u32;
    let fighter_info_h = 24 as u32;
    let fighter_info_rect = engine::image::Rect::new(0, 0, fighter_info_w, fighter_info_h);
    let fighter_info = engine::image::Image::from_png_not_premultiplied(
        "content/read-info-button.png",
        fighter_info_w,
        fighter_info_h,
    );

    let select_button = engine::image::Image::from_png_not_premultiplied(
        "content/select-button.png",
        fighter_info_w,
        fighter_info_h,
    );

    let play_again_button = engine::image::Image::from_png_not_premultiplied(
        "content/button-play-again.png",
        fighter_info_w,
        fighter_info_h,
    );

    let play_button = engine::image::Image::from_png_not_premultiplied(
        "content/button-play.png",
        fighter_info_w,
        fighter_info_h,
    );

    //a draw to for each chloe, nate, grace
    //for ease lets have 1 as chloe, 2 as nate, 3 as grace
    let fighter_info_draw_to_1 = engine::image::Vec2i { x: 15, y: 170 };
    let fighter_info_draw_to_2 = engine::image::Vec2i { x: 115, y: 170 };
    let fighter_info_draw_to_3 = engine::image::Vec2i { x: 215, y: 170 };

    let move_info_draw_to_1 = engine::image::Vec2i { x: 15, y: 163 };
    let move_info_draw_to_2 = engine::image::Vec2i { x: 115, y: 163 };
    let move_info_draw_to_3 = engine::image::Vec2i { x: 215, y: 163 };

    let next_button_draw_to = engine::image::Vec2i { x: 115, y: 205 };
    //won't actually need clickable rect for them
    let fighter_info_clickable_rect_1 = engine::image::Rect::new(
        fighter_info_draw_to_1.x,
        fighter_info_draw_to_1.y,
        fighter_info_w,
        fighter_info_h,
    );

    let fighter_info_clickable_rect_2 = engine::image::Rect::new(
        fighter_info_draw_to_2.x,
        fighter_info_draw_to_2.y,
        fighter_info_w,
        fighter_info_h,
    );

    let fighter_info_clickable_rect_3 = engine::image::Rect::new(
        fighter_info_draw_to_3.x,
        fighter_info_draw_to_3.y,
        fighter_info_w,
        fighter_info_h,
    );

    let move_info_clickable_rect_1 = engine::image::Rect::new(
        move_info_draw_to_1.x,
        move_info_draw_to_1.y,
        fighter_info_w,
        fighter_info_h,
    );

    let move_info_clickable_rect_2 = engine::image::Rect::new(
        move_info_draw_to_2.x,
        move_info_draw_to_2.y,
        fighter_info_w,
        fighter_info_h,
    );

    let move_info_clickable_rect_3 = engine::image::Rect::new(
        move_info_draw_to_3.x,
        move_info_draw_to_3.y,
        fighter_info_w,
        fighter_info_h,
    );

    //better variable name for this would be "select" button on the ChooseFighter gamestate
    let next_button_clickable_rect = engine::image::Rect::new(
        next_button_draw_to.x,
        next_button_draw_to.y,
        fighter_info_w,
        fighter_info_h,
    );

    // KEYBOARD INPUT STUFF
    let mut now_keys = [false; 255];
    let mut prev_keys = now_keys.clone();

    // MOUSE INPUT STUFF
    let mut mouse_x = 0.0;
    let mut mouse_y = 0.0;

    let mut mouse_click = false;
    let mut prev_mouse_click = false;

    #[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
    pub enum FighterType {
        None,
        Both,
        Chloe,
        Grace,
        Nate,
    }

    #[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
    pub enum FighterMoveType {
        None,

        //because rust is screaming at me and I can't currently be assed to make more efficient code
        //eventually we will want to delete the healing, damage, heavydamage part
        Healing,
        Damage,
        HeavyDamage,

        NateMove1,
        NateMove2,
        NateMove3,
        ChloeMove1,
        ChloeMove2,
        ChloeMove3,
        GraceMove1,
        GraceMove2,
        GraceMove3,
    }

    let grace_strings = FighterStrings {
        name: "Grace",
        desc: "Watch out for this one.",
        move1_name: "Eat Grilled Cheese",
        move1_desc: "Regenerate 15 health and 30 mana... but beware the grilled cheese bloat.",
        move2_name: "Get Pranked",
        move2_desc: "There's a camera right there.",
        move3_name: "Mystery Box",
        move3_desc: "What might this move do? I don't know. Why don't you find out?",
    };
    //random move names here
    let grace_fighter_moves = vec![
        FighterMove {
            //  eat a grilled cheese - but beware the health damage
            fighter_move_type: FighterMoveType::GraceMove1,
            damage: 0,
            mana_cost: 0,
            health_cost: 10,
            mana_generation: 30,
        },
        // get pranked -- camera RIGHT there
        FighterMove {
            fighter_move_type: FighterMoveType::GraceMove2,
            damage: -30,
            mana_cost: -30,
            health_cost: 0,
            mana_generation: 0,
        },
        // mystery box move -- so much mana
        // it's a terrible mystery box damage wise but health goes up
        FighterMove {
            fighter_move_type: FighterMoveType::GraceMove3,
            damage: -10,
            mana_cost: -40,
            health_cost: 40,
            mana_generation: 0,
        },
    ];

    let chloe_strings = FighterStrings{
        name: "Chloe",
        desc: "Chloe is a junior attending Pomona College majoring in Computer Science and Philosophy.",
        move1_name: "Metaphysical Inquiry",
        move1_desc: "The secrets of the universe are burdensome to bear.",
        move2_name: "Curb Stomp",
        move2_desc: "Did I mention I'm from New York?",
        move3_name: "Hack the Simulation",
        move3_desc: "Chloe just hacked the simluation and deleted yo ass -- fatal",
    };

    let chloe_fighter_moves = vec![
        FighterMove {
            //metaphysical inquiry
            fighter_move_type: FighterMoveType::ChloeMove1,
            damage: 0,
            mana_cost: 0,
            health_cost: -5,
            mana_generation: 15,
        },
        FighterMove {
            // curb stomp -
            fighter_move_type: FighterMoveType::ChloeMove2,
            damage: -10,
            mana_cost: -10,
            health_cost: 0,
            mana_generation: 0,
        },
        FighterMove {
            // chloe just hacked the simulation and deleted yo ass -- fatal
            fighter_move_type: FighterMoveType::ChloeMove3,
            damage: -100,
            mana_cost: -69,
            health_cost: 0,
            mana_generation: 20,
        },
    ];

    let nate_strings = FighterStrings {
        name: "Nate",
        desc: "The dopest guy around.",
        move1_name: "Take a Chill Pill",
        move1_desc: "Feel the chillness coursing through your bloodstream.",
        move2_name: "Face Melting Guitar Solo",
        move2_desc: "Just like a microwave.",
        move3_name: "Toss Some Yo",
        move3_desc: "Nate will flex some badass tricks and then... BAM!",
    };

    let nate_fighter_moves = vec![
        FighterMove {
            // take a chill pill -- feel the chillness coursing through your bloodstream
            fighter_move_type: FighterMoveType::NateMove1,
            damage: 0,
            mana_cost: 0,
            health_cost: 0,
            mana_generation: 20,
        },
        FighterMove {
            // Face melting guitar solo -- just like a microwave
            fighter_move_type: FighterMoveType::NateMove2,
            damage: -20,
            mana_cost: -20,
            health_cost: 0,
            mana_generation: 0,
        },
        FighterMove {
            // toss some yo -- flex some badass tricks and then BAM!
            fighter_move_type: FighterMoveType::NateMove3,
            damage: -55,
            mana_cost: -40,
            health_cost: 0,
            mana_generation: 0,
        },
    ];

    let placeholder_fightermove = FighterMove {
        fighter_move_type: FighterMoveType::None,
        damage: -69,
        mana_cost: -69,
        health_cost: 69,
        mana_generation: 0,
    };

    // this is a stand in variable to test hp bar animation
    let hp_y = 20;
    let hp_draw_to = engine::image::Vec2i {
        x: (WIDTH as i32) - 100 - 10,
        y: 10,
    };
    let hp_color = (255, 0, 0, 1);

    let mana_y = 20;
    let mana_draw_to = engine::image::Vec2i {
        x: (WIDTH as i32) - 100 - 10,
        y: 40,
    };
    let mana_color = (0, 0, 255, 1);

    let pick_fps = 10;
    let mut pick_frame_count = 0;
    let mut pick_anim_playing = false;
    let mut pick_anim_done = false;

    /*
    let mut chloe = Fighter::new(FighterType::Chloe, false, true);
    let mut grace = Fighter::new(FighterType::Grace, false, true);
    let mut nate = Fighter::new(FighterType::Nate, false, true);
    */

    let back_button_w = 271 / 6;
    let back_button_h = 91 / 6;
    let mut back_button =
        engine::image::Image::from_png("content/backbutton.png", back_button_w, back_button_h);

    let mut back_button_rect = engine::image::Rect::new(0, 0, back_button_w, back_button_h);

    let back_button_to = Vec2i { x: 10, y: 10 };
    let back_button_clickable_rect = engine::image::Rect::new(
        back_button_to.x,
        back_button_to.y,
        back_button_w,
        back_button_h,
    );
    // GAME STUFF
    let mut game = Game {
        state: GameStates::MainScreen,
    };

    pub struct GameInfo {
        pub current_player1: FighterType,
        pub player1_info: FighterType,
        pub player1_current_move: FighterMove<FighterMoveType>,
        pub player1_move_info: FighterMove<FighterMoveType>,
        pub current_player2: FighterType,
        pub player2_info: FighterType,
        pub player2_current_move: FighterMove<FighterMoveType>,
        pub player2_move_info: FighterMove<FighterMoveType>,
        pub winning_player: FighterType,
    }

    impl GameInfo {
        pub fn new(
            current_player1: FighterType,
            player1_info: FighterType,
            player1_current_move: FighterMove<FighterMoveType>,
            player1_move_info: FighterMove<FighterMoveType>,
            current_player2: FighterType,
            player2_info: FighterType,
            player2_current_move: FighterMove<FighterMoveType>,
            player2_move_info: FighterMove<FighterMoveType>,
            winning_player: FighterType,
        ) {
            GameInfo {
                current_player1: current_player1,
                player1_info: player1_info,
                player1_current_move: player1_current_move,
                player1_move_info: player1_move_info,
                current_player2: current_player2,
                player2_info: player2_info,
                player2_current_move: player2_current_move,
                player2_move_info: player2_move_info,
                winning_player: winning_player,
            };
        }
    }

    //initializing the players here. would we want this as a multiplayer game? Would that be possible?
    //aka eliminate the iscpu screen
    let mut f1 = Fighter::<FighterType> {
        name: FighterType::None,
        is_cpu: false,
        is_turn: true,
        health: 100,
        mana: 20,
    };
    let mut f2 = Fighter::<FighterType> {
        name: FighterType::None,
        is_cpu: true,
        is_turn: false,
        health: 100,
        mana: 20,
    };
    let mut player1_finish_selecting = false;
    let mut player2_finish_selecting = false;
    let mut player1_selected = false;
    let mut player2_selected = false;
    let mut player1_finish_selecting_move = false;
    let mut player2_finish_selecting_move = false;
    let mut player1_move_selected = false;
    let mut player2_move_selected = false;
    let mut gameinfo = GameInfo {
        current_player1: FighterType::None,
        player1_info: FighterType::None,
        player1_current_move: placeholder_fightermove,
        player1_move_info: placeholder_fightermove,
        current_player2: FighterType::None,
        player2_info: FighterType::None,
        player2_current_move: placeholder_fightermove,
        player2_move_info: placeholder_fightermove,
        winning_player: FighterType::None,
    };
    let mut done_execute_move = false;
    //
    let mut player_info = FighterType::None;

    let mut player_info_temp = 0;

    let mut p1_initial_health = 0;
    let mut p2_initial_health = 0;
    let mut p1_initial_mana = 0;
    let mut p2_initial_mana = 0;

    let pick_bars_draw_to = Vec2i {
        x: 10,
        y: HEIGHT as i32 - 48,
    };
    let bar_y = 10;
    let pick_bars_outof = 100;
    let mut audio_play = true;

    // let letters_frames: Vec<u32> = (65..71).collect();

    //we are going to want to define our FighterMoves in here
    //and then initialize the three fighters here with the FighterMoves in the move_inventory

    // let mut audio_play = true;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                vulkan_state.recreate_swapchain = true;
            }

            // putting this here in case we want keyboard input in the future
            Event::WindowEvent {
                // Note this deeply nested pattern match
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                // Which serves to filter out only events we actually want
                                virtual_keycode: Some(keycode),
                                state,
                                ..
                            },
                        ..
                    },
                ..
            } => {
                // It also binds these handy variable names!
                match state {
                    winit::event::ElementState::Pressed => {
                        // VirtualKeycode is an enum with a defined representation
                        now_keys[keycode as usize] = true;
                    }
                    winit::event::ElementState::Released => {
                        now_keys[keycode as usize] = false;
                    }
                }
            }

            // MOUSE INPUT
            Event::WindowEvent {
                // Note this deeply nested pattern match
                event:
                    WindowEvent::CursorMoved {
                        position: winit::dpi::PhysicalPosition { x, y, .. },
                        ..
                    },
                ..
            } => {
                // this is just resizing the mouse_x and mouse_y based on the dimensions
                let dimensions = vulkan_state.viewport.dimensions;
                if ((y as f32) < dimensions[1]) && (y > 0.0) {
                    mouse_y = ((y as f32 / dimensions[1] as f32) * HEIGHT as f32) as f32;
                    mouse_x = ((x as f32 / dimensions[0] as f32) * WIDTH as f32) as f32;
                }
            }

            // button is not being used here but later if we want to specify
            // left click vs right click, we'll want to use it
            Event::WindowEvent {
                // Note this deeply nested pattern match
                event: WindowEvent::MouseInput { state, button, .. },
                ..
            } => match state {
                winit::event::ElementState::Pressed => {
                    if mouse_click == false {
                        mouse_click = true;
                    }
                }
                winit::event::ElementState::Released => {
                    mouse_click = false;
                }
            },

            // NewEvents: Let's start processing events.
            Event::NewEvents(_) => {
                // This used to contain now_keys and prev_keys update. Osborn told me to move
                // it to right before rendering, below
            }

            Event::MainEventsCleared => {
                {
                    // update fx: input handling, vulkan stuff
                    // then copy over prev keys from now ekys or something

                    // We need to synchronize here to send new data to the GPU.
                    // We can't send the new framebuffer until the previous frame is done being drawn.
                    // Dropping the future will block until it's done.
                    if let Some(mut fut) = vulkan_state.previous_frame_end.take() {
                        fut.cleanup_finished();
                    }
                }

                if now_keys[VirtualKeyCode::Escape as usize] {
                    *control_flow = ControlFlow::Exit;
                }

                //yellowbackground
                vulkan_state.fb2d.clear((255_u8, 242_u8, 0_u8, 100_u8));

                if audio_play == true {
                    arrangement_handle.play(InstanceSettings::default());
                    audio_play = false;
                }

                if game.state == GameStates::MainScreen {
                    vulkan_state
                        .fb2d
                        .bitblt(&main_screen, &main_screen_rect, main_screen_draw_to);

                    vulkan_state
                        .fb2d
                        .bitblt(&play_button, &fighter_info_rect, next_button_draw_to);

                    if mouse_click == true && prev_mouse_click == false {
                        let mouse_pos = engine::image::Vec2i {
                            x: mouse_x as i32,
                            y: mouse_y as i32,
                        };

                        if next_button_clickable_rect.rect_inside(mouse_pos) {
                            coin_handle_click.play(InstanceSettings::default());
                            game.state = GameStates::ChooseFighter;
                        }
                    }
                } else if game.state == GameStates::ChooseFighter {
                    if !player1_finish_selecting {
                        if player1_selected {
                            if gameinfo.current_player1 == FighterType::Nate {
                                vulkan_state.fb2d.bitblt(
                                    &highlight_rect,
                                    &highlight_rect_rect,
                                    highlight_rect_draw_to_1,
                                );
                            }

                            if gameinfo.current_player1 == FighterType::Chloe {
                                vulkan_state.fb2d.bitblt(
                                    &highlight_rect,
                                    &highlight_rect_rect,
                                    highlight_rect_draw_to_2,
                                );
                            }

                            if gameinfo.current_player1 == FighterType::Grace {
                                vulkan_state.fb2d.bitblt(
                                    &highlight_rect,
                                    &highlight_rect_rect,
                                    highlight_rect_draw_to_3,
                                );
                            }
                        }
                        //draw a rectangle that's a little larger than the fighter_rect
                        //to show that the current fighter is selected (aka highlighting the fighter)

                        //header text
                        vulkan_state.fb2d.bitblt(
                            &header_player1,
                            &header_rect,
                            player_header_rect_draw_to,
                        );

                        //nate image
                        vulkan_state.fb2d.bitblt(
                            &nate_fighter_rect,
                            &fighter_rect_rect,
                            fighter_rect_draw_to_1,
                        );

                        //chloe image
                        vulkan_state.fb2d.bitblt(
                            &chloe_fighter_rect,
                            &fighter_rect_rect,
                            fighter_rect_draw_to_2,
                        );

                        //grace image
                        vulkan_state.fb2d.bitblt(
                            &grace_fighter_rect,
                            &fighter_rect_rect,
                            fighter_rect_draw_to_3,
                        );

                        //nate "read more info" button
                        vulkan_state.fb2d.bitblt(
                            &fighter_info,
                            &fighter_info_rect,
                            fighter_info_draw_to_1,
                        );

                        //chloe "read more info" button
                        vulkan_state.fb2d.bitblt(
                            &fighter_info,
                            &fighter_info_rect,
                            fighter_info_draw_to_2,
                        );

                        //grace "read more info" button
                        vulkan_state.fb2d.bitblt(
                            &fighter_info,
                            &fighter_info_rect,
                            fighter_info_draw_to_3,
                        );

                        //"select" button
                        vulkan_state.fb2d.bitblt(
                            &select_button,
                            &fighter_info_rect,
                            next_button_draw_to,
                        );

                        if mouse_click == true && prev_mouse_click == false {
                            let mouse_pos = engine::image::Vec2i {
                                x: mouse_x as i32,
                                y: mouse_y as i32,
                            };

                            if fighter_rect_clickable_rect_1.rect_inside(mouse_pos) {
                                player1_selected = true;
                                click_handle_click.play(InstanceSettings::default());
                                gameinfo.current_player1 = FighterType::Nate;
                            }

                            if fighter_rect_clickable_rect_2.rect_inside(mouse_pos) {
                                player1_selected = true;
                                click_handle_click.play(InstanceSettings::default());
                                gameinfo.current_player1 = FighterType::Chloe;
                            }

                            if fighter_rect_clickable_rect_3.rect_inside(mouse_pos) {
                                player1_selected = true;
                                click_handle_click.play(InstanceSettings::default());
                                gameinfo.current_player1 = FighterType::Grace;
                            }

                            if fighter_info_clickable_rect_1.rect_inside(mouse_pos) {
                                gameinfo.player1_info = FighterType::Nate;
                                click_handle_click.play(InstanceSettings::default());
                                game.state = GameStates::FighterInfo;
                            }

                            if fighter_info_clickable_rect_2.rect_inside(mouse_pos) {
                                gameinfo.player1_info = FighterType::Chloe;
                                click_handle_click.play(InstanceSettings::default());
                                game.state = GameStates::FighterInfo;
                            }

                            if fighter_info_clickable_rect_3.rect_inside(mouse_pos) {
                                gameinfo.player1_info = FighterType::Grace;
                                click_handle_click.play(InstanceSettings::default());
                                game.state = GameStates::FighterInfo;
                            }

                            if next_button_clickable_rect.rect_inside(mouse_pos) && player1_selected
                            {
                                coin_handle_click.play(InstanceSettings::default());
                                player1_finish_selecting = true;
                            }
                        }
                    } else if !player2_finish_selecting {
                        //add highlight to the boxes
                        if player2_selected {
                            if gameinfo.current_player2 == FighterType::Nate {
                                vulkan_state.fb2d.bitblt(
                                    &highlight_rect,
                                    &highlight_rect_rect,
                                    highlight_rect_draw_to_1,
                                );
                            }

                            if gameinfo.current_player2 == FighterType::Chloe {
                                vulkan_state.fb2d.bitblt(
                                    &highlight_rect,
                                    &highlight_rect_rect,
                                    highlight_rect_draw_to_2,
                                );
                            }

                            if gameinfo.current_player2 == FighterType::Grace {
                                vulkan_state.fb2d.bitblt(
                                    &highlight_rect,
                                    &highlight_rect_rect,
                                    highlight_rect_draw_to_3,
                                );
                            }
                        }
                        //draw a rectangle that's a little larger than the fighter_rect
                        //to show that the current fighter is selected (aka highlighting the fighter)

                        //header
                        vulkan_state.fb2d.bitblt(
                            &header_player2,
                            &header_rect,
                            player_header_rect_draw_to,
                        );

                        //nate image
                        vulkan_state.fb2d.bitblt(
                            &nate_fighter_rect,
                            &fighter_rect_rect,
                            fighter_rect_draw_to_1,
                        );

                        //chloe image
                        vulkan_state.fb2d.bitblt(
                            &chloe_fighter_rect,
                            &fighter_rect_rect,
                            fighter_rect_draw_to_2,
                        );

                        //grace image
                        vulkan_state.fb2d.bitblt(
                            &grace_fighter_rect,
                            &fighter_rect_rect,
                            fighter_rect_draw_to_3,
                        );

                        //nate "read more info" button
                        vulkan_state.fb2d.bitblt(
                            &fighter_info,
                            &fighter_info_rect,
                            fighter_info_draw_to_1,
                        );

                        //chloe "read more info" button
                        vulkan_state.fb2d.bitblt(
                            &fighter_info,
                            &fighter_info_rect,
                            fighter_info_draw_to_2,
                        );

                        //grace "read more info" button
                        vulkan_state.fb2d.bitblt(
                            &fighter_info,
                            &fighter_info_rect,
                            fighter_info_draw_to_3,
                        );

                        //"select" button
                        vulkan_state.fb2d.bitblt(
                            &select_button,
                            &fighter_info_rect,
                            next_button_draw_to,
                        );

                        if mouse_click == true && prev_mouse_click == false {
                            let mouse_pos = engine::image::Vec2i {
                                x: mouse_x as i32,
                                y: mouse_y as i32,
                            };

                            if fighter_rect_clickable_rect_1.rect_inside(mouse_pos) {
                                player2_selected = true;
                                click_handle_click.play(InstanceSettings::default());
                                gameinfo.current_player2 = FighterType::Nate;
                            }

                            if fighter_rect_clickable_rect_2.rect_inside(mouse_pos) {
                                player2_selected = true;
                                click_handle_click.play(InstanceSettings::default());
                                gameinfo.current_player2 = FighterType::Chloe;
                            }

                            if fighter_rect_clickable_rect_3.rect_inside(mouse_pos) {
                                player2_selected = true;
                                click_handle_click.play(InstanceSettings::default());
                                gameinfo.current_player2 = FighterType::Grace;
                            }

                            if fighter_info_clickable_rect_1.rect_inside(mouse_pos) {
                                gameinfo.player2_info = FighterType::Nate;
                                click_handle_click.play(InstanceSettings::default());
                                game.state = GameStates::FighterInfo;
                            }

                            if fighter_info_clickable_rect_2.rect_inside(mouse_pos) {
                                gameinfo.player2_info = FighterType::Chloe;
                                click_handle_click.play(InstanceSettings::default());
                                game.state = GameStates::FighterInfo;
                            }

                            if fighter_info_clickable_rect_3.rect_inside(mouse_pos) {
                                gameinfo.player2_info = FighterType::Grace;
                                click_handle_click.play(InstanceSettings::default());
                                game.state = GameStates::FighterInfo;
                            }

                            if next_button_clickable_rect.rect_inside(mouse_pos) && player2_selected
                            {
                                player2_finish_selecting = true;
                                coin_handle_click.play(InstanceSettings::default());
                                game.state = GameStates::ChooseMove;
                            }
                        }
                    }
                }
                //gamestate::fighterinfo
                else if game.state == GameStates::FighterInfo {
                    //back button

                    // back button back to GameStates::ChooseFighter
                    // we can replace this with an image later
                    vulkan_state
                        .fb2d
                        .bitblt(&mut back_button, &back_button_rect, back_button_to);

                    if mouse_click == true && prev_mouse_click == false {
                        let mouse_pos = engine::image::Vec2i {
                            x: mouse_x as i32,
                            y: mouse_y as i32,
                        };

                        if back_button_clickable_rect.rect_inside(mouse_pos) {
                            click_handle_click.play(InstanceSettings::default());
                            game.state = GameStates::ChooseFighter;
                        }
                    }
                    let current_info = if player1_finish_selecting {
                        gameinfo.player2_info
                    } else {
                        gameinfo.player1_info
                    };

                    //if player_info == nate, bit blit certain images
                    if current_info == FighterType::Nate {
                        vulkan_state.fb2d.write_to_font(
                            nate_strings.name,
                            &mut titlefont,
                            title_draw_to,
                            title_box_dim,
                        );

                        vulkan_state.fb2d.write_to_font(
                            nate_strings.desc,
                            &mut font,
                            description_draw_to,
                            description_box_dim,
                        );
                    }

                    //if player_info == chloe, bit blit certain images
                    if current_info == FighterType::Chloe {
                        vulkan_state.fb2d.write_to_font(
                            chloe_strings.name,
                            &mut titlefont,
                            title_draw_to,
                            title_box_dim,
                        );
                        vulkan_state.fb2d.write_to_font(
                            chloe_strings.desc,
                            &mut font,
                            description_draw_to,
                            description_box_dim,
                        );
                    }
                    //if player_info == grace, bit blit certain images into the rectangles
                    if current_info == FighterType::Grace {
                        vulkan_state.fb2d.write_to_font(
                            grace_strings.name,
                            &mut titlefont,
                            title_draw_to,
                            title_box_dim,
                        );

                        vulkan_state.fb2d.write_to_font(
                            grace_strings.desc,
                            &mut font,
                            description_draw_to,
                            description_box_dim,
                        );
                    }
                }
                //gamestate::choosemove
                else if game.state == GameStates::ChooseMove {
                    done_execute_move = false;

                    //choose move header
                    vulkan_state.fb2d.bitblt(
                        &header_player1_choose_move_rect,
                        &header_choose_move_rect,
                        player_choose_move_draw_to,
                    );

                    //move 1 "read more info" button
                    vulkan_state.fb2d.bitblt(
                        &fighter_info,
                        &fighter_info_rect,
                        move_info_draw_to_1,
                    );

                    //move 2 "read more info" button
                    vulkan_state.fb2d.bitblt(
                        &fighter_info,
                        &fighter_info_rect,
                        move_info_draw_to_2,
                    );

                    //move 3 "read more info" button
                    vulkan_state.fb2d.bitblt(
                        &fighter_info,
                        &fighter_info_rect,
                        move_info_draw_to_3,
                    );

                    //"select" button
                    vulkan_state.fb2d.bitblt(
                        &select_button,
                        &fighter_info_rect,
                        next_button_draw_to,
                    );

                    if !player1_finish_selecting_move {
                        if gameinfo.current_player1 == FighterType::Nate {
                            // vulkan_state.fb2d.clear((118_u8, 188_u8, 83_u8, 100_u8));

                            //add highlight rectangle outside the selected move
                            if player1_move_selected {
                                if gameinfo.player1_current_move.fighter_move_type
                                    == FighterMoveType::NateMove1
                                {
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_1,
                                    );
                                }

                                if gameinfo.player1_current_move.fighter_move_type
                                    == FighterMoveType::NateMove2
                                {
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_2,
                                    );
                                }

                                if gameinfo.player1_current_move.fighter_move_type
                                    == FighterMoveType::NateMove3
                                {
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_3,
                                    );
                                }
                            }

                            //nate move 1
                            //health move?
                            vulkan_state.fb2d.bitblt(
                                &move_chill_pill,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_1,
                            );

                            //nate move 2
                            vulkan_state.fb2d.bitblt(
                                &move_guitar,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_2,
                            );

                            //nate move 3
                            vulkan_state.fb2d.bitblt(
                                &move_yoyo,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_3,
                            );

                            if mouse_click == true && prev_mouse_click == false {
                                let mouse_pos = engine::image::Vec2i {
                                    x: mouse_x as i32,
                                    y: mouse_y as i32,
                                };

                                //select the first move
                                if fighter_rect_clickable_rect_1.rect_inside(mouse_pos) {
                                    if f1.mana + nate_fighter_moves[0].mana_cost > -1 {
                                        println!("nate choose move 0");
                                        player1_move_selected = true;
                                        click_handle_click.play(InstanceSettings::default());
                                        gameinfo.player1_current_move = nate_fighter_moves[0];
                                    } else {
                                        println!("not enough mana doe");
                                    }
                                }

                                //select the second move
                                if fighter_rect_clickable_rect_2.rect_inside(mouse_pos) {
                                    if f1.mana + nate_fighter_moves[1].mana_cost > -1 {
                                        println!("nate choose move 1");
                                        player1_move_selected = true;
                                        click_handle_click.play(InstanceSettings::default());
                                        gameinfo.player1_current_move = nate_fighter_moves[1];
                                    } else {
                                        println!("not enough mana doe");
                                    }
                                }

                                //select the third move
                                if fighter_rect_clickable_rect_3.rect_inside(mouse_pos) {
                                    if f1.mana + nate_fighter_moves[2].mana_cost > -1 {
                                        println!("nate choose move 2");
                                        player1_move_selected = true;
                                        click_handle_click.play(InstanceSettings::default());
                                        gameinfo.player1_current_move = nate_fighter_moves[2];
                                    } else {
                                        println!("not enough mana doe");
                                    }
                                }

                                //info on first move
                                if move_info_clickable_rect_1.rect_inside(mouse_pos) {
                                    gameinfo.player1_move_info = nate_fighter_moves[0];
                                    click_handle_click.play(InstanceSettings::default());
                                    game.state = GameStates::MoveInfo;
                                }

                                //info on second move
                                if move_info_clickable_rect_2.rect_inside(mouse_pos) {
                                    gameinfo.player1_move_info = nate_fighter_moves[1];
                                    click_handle_click.play(InstanceSettings::default());
                                    game.state = GameStates::MoveInfo;
                                }

                                //info on third move
                                if move_info_clickable_rect_3.rect_inside(mouse_pos) {
                                    gameinfo.player1_move_info = nate_fighter_moves[2];
                                    click_handle_click.play(InstanceSettings::default());
                                    game.state = GameStates::MoveInfo;
                                }

                                //select move and go to next state
                                if next_button_clickable_rect.rect_inside(mouse_pos)
                                    && player1_move_selected
                                {
                                    player1_finish_selecting_move = true;
                                    coin_handle_click.play(InstanceSettings::default());
                                }
                            }
                        }
                        if gameinfo.current_player1 == FighterType::Chloe {
                            // vulkan_state.fb2d.clear((80_u8, 150_u8, 248_u8, 100_u8));
                            if player1_selected {
                                if gameinfo.player1_current_move.fighter_move_type
                                    == FighterMoveType::ChloeMove1
                                {
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_1,
                                    );
                                }

                                if gameinfo.player1_current_move.fighter_move_type
                                    == FighterMoveType::ChloeMove2
                                {
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_2,
                                    );
                                }

                                if gameinfo.player1_current_move.fighter_move_type
                                    == FighterMoveType::ChloeMove3
                                {
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_3,
                                    );
                                }
                            }

                            //chloemove1
                            vulkan_state.fb2d.bitblt(
                                &move_metaphysical_inquiry,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_1,
                            );

                            //chloemove2
                            vulkan_state.fb2d.bitblt(
                                &move_curb_stomp,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_2,
                            );

                            //chloemove3
                            vulkan_state.fb2d.bitblt(
                                &move_hacker,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_3,
                            );

                            if mouse_click == true && prev_mouse_click == false {
                                let mouse_pos = engine::image::Vec2i {
                                    x: mouse_x as i32,
                                    y: mouse_y as i32,
                                };

                                //select the first move
                                if fighter_rect_clickable_rect_1.rect_inside(mouse_pos) {
                                    if f1.mana + chloe_fighter_moves[0].mana_cost > -1 {
                                        player1_move_selected = true;
                                        click_handle_click.play(InstanceSettings::default());
                                        gameinfo.player1_current_move = chloe_fighter_moves[0];
                                    } else {
                                        println!("not enough mana doe");
                                    }
                                }

                                //select the second move
                                if fighter_rect_clickable_rect_2.rect_inside(mouse_pos) {
                                    if f1.mana + chloe_fighter_moves[1].mana_cost > -1 {
                                        player1_move_selected = true;
                                        click_handle_click.play(InstanceSettings::default());
                                        gameinfo.player1_current_move = chloe_fighter_moves[1];
                                    } else {
                                        println!("not enough mana doe");
                                    }
                                }

                                //select the third move
                                if fighter_rect_clickable_rect_3.rect_inside(mouse_pos) {
                                    if f1.mana + chloe_fighter_moves[2].mana_cost > -1 {
                                        player1_move_selected = true;
                                        click_handle_click.play(InstanceSettings::default());
                                        gameinfo.player1_current_move = chloe_fighter_moves[2];
                                    } else {
                                        println!("not enough mana doe");
                                    }
                                }

                                //info on first move
                                if move_info_clickable_rect_1.rect_inside(mouse_pos) {
                                    gameinfo.player1_move_info = chloe_fighter_moves[0];
                                    click_handle_click.play(InstanceSettings::default());
                                    game.state = GameStates::MoveInfo;
                                }

                                //info on second move
                                if move_info_clickable_rect_2.rect_inside(mouse_pos) {
                                    gameinfo.player1_move_info = chloe_fighter_moves[1];
                                    click_handle_click.play(InstanceSettings::default());
                                    game.state = GameStates::MoveInfo;
                                }

                                //info on third move
                                if move_info_clickable_rect_3.rect_inside(mouse_pos) {
                                    gameinfo.player1_move_info = chloe_fighter_moves[2];
                                    click_handle_click.play(InstanceSettings::default());
                                    game.state = GameStates::MoveInfo;
                                }

                                //select move and go to next state
                                if next_button_clickable_rect.rect_inside(mouse_pos)
                                    && player1_move_selected
                                {
                                    player1_finish_selecting_move = true;
                                    coin_handle_click.play(InstanceSettings::default());
                                }
                            }
                        }
                        if gameinfo.current_player1 == FighterType::Grace {
                            // vulkan_state.fb2d.clear((198_u8, 82_u8, 140_u8, 100_u8));
                            if player2_selected {
                                if gameinfo.player1_current_move.fighter_move_type
                                    == FighterMoveType::GraceMove1
                                {
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_1,
                                    );
                                }

                                if gameinfo.player1_current_move.fighter_move_type
                                    == FighterMoveType::GraceMove2
                                {
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_2,
                                    );
                                }

                                if gameinfo.player1_current_move.fighter_move_type
                                    == FighterMoveType::GraceMove3
                                {
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_3,
                                    );
                                }
                            }

                            //grace move 1
                            vulkan_state.fb2d.bitblt(
                                &move_grilled_cheese,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_1,
                            );

                            //grace move 2
                            vulkan_state.fb2d.bitblt(
                                &move_prank,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_2,
                            );

                            //grace move 3
                            vulkan_state.fb2d.bitblt(
                                &move_mystery_box,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_3,
                            );

                            if mouse_click == true && prev_mouse_click == false {
                                let mouse_pos = engine::image::Vec2i {
                                    x: mouse_x as i32,
                                    y: mouse_y as i32,
                                };

                                //select the first move
                                if fighter_rect_clickable_rect_1.rect_inside(mouse_pos) {
                                    if f1.mana + grace_fighter_moves[0].mana_cost > -1 {
                                        println!("true");
                                        player1_move_selected = true;
                                        click_handle_click.play(InstanceSettings::default());
                                        gameinfo.player1_current_move = grace_fighter_moves[0];
                                    } else {
                                        println!("Not enough mana doe");
                                    }
                                }

                                //select the second move
                                if fighter_rect_clickable_rect_2.rect_inside(mouse_pos) {
                                    if f1.mana + grace_fighter_moves[1].mana_cost > -1 {
                                        player1_move_selected = true;
                                        click_handle_click.play(InstanceSettings::default());
                                        gameinfo.player1_current_move = grace_fighter_moves[1];
                                    } else {
                                        println!("Not enough mana doe");
                                    }
                                }

                                //select the third move
                                if fighter_rect_clickable_rect_3.rect_inside(mouse_pos) {
                                    if f1.mana + grace_fighter_moves[2].mana_cost > -1 {
                                        player1_move_selected = true;
                                        click_handle_click.play(InstanceSettings::default());
                                        gameinfo.player1_current_move = grace_fighter_moves[2];
                                    } else {
                                        println!("Not enough mana doe");
                                    }
                                }

                                //info on first move
                                if move_info_clickable_rect_1.rect_inside(mouse_pos) {
                                    gameinfo.player1_move_info = grace_fighter_moves[0];
                                    click_handle_click.play(InstanceSettings::default());
                                    game.state = GameStates::MoveInfo;
                                }

                                //info on second move
                                if move_info_clickable_rect_2.rect_inside(mouse_pos) {
                                    gameinfo.player1_move_info = grace_fighter_moves[1];
                                    click_handle_click.play(InstanceSettings::default());
                                    game.state = GameStates::MoveInfo;
                                }

                                //info on third move
                                if move_info_clickable_rect_3.rect_inside(mouse_pos) {
                                    gameinfo.player1_move_info = grace_fighter_moves[2];
                                    click_handle_click.play(InstanceSettings::default());
                                    game.state = GameStates::MoveInfo;
                                }

                                //select move and go to next state
                                if next_button_clickable_rect.rect_inside(mouse_pos)
                                    && player1_move_selected
                                {
                                    player1_finish_selecting_move = true;
                                    coin_handle_click.play(InstanceSettings::default());
                                }
                            }
                        }

                        vulkan_state.fb2d.write_to_font(
                            "HP:",
                            &mut font,
                            Vec2i {
                                x: pick_bars_draw_to.x,
                                y: pick_bars_draw_to.y,
                            },
                            Vec2i {
                                x: fontsize as i32 * 3,
                                y: fontsize_h as i32,
                            },
                        );

                        let hp_str = f1.health.to_string() + "/100";
                        vulkan_state.fb2d.write_to_font(
                            &hp_str[..],
                            &mut font,
                            Vec2i {
                                x: pick_bars_draw_to.x + (fontsize as i32 * 3),
                                y: pick_bars_draw_to.y,
                            },
                            Vec2i {
                                x: fontsize as i32 * 7,
                                y: fontsize_h as i32,
                            },
                        );

                        vulkan_state.fb2d.write_to_font(
                            "MANA:",
                            &mut font,
                            Vec2i {
                                x: pick_bars_draw_to.x,
                                y: pick_bars_draw_to.y + fontsize_h as i32,
                            },
                            Vec2i {
                                x: fontsize as i32 * 5,
                                y: fontsize_h as i32,
                            },
                        );

                        let mana_str = f1.mana.to_string() + "/100";
                        vulkan_state.fb2d.write_to_font(
                            &mana_str[..],
                            &mut font,
                            Vec2i {
                                x: pick_bars_draw_to.x + (fontsize as i32 * 5),
                                y: pick_bars_draw_to.y + fontsize_h as i32,
                            },
                            Vec2i {
                                x: fontsize as i32 * 7,
                                y: fontsize_h as i32,
                            },
                        );
                    } else if !player2_finish_selecting_move {
                        if gameinfo.current_player2 == FighterType::Nate {
                            // vulkan_state.fb2d.clear((118_u8, 188_u8, 83_u8, 100_u8));

                            //add highlight rectangle outside the selected move
                            if player2_move_selected {
                                if gameinfo.player2_current_move.fighter_move_type
                                    == FighterMoveType::NateMove1
                                {
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_1,
                                    );
                                }

                                if gameinfo.player2_current_move.fighter_move_type
                                    == FighterMoveType::NateMove2
                                {
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_2,
                                    );
                                }

                                if gameinfo.player2_current_move.fighter_move_type
                                    == FighterMoveType::NateMove3
                                {
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_3,
                                    );
                                }
                            }

                            //nate move 1
                            //health move?
                            vulkan_state.fb2d.bitblt(
                                &move_chill_pill,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_1,
                            );

                            //nate move 2
                            vulkan_state.fb2d.bitblt(
                                &move_guitar,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_2,
                            );

                            //nate move 3
                            vulkan_state.fb2d.bitblt(
                                &move_yoyo,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_3,
                            );

                            if mouse_click == true && prev_mouse_click == false {
                                let mouse_pos = engine::image::Vec2i {
                                    x: mouse_x as i32,
                                    y: mouse_y as i32,
                                };

                                // // ENEMY SELECT MOVE (need to figure out random move)
                                // let mut rng = rand::thread_rng();
                                // let ai_move = moves[rng.gen_range(0, 3)];

                                //select the first move
                                if fighter_rect_clickable_rect_1.rect_inside(mouse_pos) {
                                    if f2.mana + nate_fighter_moves[0].mana_cost > -1 {
                                        player2_move_selected = true;
                                        click_handle_click.play(InstanceSettings::default());
                                        gameinfo.player2_current_move = nate_fighter_moves[0];
                                    } else {
                                        println!("not enough mana doe");
                                    }
                                }

                                //select the second move
                                if fighter_rect_clickable_rect_2.rect_inside(mouse_pos) {
                                    if f2.mana + nate_fighter_moves[1].mana_cost > -1 {
                                        player2_move_selected = true;
                                        click_handle_click.play(InstanceSettings::default());
                                        gameinfo.player2_current_move = nate_fighter_moves[1];
                                    } else {
                                        println!("not enough mana doe");
                                    }
                                }

                                //select the third move
                                if fighter_rect_clickable_rect_3.rect_inside(mouse_pos) {
                                    if f2.mana + nate_fighter_moves[2].mana_cost > -1 {
                                        player2_move_selected = true;
                                        click_handle_click.play(InstanceSettings::default());
                                        gameinfo.player2_current_move = nate_fighter_moves[2];
                                    } else {
                                        println!("not enough mana doe");
                                    }
                                }

                                //info on first move
                                if move_info_clickable_rect_1.rect_inside(mouse_pos) {
                                    gameinfo.player2_move_info = nate_fighter_moves[0];
                                    click_handle_click.play(InstanceSettings::default());
                                    game.state = GameStates::MoveInfo;
                                }

                                //info on second move
                                if move_info_clickable_rect_2.rect_inside(mouse_pos) {
                                    gameinfo.player2_move_info = nate_fighter_moves[1];
                                    click_handle_click.play(InstanceSettings::default());
                                    game.state = GameStates::MoveInfo;
                                }

                                //info on third move
                                if move_info_clickable_rect_3.rect_inside(mouse_pos) {
                                    gameinfo.player2_move_info = nate_fighter_moves[2];
                                    click_handle_click.play(InstanceSettings::default());
                                    game.state = GameStates::MoveInfo;
                                }

                                //select move and go to next state
                                if next_button_clickable_rect.rect_inside(mouse_pos)
                                    && player2_move_selected
                                {
                                    player2_finish_selecting_move = true;
                                    coin_handle_click.play(InstanceSettings::default());
                                    game.state = GameStates::ShowPick;
                                }
                            }
                        }
                        if gameinfo.current_player2 == FighterType::Chloe {
                            // vulkan_state.fb2d.clear((80_u8, 150_u8, 248_u8, 100_u8));
                            if player2_selected {
                                if gameinfo.player2_current_move.fighter_move_type
                                    == FighterMoveType::ChloeMove1
                                {
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_1,
                                    );
                                }

                                if gameinfo.player2_current_move.fighter_move_type
                                    == FighterMoveType::ChloeMove2
                                {
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_2,
                                    );
                                }

                                if gameinfo.player2_current_move.fighter_move_type
                                    == FighterMoveType::ChloeMove3
                                {
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_3,
                                    );
                                }
                            }

                            //chloemove1
                            vulkan_state.fb2d.bitblt(
                                &move_metaphysical_inquiry,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_1,
                            );

                            //chloemove2
                            vulkan_state.fb2d.bitblt(
                                &move_curb_stomp,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_2,
                            );

                            //chloemove3
                            vulkan_state.fb2d.bitblt(
                                &move_hacker,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_3,
                            );

                            if mouse_click == true && prev_mouse_click == false {
                                let mouse_pos = engine::image::Vec2i {
                                    x: mouse_x as i32,
                                    y: mouse_y as i32,
                                };

                                //select the first move
                                if fighter_rect_clickable_rect_1.rect_inside(mouse_pos) {
                                    if f2.mana + chloe_fighter_moves[0].mana_cost > -1 {
                                        player2_move_selected = true;
                                        click_handle_click.play(InstanceSettings::default());
                                        gameinfo.player2_current_move = chloe_fighter_moves[0];
                                    } else {
                                        println!("not enough mana doe");
                                    }
                                }

                                //select the second move
                                if fighter_rect_clickable_rect_2.rect_inside(mouse_pos) {
                                    if f2.mana + chloe_fighter_moves[1].mana_cost > -1 {
                                        player2_move_selected = true;
                                        click_handle_click.play(InstanceSettings::default());
                                        gameinfo.player2_current_move = chloe_fighter_moves[1];
                                    } else {
                                        println!("not enough mana doe");
                                    }
                                }

                                //select the third move
                                if fighter_rect_clickable_rect_3.rect_inside(mouse_pos) {
                                    if f2.mana + chloe_fighter_moves[2].mana_cost > -1 {
                                        player2_move_selected = true;
                                        click_handle_click.play(InstanceSettings::default());
                                        gameinfo.player2_current_move = chloe_fighter_moves[2];
                                    } else {
                                        println!("not enough mana doe");
                                    }
                                }

                                //info on first move
                                if move_info_clickable_rect_1.rect_inside(mouse_pos) {
                                    gameinfo.player2_move_info = chloe_fighter_moves[0];
                                    click_handle_click.play(InstanceSettings::default());
                                    game.state = GameStates::MoveInfo;
                                }

                                //info on second move
                                if move_info_clickable_rect_2.rect_inside(mouse_pos) {
                                    gameinfo.player2_move_info = chloe_fighter_moves[1];
                                    click_handle_click.play(InstanceSettings::default());
                                    game.state = GameStates::MoveInfo;
                                }

                                //info on third move
                                if move_info_clickable_rect_3.rect_inside(mouse_pos) {
                                    gameinfo.player2_move_info = chloe_fighter_moves[2];
                                    click_handle_click.play(InstanceSettings::default());
                                    game.state = GameStates::MoveInfo;
                                }

                                //select move and go to next state
                                if next_button_clickable_rect.rect_inside(mouse_pos)
                                    && player2_move_selected
                                {
                                    player2_finish_selecting_move = true;
                                    coin_handle_click.play(InstanceSettings::default());
                                    game.state = GameStates::ShowPick;
                                }
                            }
                        }
                        if gameinfo.current_player2 == FighterType::Grace {
                            // vulkan_state.fb2d.clear((198_u8, 82_u8, 140_u8, 100_u8));
                            if player2_selected {
                                if gameinfo.player2_current_move.fighter_move_type
                                    == FighterMoveType::GraceMove1
                                {
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_1,
                                    );
                                }

                                if gameinfo.player2_current_move.fighter_move_type
                                    == FighterMoveType::GraceMove2
                                {
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_2,
                                    );
                                }

                                if gameinfo.player2_current_move.fighter_move_type
                                    == FighterMoveType::GraceMove3
                                {
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_3,
                                    );
                                }
                            }

                            //grace move 1
                            vulkan_state.fb2d.bitblt(
                                &move_grilled_cheese,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_1,
                            );

                            //grace move 2
                            vulkan_state.fb2d.bitblt(
                                &move_prank,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_2,
                            );

                            //grace move 3
                            vulkan_state.fb2d.bitblt(
                                &move_mystery_box,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_3,
                            );

                            if mouse_click == true && prev_mouse_click == false {
                                let mouse_pos = engine::image::Vec2i {
                                    x: mouse_x as i32,
                                    y: mouse_y as i32,
                                };

                                //select the first move
                                if fighter_rect_clickable_rect_1.rect_inside(mouse_pos) {
                                    if f2.mana + grace_fighter_moves[0].mana_cost > -1 {
                                        player2_move_selected = true;
                                        click_handle_click.play(InstanceSettings::default());
                                        gameinfo.player2_current_move = grace_fighter_moves[0];
                                    } else {
                                        println!("not enough mana doe");
                                    }
                                }

                                //select the second move
                                if fighter_rect_clickable_rect_2.rect_inside(mouse_pos) {
                                    if f2.mana + grace_fighter_moves[1].mana_cost > -1 {
                                        player2_move_selected = true;
                                        click_handle_click.play(InstanceSettings::default());
                                        gameinfo.player2_current_move = grace_fighter_moves[1];
                                    } else {
                                        println!("not enough mana doe");
                                    }
                                }

                                //select the third move
                                if fighter_rect_clickable_rect_3.rect_inside(mouse_pos) {
                                    if f2.mana + grace_fighter_moves[2].mana_cost > -1 {
                                        player2_move_selected = true;
                                        click_handle_click.play(InstanceSettings::default());
                                        gameinfo.player2_current_move = grace_fighter_moves[2];
                                    } else {
                                        println!("not enough mana doe");
                                    }
                                }

                                //info on first move
                                if move_info_clickable_rect_1.rect_inside(mouse_pos) {
                                    gameinfo.player2_move_info = grace_fighter_moves[0];
                                    click_handle_click.play(InstanceSettings::default());
                                    game.state = GameStates::MoveInfo;
                                }

                                //info on second move
                                if move_info_clickable_rect_2.rect_inside(mouse_pos) {
                                    gameinfo.player2_move_info = nate_fighter_moves[1];
                                    click_handle_click.play(InstanceSettings::default());
                                    game.state = GameStates::MoveInfo;
                                }

                                //info on third move
                                if move_info_clickable_rect_3.rect_inside(mouse_pos) {
                                    gameinfo.player2_move_info = grace_fighter_moves[2];
                                    click_handle_click.play(InstanceSettings::default());
                                    game.state = GameStates::MoveInfo;
                                }

                                //select move and go to next state
                                if next_button_clickable_rect.rect_inside(mouse_pos)
                                    && player2_move_selected
                                {
                                    player2_finish_selecting_move = true;
                                    coin_handle_click.play(InstanceSettings::default());
                                    game.state = GameStates::ShowPick;
                                }
                            }
                        }

                        vulkan_state.fb2d.write_to_font(
                            "HP:",
                            &mut font,
                            Vec2i {
                                x: pick_bars_draw_to.x,
                                y: pick_bars_draw_to.y,
                            },
                            Vec2i {
                                x: fontsize as i32 * 3,
                                y: fontsize_h as i32,
                            },
                        );

                        let hp_str = f2.health.to_string() + "/100";
                        vulkan_state.fb2d.write_to_font(
                            &hp_str[..],
                            &mut font,
                            Vec2i {
                                x: pick_bars_draw_to.x + (fontsize as i32 * 3),
                                y: pick_bars_draw_to.y,
                            },
                            Vec2i {
                                x: fontsize as i32 * 7,
                                y: fontsize_h as i32,
                            },
                        );

                        vulkan_state.fb2d.write_to_font(
                            "MANA:",
                            &mut font,
                            Vec2i {
                                x: pick_bars_draw_to.x,
                                y: pick_bars_draw_to.y + fontsize_h as i32,
                            },
                            Vec2i {
                                x: fontsize as i32 * 5,
                                y: fontsize_h as i32,
                            },
                        );

                        let mana_str = f2.mana.to_string() + "/100";
                        vulkan_state.fb2d.write_to_font(
                            &mana_str[..],
                            &mut font,
                            Vec2i {
                                x: pick_bars_draw_to.x + (fontsize as i32 * 5),
                                y: pick_bars_draw_to.y + fontsize_h as i32,
                            },
                            Vec2i {
                                x: fontsize as i32 * 7,
                                y: fontsize_h as i32,
                            },
                        );
                    }
                } else if game.state == GameStates::MoveInfo {
                    vulkan_state
                        .fb2d
                        .bitblt(&mut back_button, &back_button_rect, back_button_to);

                    if mouse_click == true && prev_mouse_click == false {
                        let mouse_pos = engine::image::Vec2i {
                            x: mouse_x as i32,
                            y: mouse_y as i32,
                        };

                        if back_button_clickable_rect.rect_inside(mouse_pos) {
                            click_handle_click.play(InstanceSettings::default());
                            game.state = GameStates::ChooseMove;
                        }
                    }

                    let current_info = if player1_finish_selecting_move {
                        gameinfo.player2_move_info.fighter_move_type
                    } else {
                        gameinfo.player1_move_info.fighter_move_type
                    };

                    //if player_info == nate, bit blit certain images
                    if current_info == FighterMoveType::NateMove1 {
                        vulkan_state.fb2d.write_to_font(
                            nate_strings.move1_name,
                            &mut titlefont,
                            title_draw_to,
                            title_box_dim,
                        );
                        vulkan_state.fb2d.write_to_font(
                            nate_strings.move1_desc,
                            &mut font,
                            description_draw_to,
                            description_box_dim,
                        );
                    }

                    if current_info == FighterMoveType::NateMove2 {
                        vulkan_state.fb2d.write_to_font(
                            nate_strings.move2_name,
                            &mut titlefont,
                            title_draw_to,
                            title_box_dim,
                        );
                        vulkan_state.fb2d.write_to_font(
                            nate_strings.move2_desc,
                            &mut font,
                            description_draw_to,
                            description_box_dim,
                        );
                    }

                    if current_info == FighterMoveType::NateMove3 {
                        vulkan_state.fb2d.write_to_font(
                            nate_strings.move3_name,
                            &mut titlefont,
                            title_draw_to,
                            title_box_dim,
                        );
                        vulkan_state.fb2d.write_to_font(
                            nate_strings.move3_desc,
                            &mut font,
                            description_draw_to,
                            description_box_dim,
                        );
                    }

                    //if player_info == chloe, bit blit certain images
                    if current_info == FighterMoveType::ChloeMove1 {
                        vulkan_state.fb2d.write_to_font(
                            chloe_strings.move1_name,
                            &mut titlefont,
                            title_draw_to,
                            title_box_dim,
                        );
                        vulkan_state.fb2d.write_to_font(
                            chloe_strings.move1_desc,
                            &mut font,
                            description_draw_to,
                            description_box_dim,
                        );
                    }

                    if current_info == FighterMoveType::ChloeMove2 {
                        vulkan_state.fb2d.write_to_font(
                            chloe_strings.move2_name,
                            &mut titlefont,
                            title_draw_to,
                            title_box_dim,
                        );
                        vulkan_state.fb2d.write_to_font(
                            chloe_strings.move2_desc,
                            &mut font,
                            description_draw_to,
                            description_box_dim,
                        );
                    }

                    if current_info == FighterMoveType::ChloeMove3 {
                        vulkan_state.fb2d.write_to_font(
                            chloe_strings.move3_name,
                            &mut titlefont,
                            title_draw_to,
                            title_box_dim,
                        );
                        vulkan_state.fb2d.write_to_font(
                            chloe_strings.move3_desc,
                            &mut font,
                            description_draw_to,
                            description_box_dim,
                        );
                    }
                    //if player_info == grace, bit blit certain images into the rectangles
                    if current_info == FighterMoveType::GraceMove1 {
                        vulkan_state.fb2d.write_to_font(
                            grace_strings.move1_name,
                            &mut titlefont,
                            title_draw_to,
                            title_box_dim,
                        );
                        vulkan_state.fb2d.write_to_font(
                            grace_strings.move1_desc,
                            &mut font,
                            description_draw_to,
                            description_box_dim,
                        );
                    }

                    if current_info == FighterMoveType::GraceMove2 {
                        vulkan_state.fb2d.write_to_font(
                            grace_strings.move2_name,
                            &mut titlefont,
                            title_draw_to,
                            title_box_dim,
                        );
                        vulkan_state.fb2d.write_to_font(
                            grace_strings.move2_desc,
                            &mut font,
                            description_draw_to,
                            description_box_dim,
                        );
                    }

                    if current_info == FighterMoveType::GraceMove3 {
                        vulkan_state.fb2d.write_to_font(
                            grace_strings.move3_name,
                            &mut titlefont,
                            title_draw_to,
                            title_box_dim,
                        );
                        vulkan_state.fb2d.write_to_font(
                            grace_strings.move3_desc,
                            &mut font,
                            description_draw_to,
                            description_box_dim,
                        );
                    }
                }
                //gamestate:showpick
                else if game.state == GameStates::ShowPick {
                    //print out updated mana's and health;

                    let player1_move_damage = gameinfo.player1_current_move.damage;
                    let player1_move_mana = gameinfo.player1_current_move.mana_cost;
                    let player1_move_health = gameinfo.player1_current_move.health_cost;
                    let player1_mana_generation = gameinfo.player1_current_move.mana_generation;

                    let player2_move_damage = gameinfo.player2_current_move.damage;
                    let player2_move_mana = gameinfo.player2_current_move.mana_cost;
                    let player2_move_health = gameinfo.player2_current_move.health_cost;
                    let player2_mana_generation = gameinfo.player2_current_move.mana_generation;

                    if done_execute_move == false {
                        p1_initial_health = f1.health;
                        p2_initial_health = f2.health;
                        p1_initial_mana = f1.mana;
                        p2_initial_mana = f2.mana;

                        //while f2.health > 0 && f2.mana > 0 && f1.health > 0 && f1.mana > 0
                        // println!("Player1 move: {:?}", gameinfo.player1_current_move.fighter_move_type);
                        // println!("Player1 current health: {:?}", f1.health);
                        // println!("Player1 current mana: {:?}", f1.mana);
                        // println!("Player2 move: {:?}", gameinfo.player2_current_move.fighter_move_type);
                        // println!("Player2 current health: {:?}", f2.health);
                        // println!("Player12 current mana: {:?}", f2.mana);

                        if f1.health + player2_move_damage < 1 {
                            gameinfo.winning_player = gameinfo.current_player2;
                            println!("go to final screen player 2 wins");
                            println!("apple {:?}", gameinfo.winning_player);
                            game.state = GameStates::FinalScreen;
                        } else {
                            f1.health += player2_move_damage;
                        }

                        // execute p1 damage
                        if f2.health + player1_move_damage < 1 {
                            println!("go to final screen player 1 wins");
                            println!("apple {:?}", gameinfo.winning_player);
                            gameinfo.winning_player = gameinfo.current_player1;
                            game.state = GameStates::FinalScreen;
                            //and gameinfo.winning player = blank
                        } else {
                            f2.health += player1_move_damage;
                        }

                        //if they both die this round then go to the end
                        if f1.health + player2_move_damage < 1
                            && f2.health + player1_move_damage < 1
                        {
                            println!("go to final screen draw");
                            println!("apple {:?}", gameinfo.winning_player);
                            gameinfo.winning_player = FighterType::Both;
                            game.state = GameStates::FinalScreen;
                        }

                        //checking so that they don't go above 100 health
                        if f2.health + player2_move_health > 100 {
                            f2.health = 100;
                        } else {
                            f2.health += player2_move_health;
                        }

                        //checking so that they don't go above 100 health
                        if f1.health + player1_move_health > 100 {
                            f1.health = 100;
                        } else {
                            f1.health += player1_move_health;
                        }

                        // Apply mana cost
                        //the "not enough mana doe" message should account for this if statement
                        //but going to add it anyways because of the issue
                        if f1.mana + gameinfo.player1_current_move.mana_cost < 0 {
                            gameinfo.winning_player = gameinfo.current_player2;
                            println!("go to final screen player 2 wins");
                            println!("apple {:?}", gameinfo.winning_player);
                            game.state = GameStates::FinalScreen;
                        } else {
                            f1.mana += gameinfo.player1_current_move.mana_cost;
                        }

                        if f2.mana + gameinfo.player2_current_move.mana_cost < 0 {
                            gameinfo.winning_player = gameinfo.current_player1;
                            println!("go to final screen player 1 wins");
                            println!("apple {:?}", gameinfo.winning_player);
                            game.state = GameStates::FinalScreen;
                        } else {
                            f2.mana += gameinfo.player2_current_move.mana_cost;
                        }

                        //Apply mana generation and check that it won't go over 100
                        if f1.mana + player1_mana_generation > 100 {
                            f1.mana = 100;
                        } else {
                            f1.mana += player1_mana_generation;
                        }

                        if f2.mana + player2_mana_generation > 100 {
                            f2.mana = 100;
                        } else {
                            f2.mana += player2_mana_generation;
                        }

                        done_execute_move = true;
                        //print out updated mana's and health's
                        println!(
                            "Player1 move: {:?}",
                            gameinfo.player1_current_move.fighter_move_type
                        );
                        println!("Player1 current health: {:?}", f1.health);
                        println!("Player1 current mana: {:?}", f1.mana);
                        println!(
                            "Player2 move: {:?}",
                            gameinfo.player2_current_move.fighter_move_type
                        );
                        println!("Player2 current health: {:?}", f2.health);
                        println!("Player2 current mana: {:?}", f2.mana);
                    }

                    // this is weird but basically if we don't have this, it'll animate one frame before moving to final screen
                    // the more refined solution would be to separate this into its own game state, i.e., CalculatePick vs ShowPick
                    // but i am lazy
                    if game.state != GameStates::FinalScreen {
                        //else if we are out of the while loop...

                        // playing anim check to reset frame counter
                        if !playing_anim {
                            pick_anim_done = false;
                            playing_anim = true;
                            pick_frame_count = 0;
                        } else if !pick_anim_done {
                            // update the frame counter every fps frames
                            if pick_frame_count % pick_fps == 0 {
                                // there are definitely better ways to do this if we have more time
                                if p1_initial_health > f1.health {
                                    p1_initial_health -= 1;
                                } else if p1_initial_health < f1.health {
                                    p1_initial_health += 1;
                                }

                                if p1_initial_mana > f1.mana {
                                    p1_initial_mana -= 1;
                                } else if p1_initial_mana < f1.mana {
                                    p1_initial_mana += 1;
                                };

                                if p2_initial_health > f2.health {
                                    p2_initial_health -= 1;
                                } else if p2_initial_health < f2.health {
                                    p2_initial_health += 1;
                                };

                                if p2_initial_mana > f2.mana {
                                    p2_initial_mana -= 1;
                                } else if p2_initial_mana < f2.mana {
                                    p2_initial_mana += 1;
                                };

                                if p1_initial_health == f1.health
                                    && p1_initial_mana == f1.mana
                                    && p2_initial_health == f2.health
                                    && p2_initial_mana == f2.mana
                                {
                                    pick_anim_done = true;
                                };
                            };
                        }

                        // drawing the p1 and p2 sprites
                        // i put them before the bars bc i think i prefer the overlap of the bars
                        // rather than the sprite on top of the bars
                        let mut p1_src_img = &nate_fighter_rect;
                        if gameinfo.current_player1 == FighterType::Chloe {
                            p1_src_img = &chloe_fighter_rect;
                        } else if gameinfo.current_player1 == FighterType::Grace {
                            p1_src_img = &grace_fighter_rect; // change tograce later
                        }

                        let mut p2_src_img = &nate_fighter_rect;
                        if gameinfo.current_player2 == FighterType::Chloe {
                            p2_src_img = &chloe_fighter_rect;
                        } else if gameinfo.current_player2 == FighterType::Grace {
                            p2_src_img = &grace_fighter_rect; // change tograce later
                        }

                        vulkan_state
                            .fb2d
                            .bitblt(p1_src_img, &fighter_rect_rect, p1_draw_to);

                        vulkan_state
                            .fb2d
                            .bitblt(p2_src_img, &fighter_rect_rect, p2_draw_to);

                        // p1 labels and bars are all on the upper right corner
                        // maybe we should swap p1 and p2 positions though, could be more intuitive

                        vulkan_state.fb2d.write_to_font(
                            "HP",
                            &mut titlefont,
                            Vec2i {
                                x: hp_draw_to.x - (titlefont_size as i32) * 4,
                                y: (HEIGHT as i32) - (mana_draw_to.y + titlefont_size as i32),
                            },
                            Vec2i {
                                x: (titlefont_size as i32) * 2,
                                y: titlefont_size as i32,
                            },
                        );

                        vulkan_state.fb2d.write_to_font(
                            "MANA",
                            &mut titlefont,
                            Vec2i {
                                x: mana_draw_to.x - (titlefont_size as i32) * 4,
                                y: (HEIGHT as i32) - (hp_draw_to.y + titlefont_size as i32),
                            },
                            Vec2i {
                                x: (titlefont_size as i32) * 4,
                                y: titlefont_size as i32,
                            },
                        );

                        let mut f1_health_rect = engine::image::Rect::new(
                            hp_draw_to.x,
                            HEIGHT as i32 - (mana_draw_to.y + hp_y as i32),
                            p1_initial_health as u32,
                            hp_y,
                        );
                        vulkan_state
                            .fb2d
                            .draw_filled_rect(&mut f1_health_rect, hp_color);
                        let mut f1_mana_rect = engine::image::Rect::new(
                            mana_draw_to.x,
                            HEIGHT as i32 - (hp_draw_to.y + mana_y as i32),
                            p1_initial_mana as u32,
                            mana_y,
                        );
                        vulkan_state
                            .fb2d
                            .draw_filled_rect(&mut f1_mana_rect, mana_color);

                        vulkan_state.fb2d.write_to_font(
                            "HP",
                            &mut titlefont,
                            Vec2i {
                                x: 10,
                                y: hp_draw_to.y - 2,
                            },
                            Vec2i {
                                x: (titlefont_size as i32) * 6,
                                y: titlefont_size as i32,
                            },
                        );

                        vulkan_state.fb2d.write_to_font(
                            "MANA",
                            &mut titlefont,
                            Vec2i {
                                x: 10,
                                y: mana_draw_to.y - 2,
                            },
                            Vec2i {
                                x: (titlefont_size as i32) * 6,
                                y: titlefont_size as i32,
                            },
                        );

                        let mut f2_health_rect = engine::image::Rect::new(
                            10 + (titlefont_size as i32) * 4,
                            hp_draw_to.y,
                            p2_initial_health as u32,
                            hp_y,
                        );
                        vulkan_state
                            .fb2d
                            .draw_filled_rect(&mut f2_health_rect, hp_color);
                        let mut f2_mana_rect = engine::image::Rect::new(
                            10 + (titlefont_size as i32) * 4,
                            mana_draw_to.y,
                            p2_initial_mana as u32,
                            mana_y,
                        );
                        vulkan_state
                            .fb2d
                            .draw_filled_rect(&mut f2_mana_rect, mana_color);

                        // if done, we can go to next move
                        // added another way to go to next move
                        if (now_keys[VirtualKeyCode::Return as usize] && pick_anim_done)
                            || (mouse_click == true && prev_mouse_click == false)
                        {
                            game.state = GameStates::ChooseMove;
                            player1_finish_selecting_move = false;
                            player2_finish_selecting_move = false;
                            gameinfo.player1_current_move = placeholder_fightermove;
                            gameinfo.player2_current_move = placeholder_fightermove;
                            player1_move_selected = false;
                            player2_move_selected = false;
                            pick_anim_done = false;
                            playing_anim = false;
                        }
                    }
                } else if game.state == GameStates::FinalScreen {
                    //play again button

                    vulkan_state
                        .fb2d
                        .bitblt(&wins_text, &header_rect, player_header_rect_draw_to);

                    vulkan_state.fb2d.bitblt(
                        &play_again_button,
                        &fighter_info_rect,
                        next_button_draw_to,
                    );

                    if gameinfo.winning_player == FighterType::Nate {
                        vulkan_state.fb2d.bitblt(
                            &nate_fighter_rect,
                            &fighter_rect_rect,
                            fighter_rect_draw_to_2,
                        );
                    }
                    if gameinfo.winning_player == FighterType::Chloe {
                        vulkan_state.fb2d.bitblt(
                            &chloe_fighter_rect,
                            &fighter_rect_rect,
                            fighter_rect_draw_to_2,
                        );
                    }
                    if gameinfo.winning_player == FighterType::Grace {
                        vulkan_state.fb2d.bitblt(
                            &grace_fighter_rect,
                            &fighter_rect_rect,
                            fighter_rect_draw_to_2,
                        );
                    }
                    if gameinfo.winning_player == FighterType::Both {
                        //game is a draw card drawn in the second bit
                        vulkan_state.fb2d.bitblt(
                            &nate_fighter_rect,
                            &fighter_rect_rect,
                            fighter_rect_draw_to_1,
                        );

                        if gameinfo.current_player1 == FighterType::Nate {
                            vulkan_state.fb2d.bitblt(
                                &nate_fighter_rect,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_1,
                            );
                        }
                        if gameinfo.current_player1 == FighterType::Chloe {
                            vulkan_state.fb2d.bitblt(
                                &chloe_fighter_rect,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_1,
                            );
                        }
                        if gameinfo.current_player1 == FighterType::Grace {
                            vulkan_state.fb2d.bitblt(
                                &grace_fighter_rect,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_1,
                            );
                        }
                        if gameinfo.current_player2 == FighterType::Nate {
                            vulkan_state.fb2d.bitblt(
                                &nate_fighter_rect,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_3,
                            );
                        }
                        if gameinfo.current_player2 == FighterType::Chloe {
                            vulkan_state.fb2d.bitblt(
                                &chloe_fighter_rect,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_3,
                            );
                        }
                        if gameinfo.current_player2 == FighterType::Grace {
                            vulkan_state.fb2d.bitblt(
                                &grace_fighter_rect,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_3,
                            );
                        }
                    }

                    if mouse_click == true && prev_mouse_click == false {
                        let mouse_pos = engine::image::Vec2i {
                            x: mouse_x as i32,
                            y: mouse_y as i32,
                        };

                        if next_button_clickable_rect.rect_inside(mouse_pos) {
                            coin_handle_click.play(InstanceSettings::default());
                            game.state = GameStates::ChooseFighter;
                            //reset all the booleans
                            f1 = Fighter::<FighterType> {
                                name: FighterType::None,
                                is_cpu: false,
                                is_turn: true,
                                health: 100,
                                mana: 20,
                            };
                            f2 = Fighter::<FighterType> {
                                name: FighterType::None,
                                is_cpu: true,
                                is_turn: false,
                                health: 100,
                                mana: 20,
                            };
                            player1_finish_selecting = false;
                            player2_finish_selecting = false;
                            player1_selected = false;
                            player2_selected = false;
                            player1_finish_selecting_move = false;
                            player2_finish_selecting_move = false;
                            player1_move_selected = false;
                            player2_move_selected = false;
                            gameinfo = GameInfo {
                                current_player1: FighterType::None,
                                player1_info: FighterType::None,
                                player1_current_move: placeholder_fightermove,
                                player1_move_info: placeholder_fightermove,
                                current_player2: FighterType::None,
                                player2_info: FighterType::None,
                                player2_current_move: placeholder_fightermove,
                                player2_move_info: placeholder_fightermove,
                                winning_player: FighterType::None,
                            };
                            done_execute_move = false;
                            player_info = FighterType::None;
                            player_info_temp = 0;
                            p1_initial_health = 0;
                            p2_initial_health = 0;
                            p1_initial_mana = 0;
                            p2_initial_mana = 0;
                        }
                    }
                    /*
                    vulkan_state.fb2d.write_to(
                        "GAME OVER",
                        &mut titlefontsheet_sprite,
                        Vec2i{x:20, y:20},
                        titlefont_size,
                        Vec2i{x: WIDTH as i32- 20, y: HEIGHT as i32 - 20}
                    )
                    */
                }

                // if audio_play == true {
                //     arrangement_handle.play(InstanceSettings::default());
                //     audio_play = false;
                // }

                // Update prev_keys and prev_mouse_click to store previous inputs
                prev_keys.copy_from_slice(&now_keys);
                prev_mouse_click = mouse_click;
                {
                    let writable_fb = &mut *vulkan_config.fb2d_buffer.write().unwrap();
                    writable_fb.copy_from_slice(vulkan_state.fb2d.as_slice());
                }
                if vulkan_state.recreate_swapchain {
                    let dimensions: [u32; 2] = vulkan_config.surface.window().inner_size().into();
                    let (new_swapchain, new_images) = match vulkan_state
                        .swapchain
                        .recreate()
                        .dimensions(dimensions)
                        .build()
                    {
                        Ok(r) => r,
                        Err(SwapchainCreationError::UnsupportedDimensions) => return,
                        Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
                    };

                    vulkan_state.swapchain = new_swapchain;
                    vulkan_state.framebuffers = engine::window_size_dependent_setup(
                        &new_images,
                        vulkan_config.render_pass.clone(),
                        &mut vulkan_state.viewport,
                    );
                    vulkan_state.recreate_swapchain = false;
                }
                let (image_num, suboptimal, acquire_future) =
                    match swapchain::acquire_next_image(vulkan_state.swapchain.clone(), None) {
                        Ok(r) => r,
                        Err(AcquireError::OutOfDate) => {
                            vulkan_state.recreate_swapchain = true;
                            return;
                        }
                        Err(e) => panic!("Failed to acquire next image: {:?}", e),
                    };
                if suboptimal {
                    vulkan_state.recreate_swapchain = true;
                }

                // let start = Instant::now();

                let mut builder = AutoCommandBufferBuilder::primary(
                    vulkan_config.device.clone(),
                    vulkan_config.queue.family(),
                    CommandBufferUsage::OneTimeSubmit,
                )
                .unwrap();

                builder
                    // Now copy that framebuffer buffer into the framebuffer image
                    .copy_buffer_to_image(
                        vulkan_config.fb2d_buffer.clone(),
                        vulkan_config.fb2d_image.clone(),
                    )
                    .unwrap()
                    // And resume our regularly scheduled programming
                    .begin_render_pass(
                        vulkan_state.framebuffers[image_num].clone(),
                        SubpassContents::Inline,
                        std::iter::once(vulkano::format::ClearValue::None),
                    )
                    .unwrap()
                    .set_viewport(0, [vulkan_state.viewport.clone()])
                    .bind_pipeline_graphics(vulkan_config.pipeline.clone())
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        vulkan_config.pipeline.layout().clone(),
                        0,
                        vulkan_config.set.clone(),
                    )
                    .bind_vertex_buffers(0, vulkan_config.vertex_buffer.clone())
                    .draw(vulkan_config.vertex_buffer.len() as u32, 1, 0, 0)
                    .unwrap()
                    .end_render_pass()
                    .unwrap();

                let command_buffer = builder.build().unwrap();

                let future = acquire_future
                    .then_execute(vulkan_config.queue.clone(), command_buffer)
                    .unwrap()
                    .then_swapchain_present(
                        vulkan_config.queue.clone(),
                        vulkan_state.swapchain.clone(),
                        image_num,
                    )
                    .then_signal_fence_and_flush();
                // dbg!(start.elapsed());

                match future {
                    Ok(future) => {
                        vulkan_state.previous_frame_end = Some(future.boxed());
                    }
                    Err(FlushError::OutOfDate) => {
                        vulkan_state.recreate_swapchain = true;
                        vulkan_state.previous_frame_end =
                            Some(sync::now(vulkan_config.device.clone()).boxed());
                    }
                    Err(e) => {
                        println!("Failed to flush future: {:?}", e);
                        vulkan_state.previous_frame_end =
                            Some(sync::now(vulkan_config.device.clone()).boxed());
                    }
                }
            }
            _ => (),
        }
    });
}
