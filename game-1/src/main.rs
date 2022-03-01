use engine::image::{Image, Rect, Vec2i};

use engine::*;

use rand;
use rand::Rng;
// use std::env;
// use std::fmt::{self, Display, Formatter};
// use std::io;
// use std::io::Cursor;
use kira::arrangement::{Arrangement, LoopArrangementSettings};
use kira::instance::InstanceSettings;
use kira::manager::{AudioManager, AudioManagerSettings};
use kira::sound::SoundSettings;
use std::rc::Rc;
use std::sync::Arc;
// use std::time::Instant;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents};
use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::{Device, DeviceExtensions, Features};
use vulkano::format::Format;
use vulkano::image::ImageCreateFlags;
use vulkano::image::{
    view::ImageView, ImageAccess, ImageDimensions, ImageUsage, ImmutableImage, MipmapsCount,
    StorageImage, SwapchainImage,
};
use vulkano::instance::Instance;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint};
use vulkano::render_pass::{Framebuffer, RenderPass, Subpass};
use vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode};
use vulkano::swapchain::{self, AcquireError, Swapchain, SwapchainCreationError};
use vulkano::sync::{self, FlushError, GpuFuture};
use vulkano::Version;
use vulkano_win::VkSurfaceBuild;
use winit::event::{Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

//we would have RPSTypes and Outcomes in the main of the rock paper scissors main.rs
//need to derive clone, copy, partialeq, eq, debug
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
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

const WIDTH: usize = 320;
const HEIGHT: usize = 240;

// something from zulip -Grace 2/14/2021
// A way to use structs to organize the stuff that we have in that big fn main
// would this go in our main.rs of the game-1?

//this could live in the engine instead
//vulkan.rs or lib.rs add that in
//input.rs --> just movce it all into lib.rs and start separating it out

// struct Input { now_keys:[bool;255], prev_keys:[bool;255], ... }
// impl Input { pub fn is_key_down(&self, key:VirtualKeyCode) -> bool { ... }  ... }
// struct VulkanConfig { instance, physical_device, device, queues, swapchain, images,... }
// struct VulkanState { render_pass, framebuffers, last_frame_future, next_image, ... }
// struct FBState { pipeline,  vertex_buffer, fb2d_buffer, fb2d_image, fb2d_texture, fb2d_sampler, set, fb2d }

// from class on 2/22: organizing vulkan stuff as its own stuff

#[derive(Default, Debug, Clone)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}
vulkano::impl_vertex!(Vertex, position, uv);

// stuff which doesn't change from frame to frame
// Arc: enables a type to go across threads, read only
struct VulkanConfig {
    surface: Arc<vulkano::swapchain::Surface<winit::window::Window>>,
    device: Arc<vulkano::device::Device>,
    set: Arc<vulkano::descriptor_set::PersistentDescriptorSet>,
    pipeline: Arc<vulkano::pipeline::GraphicsPipeline>,
    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    fb2d_image: Arc<StorageImage>,
    fb2d_buffer: Arc<CpuAccessibleBuffer<[engine::image::Color]>>,
    queue: Arc<vulkano::device::Queue>,
    render_pass: Arc<vulkano::render_pass::RenderPass>,
}

// stuff which does change
struct VulkanState {
    fb2d: engine::image::Image,
    swapchain: Arc<Swapchain<winit::window::Window>>,
    viewport: Viewport,
    framebuffers: Vec<Arc<vulkano::render_pass::Framebuffer>>,
    recreate_swapchain: bool,
    previous_frame_end: Option<Box<dyn vulkano::sync::GpuFuture>>,
}

fn vulkan_init(event_loop: &EventLoop<()>) -> (VulkanConfig, VulkanState) {
    let required_extensions = vulkano_win::required_extensions();
    let instance = Instance::new(None, Version::V1_1, &required_extensions, None).unwrap();

    let surface = WindowBuilder::new()
        .build_vk_surface(&event_loop, instance.clone())
        .unwrap();
    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::none()
    };
    let (physical_device, queue_family) = PhysicalDevice::enumerate(&instance)
        .filter(|&p| p.supported_extensions().is_superset_of(&device_extensions))
        .filter_map(|p| {
            p.queue_families()
                .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
                .map(|q| (p, q))
        })
        .min_by_key(|(p, _)| match p.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,
            PhysicalDeviceType::Other => 4,
        })
        .unwrap();
    let (device, mut queues) = Device::new(
        physical_device,
        &Features::none(),
        &physical_device
            .required_extensions()
            .union(&device_extensions),
        [(queue_family, 0.5)].iter().cloned(),
    )
    .unwrap();
    let queue = queues.next().unwrap();
    let (swapchain, images) = {
        let caps = surface.capabilities(physical_device).unwrap();
        // let present_mode = best_present_mode(&caps);
        let composite_alpha = caps.supported_composite_alpha.iter().next().unwrap();
        let format = caps.supported_formats[0].0;
        let dimensions: [u32; 2] = surface.window().inner_size().into();
        Swapchain::start(device.clone(), surface.clone())
            .num_images(caps.min_image_count)
            .format(format)
            .dimensions(dimensions)
            .usage(ImageUsage::color_attachment())
            .sharing_mode(&queue)
            .composite_alpha(composite_alpha)
            // .present_mode(present_mode)
            .build()
            .unwrap()
    };

    // We now create a buffer that will store the shape of our triangle.
    let vertex_buffer = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        false,
        [
            Vertex {
                position: [-1.0, -1.0],
                uv: [0.0, 0.0],
            },
            Vertex {
                position: [3.0, -1.0],
                uv: [2.0, 0.0],
            },
            Vertex {
                position: [-1.0, 3.0],
                uv: [0.0, 2.0],
            },
        ]
        .iter()
        .cloned(),
    )
    .unwrap();

    mod vs {
        vulkano_shaders::shader! {
            ty: "vertex",
            src: "
                #version 450

                layout(location = 0) in vec2 position;
                layout(location = 1) in vec2 uv;
                layout(location = 0) out vec2 out_uv;
                void main() {
                    gl_Position = vec4(position, 0.0, 1.0);
                    out_uv = uv;
                }
            "
        }
    }

    mod fs {
        vulkano_shaders::shader! {
            ty: "fragment",
            src: "
                #version 450

                layout(set = 0, binding = 0) uniform sampler2D tex;
                layout(location = 0) in vec2 uv;
                layout(location = 0) out vec4 f_color;

                void main() {
                    f_color = texture(tex, uv);
                }
            "
        }
    }

    let vs = vs::load(device.clone()).unwrap();
    let fs = fs::load(device.clone()).unwrap();

    // Here's our (2D drawing) framebuffer.
    let fb2d = engine::image::Image::new((0, 0, 0, 0), WIDTH, HEIGHT);
    // We'll work on it locally, and copy it to a GPU buffer every frame.
    // Then on the GPU, we'll copy it into an Image.
    let fb2d_buffer = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::transfer_source(),
        false,
        (0..WIDTH * HEIGHT).map(|_| (255_u8, 0_u8, 0_u8, 0_u8)),
    )
    .unwrap();
    // Let's set up the Image we'll copy into:
    let dimensions = ImageDimensions::Dim2d {
        width: WIDTH as u32,
        height: HEIGHT as u32,
        array_layers: 1,
    };
    let fb2d_image = StorageImage::with_usage(
        device.clone(),
        dimensions,
        Format::R8G8B8A8_UNORM,
        ImageUsage {
            // This part is key!
            transfer_destination: true,
            sampled: true,
            storage: true,
            transfer_source: false,
            color_attachment: false,
            depth_stencil_attachment: false,
            transient_attachment: false,
            input_attachment: false,
        },
        ImageCreateFlags::default(),
        std::iter::once(queue_family),
    )
    .unwrap();
    // Get a view on it to use as a texture:
    let fb2d_texture = ImageView::new(fb2d_image.clone()).unwrap();

    let fb2d_sampler = Sampler::new(
        device.clone(),
        Filter::Linear,
        Filter::Linear,
        MipmapMode::Nearest,
        SamplerAddressMode::Repeat,
        SamplerAddressMode::Repeat,
        SamplerAddressMode::Repeat,
        0.0,
        1.0,
        0.0,
        0.0,
    )
    .unwrap();

    let render_pass = vulkano::single_pass_renderpass!(
        device.clone(),
        attachments: {
            color: {
                // Pro move: We're going to cover the screen completely. Trust us!
                load: DontCare,
                store: Store,
                format: swapchain.format(),
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {}
        }
    )
    .unwrap();

    let pipeline = GraphicsPipeline::start()
        .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
        .vertex_shader(vs.entry_point("main").unwrap(), ())
        .input_assembly_state(InputAssemblyState::new())
        .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
        .fragment_shader(fs.entry_point("main").unwrap(), ())
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        .build(device.clone())
        .unwrap();
    let layout = pipeline.layout().descriptor_set_layouts().get(0).unwrap();
    let mut set_builder = PersistentDescriptorSet::start(layout.clone());

    set_builder
        .add_sampled_image(fb2d_texture, fb2d_sampler)
        .unwrap();

    let set = set_builder.build().unwrap();

    let mut viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [0.0, 0.0],
        depth_range: 0.0..1.0,
    };

    let framebuffers = window_size_dependent_setup(&images, render_pass.clone(), &mut viewport);
    let recreate_swapchain = false;
    let previous_frame_end = Some(sync::now(device.clone()).boxed());

    (
        VulkanConfig {
            surface,
            device,
            set,
            pipeline,
            vertex_buffer,
            fb2d_image,
            fb2d_buffer,
            queue,
            render_pass,
        },
        VulkanState {
            fb2d,
            swapchain,
            viewport,
            framebuffers,
            recreate_swapchain,
            previous_frame_end,
        },
    )
}

// consider Assets struct: contains image, colors, etc.
// consider update function and "go" function??

fn create_move_sprite(
    img_width: u32,
    img_height: u32,
    filename: &str,
) -> engine::animation::Sprite {
    //initialize the scissor animation

    let sprite_w = img_width / 2;
    let sprite_h = img_height / 2;

    // the rectangle of one sprite
    let sprite_rect = engine::image::Rect::new(0, 0, sprite_w, sprite_h);

    let sheet = engine::image::Image::from_png(filename, img_width, img_height);

    let anim_state = engine::animation::AnimationState {
        current_frame: 0,
        elapsed_time: 0,
        animation_index: 0,
    };

    let anim_1 = engine::animation::Animation {
        frames: vec![0, 1, 2, 3],
        frame_duration: 10,
        loops: true,
        sprite_size: sprite_rect,
        sprite_width: 2,
        sprite_height: 2,
    };

    engine::animation::Sprite {
        image: Rc::new(sheet),
        animations: vec![anim_1],
        animation_state: anim_state,
    }
}

fn main() {
    /*
    Stuff during initialization (once)
    Stuff after window resize
    Stuff during each event loop

    Stuff each call/draw operation
    */

    // Load audio
    let mut audio_manager = AudioManager::new(AudioManagerSettings::default()).unwrap();
    let mut sound_handle_music = audio_manager
        .load_sound(
            "game-1/content/RPS_tunes_loop.ogg",
            SoundSettings::default(),
        )
        .unwrap();
    let mut arrangement_handle = audio_manager
        .add_arrangement(Arrangement::new_loop(
            &sound_handle_music,
            LoopArrangementSettings::default(),
        ))
        .unwrap();
    let mut sound_handle_click = audio_manager
        .load_sound("game-1/content/click.ogg", SoundSettings::default())
        .unwrap();

    let event_loop = EventLoop::new();
    let (vulkan_config, mut vulkan_state) = vulkan_init(&event_loop);

    // consider moving this vec? maybe use a rps_moves() fx
    let moves = [
        Move {
            move_type: RPSType::Rock,
            wins: RPSType::Scissors,
            loses: RPSType::Paper,
        },
        Move {
            move_type: RPSType::Scissors,
            wins: RPSType::Paper,
            loses: RPSType::Rock,
        },
        Move {
            move_type: RPSType::Paper,
            wins: RPSType::Rock,
            loses: RPSType::Scissors,
        },
    ];

    // IMAGES + SPRITES
    // consider putting all initializing sprite stuff in its own function?

    //GameState Main Screen Assets
    //Title Text image
    let text_title_w = 238;
    let text_title_h = 60;
    let text_title_rect = engine::image::Rect::new(0, 0, text_title_w, text_title_h);
    let text_title = engine::image::Image::from_png(
        "game-1/content/ROCK-PAPER-SCISSORS.png",
        text_title_w,
        text_title_h,
    );

    //make it in the center of the screen
    let text_title_draw_to = engine::image::Vec2i { x: 40, y: 20 };

    //Press Play image
    //365 x 89, scaled down to like 0.3
    let text_play_w = 100 as u32;
    let text_play_h = 18 as u32;
    let text_play_rect = engine::image::Rect::new(0, 0, text_play_w, text_play_h);
    let text_play =
        engine::image::Image::from_png("game-1/content/PLAY.png", text_play_w, text_play_h);

    let text_play_draw_to = engine::image::Vec2i { x: 110, y: 150 };

    let text_play_clickable_rect = engine::image::Rect::new(
        text_play_draw_to.x,
        text_play_draw_to.y,
        text_play_w,
        text_play_h,
    );

    // to scale down, change img_width and height and change score widthth and height variables below
    let score_image = engine::image::Image::from_png("game-1/content/score.png", 70, 24);

    //GameState Instruction Assets
    // Instruction sheet image
    let instruction_w = 350;
    let instruction_h = 245;
    let instruction_rect = engine::image::Rect::new(0, 0, instruction_w, instruction_h);

    let instruction_img = engine::image::Image::from_png(
        "game-1/content/Instruction-Screen.png",
        instruction_w,
        instruction_h,
    );

    let instruction_draw_to = engine::image::Vec2i { x: 5, y: 5 };

    // SCISSOR SPRITE
    let scissor_img_width = 136;
    let scissor_img_height = 200;
    let scissor_sprite_w = scissor_img_width / 2;
    let scissor_sprite_h = scissor_img_height / 2;

    let mut scissor_sprite = create_move_sprite(
        scissor_img_width,
        scissor_img_height,
        "game-1/content/scissors-ss.png",
    );

    let draw_y = HEIGHT as i32 - scissor_sprite_h as i32 - 10;
    // coordinates to draw to
    let scissor_draw_to = engine::image::Vec2i {
        x: WIDTH as i32 / 2 - ((scissor_sprite_w as i32) / 2),
        y: draw_y,
    };

    let scissor_clickable_rect = engine::image::Rect::new(
        scissor_draw_to.x,
        scissor_draw_to.y,
        scissor_sprite_w,
        scissor_sprite_h,
    );

    // ROCK SPRITE

    let rock_img_width = 177;
    let rock_img_height = 200;
    let rock_sprite_w = 88;
    let rock_sprite_h = 100;

    let mut rock_sprite = create_move_sprite(
        rock_img_width,
        rock_img_height,
        "game-1/content/rock-ss.png",
    );

    // coordinates to draw to
    let rock_draw_to = engine::image::Vec2i {
        x: (WIDTH as i32 / 6) - (rock_sprite_w as i32 / 2),
        y: draw_y,
    };

    let rock_clickable_rect =
        engine::image::Rect::new(rock_draw_to.x, rock_draw_to.y, rock_sprite_w, rock_sprite_h);

    // PAPER SPRITE

    let paper_img_width = 200;
    let paper_img_height = 204;
    let paper_sprite_w = paper_img_width / 2;
    let paper_sprite_h = paper_img_height / 2;

    let mut paper_sprite = create_move_sprite(
        paper_img_width,
        paper_img_height,
        "game-1/content/paper-ss.png",
    );

    let paper_draw_to = engine::image::Vec2i {
        x: (WIDTH as i32 / 3 * 2),
        y: draw_y,
    };

    let paper_clickable_rect = engine::image::Rect::new(
        paper_draw_to.x,
        paper_draw_to.y,
        paper_sprite_w,
        paper_sprite_h,
    );

    //images for gamestate: final screen

    //You Lose! text
    //828 x 89
    let text_youlose_w = 166 as u32;
    let text_youlose_h = 18 as u32;
    let text_youlose_rect = engine::image::Rect::new(0, 0, text_youlose_w, text_youlose_h);
    let text_youlose = engine::image::Image::from_png(
        "game-1/content/YOU-LOSE.png",
        text_youlose_w,
        text_youlose_h,
    );

    let text_youlose_draw_to = engine::image::Vec2i { x: 80, y: 30 };

    //You Win! text
    //828 x 89 at 0.2 scale
    let text_youwin_w = 166 as u32;
    let text_youwin_h = 18 as u32;
    let text_youwin_rect = engine::image::Rect::new(0, 0, text_youwin_w, text_youwin_h);
    let text_youwin =
        engine::image::Image::from_png("game-1/content/YOU-WIN.png", text_youwin_w, text_youwin_h);

    let text_youwin_draw_to = engine::image::Vec2i { x: 80, y: 30 };

    //Play Again text
    //951x89 at 0.2 scale
    let text_playagain_w = 190 as u32;
    let text_playagain_h = 18 as u32;
    let text_playagain_rect = engine::image::Rect::new(0, 0, text_playagain_w, text_playagain_h);
    let text_playagain = engine::image::Image::from_png(
        "game-1/content/PLAY-AGAIN.png",
        text_playagain_w,
        text_playagain_h,
    );

    let text_playagain_draw_to = engine::image::Vec2i { x: 60, y: 150 };

    let text_playagain_clickable_rect = engine::image::Rect::new(
        text_playagain_draw_to.x,
        text_playagain_draw_to.y,
        text_playagain_w,
        text_playagain_h,
    );
    let countdown_img_width = 300;
    let countdown_img_height = 300;
    let countdown_sprite_w = 150;
    let countdown_sprite_h = 150;
    // the rectangle of one sprite
    let countdown_sprite_rect =
        engine::image::Rect::new(0, 0, countdown_sprite_w, countdown_sprite_h);

    // NEED TO CHANGE
    let countdown_sheet = engine::image::Image::from_png(
        "game-1/content/countdown ss.png",
        countdown_img_width,
        countdown_img_height,
    );

    let countdown_anim_state = engine::animation::AnimationState {
        current_frame: 0,
        elapsed_time: 0,
        animation_index: 0,
    };

    let countdown_anim_1 = engine::animation::Animation {
        frames: vec![0, 1, 2],
        frame_duration: 20,
        loops: true,
        sprite_size: countdown_sprite_rect,
        sprite_width: 2,
        sprite_height: 2,
    };

    let mut countdown_sprite = engine::animation::Sprite {
        image: Rc::new(countdown_sheet),
        animations: vec![countdown_anim_1],
        animation_state: countdown_anim_state,
    };

    // coordinates to draw to
    let countdown_draw_to = engine::image::Vec2i {
        x: (WIDTH as i32 - countdown_sprite_w as i32) / 2,
        y: 10,
    };

    let mut playing_anim = false;
    let mut countdown_playing_anim = false;
    let mut countdown_timer = 0;

    // KEYBOARD INPUT STUFF
    let mut now_keys = [false; 255];
    let mut prev_keys = now_keys.clone();

    // MOUSE INPUT STUFF
    let mut mouse_x = 0.0;
    let mut mouse_y = 0.0;

    let mut mouse_click = false;
    let mut prev_mouse_click = false;

    // GAME STUFF
    let mut game = Game {
        state: GameStates::MainScreen,
    };
    let mut p1 = Player::<RPSType>::new("Joe Schmo".to_string(), false, true);
    let mut p2 = Player::<RPSType>::new("Boss playa".to_string(), true, false);
    let mut round = 0; // the round the player is on, out of 3.
    let mut score = (0, 0); // (player score, AI score)
    let mut player_move = None;
    let mut audio_play = true;

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
                        position: winit::dpi::PhysicalPosition { x: x, y: y, .. },
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

                //choose background color, I made it white
                vulkan_state.fb2d.clear((255_u8, 255_u8, 255_u8, 255_u8));

                if audio_play == true {
                    arrangement_handle.play(InstanceSettings::default());
                    audio_play = false;
                }

                //MAINSCREEN game state
                if game.state == GameStates::MainScreen {
                    vulkan_state
                        .fb2d
                        .bitblt(&text_title, &text_title_rect, text_title_draw_to);
                    vulkan_state
                        .fb2d
                        .bitblt(&text_play, &text_play_rect, text_play_draw_to);

                    if mouse_click == true && prev_mouse_click == false {
                        sound_handle_click.play(InstanceSettings::default());
                        let mouse_pos = engine::image::Vec2i {
                            x: mouse_x as i32,
                            y: mouse_y as i32,
                        };

                        if text_play_clickable_rect.rect_inside(mouse_pos) {
                            game.state = GameStates::Instructions;
                        }
                    }
                }
                //INSTRUCTIONS gamestate
                else if game.state == GameStates::Instructions {
                    vulkan_state.fb2d.bitblt(
                        &instruction_img,
                        &instruction_rect,
                        instruction_draw_to,
                    );

                    //if they click anywhere in the screen then move onto showPick
                    if mouse_click == true && prev_mouse_click == false {
                        sound_handle_click.play(InstanceSettings::default());
                        game.state = GameStates::PlayerPicking;
                    }
                }
                //SHOWPICK gamestate
                else if game.state == GameStates::PlayerPicking {
                    // resetting player move so it doesn't just keep
                    // thinking the player has a move
                    player_move = None;

                    let mouse_pos = engine::image::Vec2i {
                        x: mouse_x as i32,
                        y: mouse_y as i32,
                    };

                    // Draw score image in top right
                    let score_width = 70;
                    let score_height = 24;
                    vulkan_state.fb2d.bitblt(
                        &score_image,
                        &engine::image::Rect::new(0, 0, score_width, score_height),
                        engine::image::Vec2i { x: 235, y: 5 },
                    );

                    if !playing_anim {
                        scissor_sprite.play_animation(&mut vulkan_state.fb2d, scissor_draw_to);
                        rock_sprite.play_animation(&mut vulkan_state.fb2d, rock_draw_to);
                        paper_sprite.play_animation(&mut vulkan_state.fb2d, paper_draw_to);

                        playing_anim = true;
                    } else {
                        if rock_clickable_rect.rect_inside(mouse_pos) {
                            rock_sprite.tick_animation();
                        }

                        if scissor_clickable_rect.rect_inside(mouse_pos) {
                            scissor_sprite.tick_animation();
                        }

                        if paper_clickable_rect.rect_inside(mouse_pos) {
                            paper_sprite.tick_animation();
                        }

                        rock_sprite.draw(&mut vulkan_state.fb2d, rock_draw_to);
                        scissor_sprite.draw(&mut vulkan_state.fb2d, scissor_draw_to);
                        paper_sprite.draw(&mut vulkan_state.fb2d, paper_draw_to);
                    }

                    if mouse_click == true && prev_mouse_click == false {
                        sound_handle_click.play(InstanceSettings::default());
                        let mouse_pos = engine::image::Vec2i {
                            x: mouse_x as i32,
                            y: mouse_y as i32,
                        };

                        if rock_clickable_rect.rect_inside(mouse_pos) {
                            print!("Clicked the rock rect");
                            player_move = Some(moves[0]);
                        }

                        if scissor_clickable_rect.rect_inside(mouse_pos) {
                            print!("Clicked the scissor rect");
                            player_move = Some(moves[1]);
                        }

                        if paper_clickable_rect.rect_inside(mouse_pos) {
                            print!("Clicked the paper rect");
                            player_move = Some(moves[2]);
                        }

                        //check if the round number is < 3
                        //else: Increase round number here and change game state to countdown

                        // change code below for other clickable elements

                        // if mouse_x > 0.0 && mouse_x < 250.0 {
                        //     player_move = Some(moves[0]);
                        // }
                        // if mouse_x > 250.0 && mouse_x < 500.0 {
                        //     player_move = Some(moves[1]);
                        // }
                        // if mouse_x > 500.0 && mouse_x < 797.0 {
                        //     player_move = Some(moves[2]);
                        // }
                    }

                    // right now this is detecting when a mouse click is held
                    // so if you keep holding on scissors, then it will keep choosing scissors as a move
                    // we might need some other game - state
                    // like instead of choose pick we move to processing move, or something like that

                    if player_move.is_some() {
                        println!("Player move: {:?}", player_move.unwrap().move_type);
                        p1.set_current_move(player_move.unwrap());
                        // Random AI move
                        let mut rng = rand::thread_rng();
                        let ai_move = moves[rng.gen_range(0, 3)];
                        println!("AI move: {:?}", ai_move.move_type);
                        p2.set_current_move(ai_move);
                        let result = p1.execute_move(&p2);
                        if result == engine::Outcomes::Win {
                            score.0 += 1;
                            println!("Player Wins");
                        } else if result == engine::Outcomes::Lose {
                            score.1 += 1;
                            println!("AI Wins");
                        }
                        if score.0 == 3 {
                            println!("Player wins best of 3!");
                            score = (0, 0);
                        }
                        if score.1 == 3 {
                            println!("AI wins best of 3!");
                            score = (0, 0);
                        }
                        println!();
                        game.state = GameStates::Countdown;
                    }
                } else if game.state == GameStates::Countdown {
                    if countdown_timer >= 60 {
                        countdown_timer = 0;
                        countdown_playing_anim = false;
                        game.state = GameStates::ShowPick;
                    } else {
                        countdown_timer += 1;

                        if !countdown_playing_anim {
                            print!("is this running? {} \n", countdown_playing_anim);
                            countdown_sprite
                                .play_animation(&mut vulkan_state.fb2d, countdown_draw_to);
                            countdown_playing_anim = true;
                        } else {
                            countdown_sprite.tick_animation();
                            countdown_sprite.draw(&mut vulkan_state.fb2d, countdown_draw_to);
                        }
                    }
                } else if game.state == GameStates::ShowPick {
                    if countdown_timer >= 60 {
                        countdown_timer = 0;
                        countdown_playing_anim = false;
                        game.state = GameStates::PlayerPicking;
                    } else {
                        let player_draw_to = Vec2i {
                            x: (WIDTH as i32 / 5),
                            y: HEIGHT as i32 / 2,
                        };

                        let enemy_draw_to = Vec2i {
                            x: (WIDTH as i32 / 5) * 3,
                            y: 10,
                        };

                        vulkan_state
                            .fb2d
                            .line((1, 1), (WIDTH - 1, HEIGHT - 1), (0, 0, 0, 1));

                        countdown_timer += 1;

                        if !countdown_playing_anim {
                            countdown_playing_anim = true;

                            if p1.current_move == Some(moves[0]) {
                                rock_sprite.play_animation(&mut vulkan_state.fb2d, player_draw_to);
                            } else if p1.current_move == Some(moves[1]) {
                                scissor_sprite
                                    .play_animation(&mut vulkan_state.fb2d, player_draw_to);
                            } else if p1.current_move == Some(moves[2]) {
                                paper_sprite.play_animation(&mut vulkan_state.fb2d, player_draw_to);
                            }

                            if p2.current_move == Some(moves[0]) {
                                rock_sprite.play_animation(&mut vulkan_state.fb2d, enemy_draw_to);
                            } else if p2.current_move == Some(moves[1]) {
                                scissor_sprite
                                    .play_animation(&mut vulkan_state.fb2d, enemy_draw_to);
                            } else if p2.current_move == Some(moves[2]) {
                                paper_sprite.play_animation(&mut vulkan_state.fb2d, enemy_draw_to);
                            }
                        } else {
                            rock_sprite.tick_animation();
                            scissor_sprite.tick_animation();
                            paper_sprite.tick_animation();

                            if p1.current_move == Some(moves[0]) {
                                rock_sprite.draw(&mut vulkan_state.fb2d, player_draw_to);
                            } else if p1.current_move == Some(moves[1]) {
                                scissor_sprite.draw(&mut vulkan_state.fb2d, player_draw_to);
                            } else if p1.current_move == Some(moves[2]) {
                                paper_sprite.draw(&mut vulkan_state.fb2d, player_draw_to);
                            }

                            if p2.current_move == Some(moves[0]) {
                                rock_sprite.draw(&mut vulkan_state.fb2d, enemy_draw_to);
                            } else if p2.current_move == Some(moves[1]) {
                                scissor_sprite.draw(&mut vulkan_state.fb2d, enemy_draw_to);
                            } else if p2.current_move == Some(moves[2]) {
                                paper_sprite.draw(&mut vulkan_state.fb2d, enemy_draw_to);
                            }
                        }
                    }
                }
                //GameState Countdown
                //just have an animation that goes 3 2 1 and then moves on
                else if game.state == GameStates::FinalScreen {
                    //blit the play again at the bottom of the screen
                    vulkan_state.fb2d.bitblt(
                        &text_playagain,
                        &text_playagain_rect,
                        text_playagain_draw_to,
                    );

                    //if player score is greater than enemy score bitblt this
                    //waiting for the variable names for score

                    vulkan_state
                        .fb2d
                        .bitblt(&text_youwin, &text_youwin_rect, text_youwin_draw_to);

                    /*
                    //else bitblit this
                    vulkan_state.fb2d.bitblt(
                        &text_youlose,
                        &text_youlose_rect,
                        text_youlose_draw_to,
                    );
                    */

                    if mouse_click == true && prev_mouse_click == false {
                        sound_handle_click.play(InstanceSettings::default());
                        let mouse_pos = engine::image::Vec2i {
                            x: mouse_x as i32,
                            y: mouse_y as i32,
                        };

                        //we actually want it to be playerpicking
                        if text_playagain_clickable_rect.rect_inside(mouse_pos) {
                            game.state = GameStates::PlayerPicking;
                        }
                    }

                    //
                }
                //GameState Final Screen
                //if they win: image
                //if they lose: image
                //along with a "play again"

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
                    vulkan_state.framebuffers = window_size_dependent_setup(
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

fn window_size_dependent_setup(
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<RenderPass>,
    viewport: &mut Viewport,
) -> Vec<Arc<Framebuffer>> {
    let dimensions = images[0].dimensions().width_height();
    viewport.dimensions = [dimensions[0] as f32, dimensions[1] as f32];

    images
        .iter()
        .map(|image| {
            let view = ImageView::new(image.clone()).unwrap();
            Framebuffer::start(render_pass.clone())
                .add(view)
                .unwrap()
                .build()
                .unwrap()
        })
        .collect::<Vec<_>>()
}
