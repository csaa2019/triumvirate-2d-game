// // Export all engine types at the top level
pub mod animation;
pub mod image;

use crate::image::Image;
use std::rc::Rc;
use std::sync::Arc;
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
// mod types;
// pub use types::*;
// mod engine;
// pub use engine::Engine;

// pub mod render;
// pub mod input;
// mod util;

const WIDTH: usize = 320;
const HEIGHT: usize = 240;

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

pub struct FighterStrings<'a> {
    pub name: &'a str,
    pub desc: &'a str,
    pub move1_name: &'a str,
    pub move1_desc: &'a str,
    pub move2_name: &'a str,
    pub move2_desc: &'a str,
    pub move3_name: &'a str,
    pub move3_desc: &'a str,
}

pub struct Fighter<T: Copy + Eq + PartialEq> {
    pub name: T,
    pub is_cpu: bool,
    pub is_turn: bool,
    pub health: i32,
    pub mana: i32,
}

impl<T: Copy + Eq + PartialEq> Fighter<T> {
    pub fn new(name: T, is_cpu: bool, is_turn: bool) -> Fighter<T> {
        Fighter {
            name: name,
            is_cpu: is_cpu,
            is_turn,
            health: 100,
            mana: 100,
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
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub struct FighterMove<T: Copy + Eq + PartialEq> {
    // add string field with description? Was getting stuck with copy trait doe -Nate
    pub fighter_move_type: T,
    //damage to the other player, positive value
    pub damage: i32,
    //cost of player's mana, negative value
    pub mana_cost: i32,
    //cost of "health" positive value means it adds to players health (regenerative moves)
    //negative health would be just the case that a move takes away from player health
    pub health_cost: i32,
    // amount of mana generated by move (in order to save up for big attack)
    pub mana_generation: i32,
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

    FighterInfo,

    //choose which move from each player
    ChooseMove,

    MoveInfo,
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

pub fn window_size_dependent_setup(
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

#[derive(Default, Debug, Clone)]
pub struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}
vulkano::impl_vertex!(Vertex, position, uv);

// stuff which doesn't change from frame to frame
// Arc: enables a type to go across threads, read only
pub struct VulkanConfig {
    pub surface: Arc<vulkano::swapchain::Surface<winit::window::Window>>,
    pub device: Arc<vulkano::device::Device>,
    pub set: Arc<vulkano::descriptor_set::PersistentDescriptorSet>,
    pub pipeline: Arc<vulkano::pipeline::GraphicsPipeline>,
    pub vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    pub fb2d_image: Arc<StorageImage>,
    pub fb2d_buffer: Arc<CpuAccessibleBuffer<[image::Color]>>,
    pub queue: Arc<vulkano::device::Queue>,
    pub render_pass: Arc<vulkano::render_pass::RenderPass>,
}

// stuff which does change
pub struct VulkanState {
    pub fb2d: image::Image,
    pub swapchain: Arc<Swapchain<winit::window::Window>>,
    pub viewport: Viewport,
    pub framebuffers: Vec<Arc<vulkano::render_pass::Framebuffer>>,
    pub recreate_swapchain: bool,
    pub previous_frame_end: Option<Box<dyn vulkano::sync::GpuFuture>>,
}

pub fn vulkan_init(event_loop: &EventLoop<()>) -> (VulkanConfig, VulkanState) {
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
    let fb2d = image::Image::new((0, 0, 0, 0), WIDTH, HEIGHT);
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
