use engine::*;

use rand;
use rand::Rng;
use std::env;
use std::fmt::{self, Display, Formatter};
use std::io;

use std::io::Cursor;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;
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

fn main() {
    let required_extensions = vulkano_win::required_extensions();
    let instance = Instance::new(None, Version::V1_1, &required_extensions, None).unwrap();
    let event_loop = EventLoop::new();
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
    let (mut swapchain, images) = {
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
    #[derive(Default, Debug, Clone)]
    struct Vertex {
        position: [f32; 2],
        uv: [f32; 2],
    }
    vulkano::impl_vertex!(Vertex, position, uv);

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
    let mut fb2d = engine::image::Image::new((0, 0, 0, 0), WIDTH, HEIGHT);
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

    let mut framebuffers = window_size_dependent_setup(&images, render_pass.clone(), &mut viewport);
    let mut recreate_swapchain = false;
    let mut previous_frame_end = Some(sync::now(device.clone()).boxed());

    use std::io;
    // We need the Write trait so we can flush stdout
    use std::io::Write;

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

    //Game initialization testing - GRACE
    let mut game = Game {
        state: GameStates::Instructions,
    };

    //SPRITE STUFF
    //initialize the instruction sheet image
    //we don't technically need it to be an animation sprite at all, just need a bitblt image LOL
    let instruction_img_width = 350;
    let instruction_img_height = 245;
    let instruction_sprite_w = 350;
    let instruction_sprite_h = 245;
    let instruction_sprite_rect =
        engine::image::Rect::new(0, 0, instruction_sprite_w, instruction_sprite_h);
    let instruction_sheet_rect =
        engine::image::Rect::new(0, 0, instruction_img_width, instruction_img_height);
    let instruction_sheet = engine::image::Image::from_png(
        "game-1/content/Instruction-Screen.png",
        instruction_img_width,
        instruction_img_height,
    );
    let instruction_anim_state = engine::animation::AnimationState {
        current_frame: 0,
        elapsed_time: 0,
        animation_index: 0,
    };
    let instruction_animation = engine::animation::Animation {
        frames: vec![0],
        frame_duration: 5,
        loops: true,
        sprite_size: instruction_sprite_rect,
        sprite_width: 1,
        sprite_height: 1,
    };
    let mut instruction_sprite = engine::animation::Sprite {
        image: Rc::new(instruction_sheet),
        animations: vec![instruction_animation],
        animation_state: instruction_anim_state,
    };

    //initialize the scissor animation
    let img_width = 1470;
    let img_height = 840;
    let sprite_h = 210;
    let sprite_w = 210;
    let mut scissor_sprite_rect = engine::image::Rect::new(0, 0, sprite_w, sprite_h);

    let mut scissor_sheet_rect = engine::image::Rect::new(0, 0, img_width, img_height);

    let scissor_sheet =
        engine::image::Image::from_png("game-1/content/scissor.png", img_width, img_height);

    let scissor_anim_state = engine::animation::AnimationState {
        current_frame: 0,
        elapsed_time: 0,
        animation_index: 1,
    };

    let scissor_animation_snip_left = engine::animation::Animation {
        frames: vec![0, 1, 2, 3, 4, 5, 6],
        frame_duration: 5,
        loops: true,
        sprite_size: scissor_sprite_rect,
        sprite_width: 7,
        sprite_height: 4,
    };

    let scissor_animation_snip_right = engine::animation::Animation {
        frames: vec![14, 15, 16, 17, 18, 19, 20],
        frame_duration: 5,
        loops: true,
        sprite_size: scissor_sprite_rect,
        sprite_width: 7,
        sprite_height: 4,
    };

    let scissor_animation_spin = engine::animation::Animation {
        frames: vec![7, 8, 9, 10, 11, 12, 13, 21, 22, 23, 24, 25, 26, 27],
        frame_duration: 5,
        loops: true,
        sprite_size: scissor_sprite_rect,
        sprite_width: 7,
        sprite_height: 4,
    };

    let mut scissor_sprite = engine::animation::Sprite {
        image: Rc::new(scissor_sheet),
        animations: vec![
            scissor_animation_snip_left,
            scissor_animation_snip_right,
            scissor_animation_spin,
        ],
        animation_state: scissor_anim_state,
    };

    let mut playing_anim = false;

    // KEYBOARD INPUT STUFF

    let mut now_keys = [false; 255];
    let mut prev_keys = now_keys.clone();

    // MOUSE INPUT STUFF
    let mut mouse_x = 0.0;
    let mut mouse_y = 0.0;

    let mut mouse_click = 0;

    // GAME STUFF
    let mut p1 = Player::<RPSType>::new("Joe Schmo".to_string(), false, true);
    let mut p2 = Player::<RPSType>::new("Boss playa".to_string(), true, false);
    let mut round = 0; // the round the player is on, out of 3.
    let mut score = (0, 0); // (player score, AI score)

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
                recreate_swapchain = true;
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
                mouse_x = x;
                mouse_y = y;
            }

            // button is not being used here but later if we want to specify
            // left click vs right click, we'll want to use it
            Event::WindowEvent {
                // Note this deeply nested pattern match
                event: WindowEvent::MouseInput { state, button, .. },
                ..
            } => match state {
                winit::event::ElementState::Pressed => {
                    mouse_click = 1;
                }
                winit::event::ElementState::Released => {
                    mouse_click = 0;
                }
            },

            // NewEvents: Let's start processing events.
            Event::NewEvents(_) => {
                // Leave now_keys alone, but copy over all changed keys
                prev_keys.copy_from_slice(&now_keys);
            }

            Event::MainEventsCleared => {
                {
                    // We need to synchronize here to send new data to the GPU.
                    // We can't send the new framebuffer until the previous frame is done being drawn.
                    // Dropping the future will block until it's done.
                    if let Some(mut fut) = previous_frame_end.take() {
                        fut.cleanup_finished();
                    }
                }

                if now_keys[VirtualKeyCode::Escape as usize] {
                    *control_flow = ControlFlow::Exit;
                }

                // if mouse_click == 1 {
                //     println!("Mouse clicked at {} {}", mouse_x, mouse_y);
                // }

                //choose background color, I made it white
                fb2d.clear((255_u8, 255_u8, 255_u8, 255_u8));

                //create the game state that creates the screen for the instruction screen
                if game.state == GameStates::Instructions {
                    if !playing_anim {
                        instruction_sprite.play_animation(&mut fb2d, 0);
                        playing_anim = true;
                    } else {
                        instruction_sprite.tick_animation();
                        instruction_sprite.draw(&mut fb2d);
                    }

                    // bitblt an image that is the instructions
                    // bitblt an image that says play
                    // if play is clicked, change the game state
                    // }
                    //if we are in the intro state, then do this
                }

                if mouse_click == 1 {
                    game.state = GameStates::ShowPick;
                }

                if game.state == GameStates::ShowPick {
                    if !playing_anim {
                        scissor_sprite.play_animation(&mut fb2d, 0);
                        playing_anim = true;
                    } else {
                        scissor_sprite.tick_animation();
                        scissor_sprite.draw(&mut fb2d);
                    }
                }

                let mut player_move = None;
                if mouse_click == 1 {
                    if mouse_x > 0.0 && mouse_x < 250.0 {
                        player_move = Some(moves[0]);
                    }
                    if mouse_x > 250.0 && mouse_x < 500.0 {
                        player_move = Some(moves[1]);
                    }
                    if mouse_x > 500.0 && mouse_x < 797.0 {
                        player_move = Some(moves[2]);
                    }
                }
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
                    }
                    else if result == engine::Outcomes::Lose {
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
                }

                {
                    let writable_fb = &mut *fb2d_buffer.write().unwrap();
                    writable_fb.copy_from_slice(fb2d.as_slice());
                }
                if recreate_swapchain {
                    let dimensions: [u32; 2] = surface.window().inner_size().into();
                    let (new_swapchain, new_images) =
                        match swapchain.recreate().dimensions(dimensions).build() {
                            Ok(r) => r,
                            Err(SwapchainCreationError::UnsupportedDimensions) => return,
                            Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
                        };

                    swapchain = new_swapchain;
                    framebuffers = window_size_dependent_setup(
                        &new_images,
                        render_pass.clone(),
                        &mut viewport,
                    );
                    recreate_swapchain = false;
                }
                let (image_num, suboptimal, acquire_future) =
                    match swapchain::acquire_next_image(swapchain.clone(), None) {
                        Ok(r) => r,
                        Err(AcquireError::OutOfDate) => {
                            recreate_swapchain = true;
                            return;
                        }
                        Err(e) => panic!("Failed to acquire next image: {:?}", e),
                    };
                if suboptimal {
                    recreate_swapchain = true;
                }

                // let start = Instant::now();

                let mut builder = AutoCommandBufferBuilder::primary(
                    device.clone(),
                    queue.family(),
                    CommandBufferUsage::OneTimeSubmit,
                )
                .unwrap();

                builder
                    // Now copy that framebuffer buffer into the framebuffer image
                    .copy_buffer_to_image(fb2d_buffer.clone(), fb2d_image.clone())
                    .unwrap()
                    // And resume our regularly scheduled programming
                    .begin_render_pass(
                        framebuffers[image_num].clone(),
                        SubpassContents::Inline,
                        std::iter::once(vulkano::format::ClearValue::None),
                    )
                    .unwrap()
                    .set_viewport(0, [viewport.clone()])
                    .bind_pipeline_graphics(pipeline.clone())
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        pipeline.layout().clone(),
                        0,
                        set.clone(),
                    )
                    .bind_vertex_buffers(0, vertex_buffer.clone())
                    .draw(vertex_buffer.len() as u32, 1, 0, 0)
                    .unwrap()
                    .end_render_pass()
                    .unwrap();

                let command_buffer = builder.build().unwrap();

                let future = acquire_future
                    .then_execute(queue.clone(), command_buffer)
                    .unwrap()
                    .then_swapchain_present(queue.clone(), swapchain.clone(), image_num)
                    .then_signal_fence_and_flush();
                // dbg!(start.elapsed());

                match future {
                    Ok(future) => {
                        previous_frame_end = Some(future.boxed());
                    }
                    Err(FlushError::OutOfDate) => {
                        recreate_swapchain = true;
                        previous_frame_end = Some(sync::now(device.clone()).boxed());
                    }
                    Err(e) => {
                        println!("Failed to flush future: {:?}", e);
                        previous_frame_end = Some(sync::now(device.clone()).boxed());
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