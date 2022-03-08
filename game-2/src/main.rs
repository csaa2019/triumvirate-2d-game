use engine::image::{Image, Rect, Vec2i};
use engine::*;

// use std::env;
// use std::fmt::{self, Display, Formatter};
// use std::io;
// use std::io::Cursor;
use kira::arrangement::{Arrangement, LoopArrangementSettings};
use kira::instance::InstanceSettings;
use kira::manager::{AudioManager, AudioManagerSettings};
use kira::sound::SoundSettings;
use std::ptr::null;
use rand;
use rand::Rng;
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

const WIDTH: usize = 320;
const HEIGHT: usize = 240;

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

fn main() {
    /*
    Stuff during initialization (once)
    Stuff after window resize
    Stuff during each event loop

    Stuff each call/draw operation
    */

    // Load audio
    // let mut audio_manager = AudioManager::new(AudioManagerSettings::default()).unwrap();
    // let mut sound_handle_music = audio_manager
    //     .load_sound("content/RPS_tunes_loop.ogg", SoundSettings::default())
    //     .unwrap();
    // let mut arrangement_handle = audio_manager
    //     .add_arrangement(Arrangement::new_loop(
    //         &sound_handle_music,
    //         LoopArrangementSettings::default(),
    //     ))
    //     .unwrap();
    // let mut sound_handle_click = audio_manager
    //     .load_sound("content/click.ogg", SoundSettings::default())
    //     .unwrap();

    let event_loop = EventLoop::new();
    let (vulkan_config, mut vulkan_state) = vulkan_init(&event_loop);

    let mut playing_anim = false;

    //Image stuff

    let fontsize = (7 as f32 * 1.5) as u32;
    let fontsheet_w = 16 * fontsize;
    let fontsheet_h = 8 * fontsize;

    // the rectangle of one sprite
    let font_sprite_rect = engine::image::Rect::new(0, 0, fontsize, fontsize);

    let fontsheet = engine::image::Image::from_png_not_premultiplied(
        "content/fontsheet_7x7.png",
        fontsheet_w,
        fontsheet_h,
    );

    let font_anim_state = engine::animation::AnimationState {
        current_frame: 0,
        elapsed_time: 0,
        animation_index: 0,
    };

    let font_anim_1 = engine::animation::Animation {
        frames: (0..128).collect(),
        frame_duration: 10,
        loops: true,
        sprite_size: font_sprite_rect,
        sprite_width: 16,
        sprite_height: 8,
    };

    let mut fontsheet_sprite = engine::animation::Sprite {
        image: Rc::new(fontsheet),
        animations: vec![font_anim_1],
        animation_state: font_anim_state,
    };
    let titlefont_size = (20 as f32 * 1.5) as u32;

    let mut description_box_dim = Vec2i { x: 200, y: HEIGHT as i32 - (titlefont_size as i32+ 10)};
    let mut description_draw_to = Vec2i {
        x: ((WIDTH as i32) / 2) - (description_box_dim.x / 2),
        y: titlefont_size as i32+ 10,
    };

    let titlefontsheet_w = 16 * titlefont_size;
    let titlefontsheet_h = 8 * titlefont_size;

    // the rectangle of one sprite
    let titlefont_sprite_rect = engine::image::Rect::new(0, 0, titlefont_size, titlefont_size);

    let titlefontsheet = engine::image::Image::from_png_not_premultiplied(
        "content/fontsheet_70x70.png",
        titlefontsheet_w,
        titlefontsheet_h,
    );

    let titlefont_anim_state = engine::animation::AnimationState {
        current_frame: 0,
        elapsed_time: 0,
        animation_index: 0,
    };

    let titlefont_anim_1 = engine::animation::Animation {
        frames: (0..128).collect(),
        frame_duration: 10,
        loops: true,
        sprite_size: titlefont_sprite_rect,
        sprite_width: 16,
        sprite_height: 8,
    };

    let mut titlefontsheet_sprite = engine::animation::Sprite {
        image: Rc::new(titlefontsheet),
        animations: vec![titlefont_anim_1],
        animation_state: titlefont_anim_state,
    };

    let mut title_box_dim = Vec2i { x: 200, y: titlefont_size as i32 + 10};
    let mut title_draw_to = Vec2i {
        x: ((WIDTH as i32) / 2) - (title_box_dim.x / 2),
        y: 10,
    };

    //GameState::ChooseFighter
    let header_rect_w = 80 as u32;
    let header_rect_h = 10 as u32;
    let header_rect_rect = engine::image::Rect::new(0, 0, header_rect_w, header_rect_h);
    let header_player1_rect = engine::image::Image::from_png(
        "content/player1-text.png",
        header_rect_w,
        header_rect_h,
    );

    let header_player2_rect = engine::image::Image::from_png(
        "content/player2-text.png",
        header_rect_w,
        header_rect_h,
    );

    let header_rect_draw_to = engine::image::Vec2i { x: 120, y: 12 };

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

    let grace_fighter_rect  = engine::image::Image::from_png_not_premultiplied(
        "content/gracefighter.png",
        fighter_rect_w,
        fighter_rect_h,
    );



    //a draw to for each chloe, nate, grace
    //for ease lets have 1 as chloe, 2 as nate, 3 as grace
    let fighter_rect_draw_to_1 = engine::image::Vec2i { x: 15, y: 35 };
    let fighter_rect_draw_to_2 = engine::image::Vec2i { x: 115, y: 35 };
    let fighter_rect_draw_to_3 = engine::image::Vec2i { x: 215, y: 35 };

    let p1_draw_to = engine::image::Vec2i { x: 215, y: 10 };
    let p2_draw_to = engine::image::Vec2i { x: 15, y: HEIGHT as i32 - (fighter_rect_h as i32 + 10)};


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

    //a draw to for each chloe, nate, grace
    //for ease lets have 1 as chloe, 2 as nate, 3 as grace
    let fighter_info_draw_to_1 = engine::image::Vec2i { x: 15, y: 170 };
    let fighter_info_draw_to_2 = engine::image::Vec2i { x: 115, y: 170 };
    let fighter_info_draw_to_3 = engine::image::Vec2i { x: 215, y: 170 };
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
        GraceMove3
    }

    //random move names here
    let grace_fighter_moves = vec![
        FighterMove {
            // drink yerma mate -- Wake up!
            fighter_move_type: FighterMoveType::GraceMove1, 
            damage: 0,
            mana_cost: 0,
            health_cost: 0,
            mana_generation: 20,
        },
        FighterMove{
            fighter_move_type: FighterMoveType::GraceMove2, 
            damage: -20,
            mana_cost: -20,
            health_cost: 0,
            mana_generation: 0,
        },
        FighterMove{
            fighter_move_type: FighterMoveType::GraceMove3, 
            damage: -50,
            mana_cost: 30,
            health_cost: 0,
            mana_generation: 0,
        },
    ];

    let chloe_fighter_moves = vec![
        FighterMove {
            fighter_move_type: FighterMoveType::ChloeMove1, 
            damage: 0,
            mana_cost: 0,
            health_cost: 5,
            mana_generation: 15,
        },
        FighterMove{
            // 
            fighter_move_type: FighterMoveType::ChloeMove2, 
            damage: -10,
            mana_cost: -10,
            health_cost: 0,
            mana_generation: 0,
        },
        FighterMove{
            // chloe just hacked the simulation and deleted yo ass -- fatal
            fighter_move_type: FighterMoveType::ChloeMove3,
            damage: -100,
            mana_cost: 69,
            health_cost: 0,
            mana_generation: 20,
        },
    ];

    let nate_fighter_moves = vec![
        FighterMove {
            // take a chill pill -- feel the chillness coursing through your bloodstream
            fighter_move_type: FighterMoveType::NateMove1, 
            damage: 0,
            mana_cost: 0,
            health_cost: 0,
            mana_generation: 20,
        },
        FighterMove{
            // Face melting guitar solo -- just like a microwave
            fighter_move_type: FighterMoveType::NateMove2,
            damage: -20,
            mana_cost: -20,
            health_cost: 0,
            mana_generation: 0,
        },
        FighterMove{
            // toss some yo -- flex some badass tricks and then BAM!
            fighter_move_type: FighterMoveType::NateMove3,
            damage: 55,
            mana_cost: -40,
            health_cost: 0,
            mana_generation: 0,
        },
    ];

    let placeholder_fightermove = FighterMove{
        fighter_move_type: FighterMoveType::None,
        damage: -69,
        mana_cost: -69,
        health_cost: 69,
        mana_generation: 0
    }; 

    // this is a stand in variable to test hp bar animation
    let hp_y = 20;
    let hp_draw_to= engine::image::Vec2i{x: (WIDTH as i32) - 100 - 10, y:10};
    let hp_color = (255, 0, 0, 1); 

    let mana_y = 20; 
    let mana_draw_to = engine::image::Vec2i{x: (WIDTH as i32) - 100 - 10, y:40};
    let mana_color = (0, 255, 0 , 1); 

    let pick_fps = 10;
    let mut pick_frame_count = 0;
    let mut pick_anim_playing = false;
    let mut pick_anim_done = false;



    /*
    let mut chloe = Fighter::new(FighterType::Chloe, false, true);
    let mut grace = Fighter::new(FighterType::Grace, false, true);
    let mut nate = Fighter::new(FighterType::Nate, false, true);
    */

    let mut back_button_rect = engine::image::Rect::new(10, 10, 20, 30);
    // GAME STUFF
    let mut game = Game {
        // state: GameStates::ChooseFighter,
        state: GameStates::ChooseFighter, // i'm testing the hp bar here
    };

    pub struct GameInfo{ 
        pub current_player1: FighterType, 
        pub player1_info: FighterType, 
        pub player1_current_move: FighterMove<FighterMoveType>, 
        pub player1_move_info: FighterMove<FighterMoveType>, 
        pub current_player2: FighterType, 
        pub player2_info: FighterType, 
        pub player2_current_move: FighterMove<FighterMoveType>, 
        pub player2_move_info:FighterMove<FighterMoveType>, 
    }

    impl GameInfo { 
        pub fn new(current_player1:FighterType, player1_info:FighterType, player1_current_move: FighterMove<FighterMoveType>, player1_move_info: FighterMove<FighterMoveType>, current_player2:FighterType, player2_info:FighterType, player2_current_move: FighterMove<FighterMoveType>, player2_move_info: FighterMove<FighterMoveType>)
        {
            GameInfo {current_player1: current_player1, player1_info: player1_info, player1_current_move: player1_current_move, player1_move_info: player1_move_info,
            current_player2: current_player2, player2_info: player2_info, player2_current_move: player2_current_move, player2_move_info: player2_move_info}; 
        }
    }

    //initializing the players here. would we want this as a multiplayer game? Would that be possible? 
    //aka eliminate the iscpu screen 
    let mut f1 = Fighter::<FighterType> {name: FighterType::None, is_cpu:false, is_turn: true, health: 100, mana: 20}; 
    let mut f2 = Fighter::<FighterType> {name: FighterType::None, is_cpu:true,  is_turn: false, health: 100, mana: 20}; 
    let mut player1_finish_selecting = false; 
    let mut player2_finish_selecting = false; 
    let mut player1_selected = false;
    let mut player2_selected = false; 
    let mut player1_finish_selecting_move = false; 
    let mut player2_finish_selecting_move = false; 
    let mut player1_move_selected = false; 
    let mut player2_move_selected = false; 
    let mut gameinfo = GameInfo {current_player1: FighterType::None, player1_info: FighterType::None, player1_current_move: placeholder_fightermove, player1_move_info: placeholder_fightermove, 
        current_player2: FighterType::None, player2_info: FighterType::None, player2_current_move: placeholder_fightermove, player2_move_info: placeholder_fightermove}; 
    let mut done_execute_move = false;
    //
    let player_info = FighterType::None;

    let mut player_info_temp = 0;

    let mut p1_initial_health = 0;
    let mut p2_initial_health = 0;
    let mut p1_initial_mana = 0;
    let mut p2_initial_mana = 0;


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


                if game.state == GameStates::MainScreen { 

                    //then move main screen into instructions 
                }

                else if game.state == GameStates::Instructions {
                    //then move instructions to choose fighter 

                }

                else if game.state == GameStates::ChooseFighter {
                    
                    if !player1_finish_selecting
                    {
                        if player1_selected
                        {
                            if gameinfo.current_player1 == FighterType::Nate{
                                vulkan_state.fb2d.bitblt(
                                    &highlight_rect,
                                    &highlight_rect_rect,
                                    highlight_rect_draw_to_1,
                                );
                            }

                            if gameinfo.current_player1 == FighterType::Chloe{
                                vulkan_state.fb2d.bitblt(
                                    &highlight_rect,
                                    &highlight_rect_rect,
                                    highlight_rect_draw_to_2,
                                );
                            }

                            if gameinfo.current_player1 == FighterType::Grace{
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
                            &header_player1_rect,
                            &header_rect_rect,
                            header_rect_draw_to,
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
                                gameinfo.current_player1 = FighterType::Nate; 
                            }

                            if fighter_rect_clickable_rect_2.rect_inside(mouse_pos) {
                                player1_selected = true;
                                gameinfo.current_player1 = FighterType::Chloe; 

                            }

                            if fighter_rect_clickable_rect_3.rect_inside(mouse_pos) {
                                player1_selected = true;
                                gameinfo.current_player1 = FighterType::Grace; 

                            }

                            if fighter_info_clickable_rect_1.rect_inside(mouse_pos) {
                                gameinfo.player1_info = FighterType::Nate;
                                game.state = GameStates::FighterInfo;
                            }

                            if fighter_info_clickable_rect_2.rect_inside(mouse_pos) {
                                gameinfo.player1_info = FighterType::Chloe;
                                game.state = GameStates::FighterInfo;
                            }

                            if fighter_info_clickable_rect_3.rect_inside(mouse_pos) {
                                gameinfo.player1_info = FighterType::Grace;
                                game.state = GameStates::FighterInfo;
                            }

                            if next_button_clickable_rect.rect_inside(mouse_pos) && player1_selected {
                                player1_finish_selecting = true; 
                            }
                        }
                    
                    }

                    else if !player2_finish_selecting
                    {
                        //add highlight to the boxes 
                        if player2_selected
                        {
                            if gameinfo.current_player2 == FighterType::Nate{
                                vulkan_state.fb2d.bitblt(
                                    &highlight_rect,
                                    &highlight_rect_rect,
                                    highlight_rect_draw_to_1,
                                );
                            }

                            if gameinfo.current_player2 == FighterType::Chloe{
                                vulkan_state.fb2d.bitblt(
                                    &highlight_rect,
                                    &highlight_rect_rect,
                                    highlight_rect_draw_to_2,
                                );
                            }

                            if gameinfo.current_player2 == FighterType::Grace{
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
                            &header_player2_rect,
                            &header_rect_rect,
                            header_rect_draw_to,
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
                                gameinfo.current_player2 = FighterType::Nate; 

                            }

                            if fighter_rect_clickable_rect_2.rect_inside(mouse_pos) {
                                player2_selected = true;
                                gameinfo.current_player2 = FighterType::Chloe; 
                            }

                            if fighter_rect_clickable_rect_3.rect_inside(mouse_pos) {
                                player2_selected = true;
                                gameinfo.current_player2 = FighterType::Grace; 
                            }

                            if fighter_info_clickable_rect_1.rect_inside(mouse_pos) {
                                gameinfo.player2_info = FighterType::Nate;
                                game.state = GameStates::FighterInfo;
                            }

                            if fighter_info_clickable_rect_2.rect_inside(mouse_pos) {
                                gameinfo.player2_info = FighterType::Chloe;
                                game.state = GameStates::FighterInfo;
                            }

                            if fighter_info_clickable_rect_3.rect_inside(mouse_pos) {
                                gameinfo.player2_info = FighterType::Grace;
                                game.state = GameStates::FighterInfo;
                            }

                            if next_button_clickable_rect.rect_inside(mouse_pos) && player2_selected {
                                player2_finish_selecting = true; 
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

                    vulkan_state.fb2d.draw_filled_rect(&mut back_button_rect, (155, 252, 232, 1));

                    if mouse_click == true && prev_mouse_click == false {
                        let mouse_pos = engine::image::Vec2i {
                            x: mouse_x as i32,
                            y: mouse_y as i32,
                        };

                        if back_button_rect.rect_inside(mouse_pos){
                            game.state = GameStates::ChooseFighter;
                        }

                    }

                    //if player_info == nate, bit blit certain images
                    if gameinfo.player1_info == FighterType::Nate {
                        vulkan_state.fb2d.write_to(
                            "NATE",
                            &mut titlefontsheet_sprite,
                            title_draw_to,
                            titlefont_size,
                            title_box_dim,
                        );
                        vulkan_state.fb2d.write_to(
                            "let's pretend there is a description about our characters on the screen or something\
                            i need a really long string to test this text width thing. \
                            we can also use this to test character descriptions later. Lorem ipsum dolor sit amet, \
                            consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
                            &mut fontsheet_sprite,
                            description_draw_to,
                            fontsize,
                            description_box_dim
                        );
                    } 
                    
                    //if player_info == chloe, bit blit certain images
                    if gameinfo.player1_info == FighterType::Chloe {
                        vulkan_state.fb2d.write_to(
                            "CHLOE",
                            &mut titlefontsheet_sprite,
                            title_draw_to,
                            titlefont_size,
                            title_box_dim,
                        );
                        vulkan_state.fb2d.write_to(
                            "let's pretend there is a description about our characters on the screen or something\
                            i need a really long string to test this text width thing. \
                            we can also use this to test character descriptions later. Lorem ipsum dolor sit amet, \
                            consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
                            &mut fontsheet_sprite,
                            description_draw_to,
                            fontsize,
                            description_box_dim
                        );
                    }
                    //if player_info == grace, bit blit certain images into the rectangles
                    if gameinfo.player1_info == FighterType::Grace {
                        vulkan_state.fb2d.write_to(
                            "GRACE",
                            &mut titlefontsheet_sprite,
                            title_draw_to,
                            titlefont_size,
                            title_box_dim,
                        );
                        vulkan_state.fb2d.write_to(
                            "let's pretend there is a description about our characters on the screen or something\
                            i need a really long string to test this text width thing. \
                            we can also use this to test character descriptions later. Lorem ipsum dolor sit amet, \
                            consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
                            &mut fontsheet_sprite,
                            description_draw_to,
                            fontsize,
                            description_box_dim
                        );
                    }

                }
                //gamestate::choosemove
                else if game.state == GameStates::ChooseMove {
                    done_execute_move = false;

                    if !player1_finish_selecting_move{
                
                        if gameinfo.current_player1 == FighterType::Nate{
                            vulkan_state.fb2d.clear((242_u8, 0_u8, 255_u8, 100_u8));

                            //add highlight rectangle outside the selected move 
                            if player1_move_selected{
                                if gameinfo.player1_current_move.fighter_move_type == FighterMoveType::NateMove1{
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_1,
                                    );
                                }

                                if gameinfo.player1_current_move.fighter_move_type == FighterMoveType::NateMove2{
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_2,
                                    );
                                }

                                if gameinfo.player1_current_move.fighter_move_type == FighterMoveType::NateMove3{
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_3,
                                    );
                                }
                                
                        }
                            //nate move 1
                            vulkan_state.fb2d.bitblt(
                                &fighter_rect,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_1,
                            );

                            //nate move 2
                            vulkan_state.fb2d.bitblt(
                                &fighter_rect,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_2,
                            );

                            //nate move 3
                            vulkan_state.fb2d.bitblt(
                                &fighter_rect,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_3,
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
        
                                //select the first move
                                if fighter_rect_clickable_rect_1.rect_inside(mouse_pos) {
                                    player1_move_selected = true;
                                    gameinfo.player1_current_move = nate_fighter_moves[0];
                                }
        
                                //select the second move
                                if fighter_rect_clickable_rect_2.rect_inside(mouse_pos) {
                                    player1_move_selected = true;
                                    gameinfo.player1_current_move = nate_fighter_moves[1]; 
                                }
        
                                //select the third move 
                                if fighter_rect_clickable_rect_3.rect_inside(mouse_pos) {
                                    player1_move_selected = true;
                                    gameinfo.player1_current_move = nate_fighter_moves[2]; 
                                }
        
                                //info on first move
                                if fighter_info_clickable_rect_1.rect_inside(mouse_pos) {
                                    gameinfo.player1_move_info = nate_fighter_moves[0]; 
                                    game.state = GameStates::MoveInfo;
                                }
        
                                //info on second move
                                if fighter_info_clickable_rect_2.rect_inside(mouse_pos) {
                                    gameinfo.player1_move_info = nate_fighter_moves[1]; 
                                    game.state = GameStates::MoveInfo;
                                }
        
                                //info on third move
                                if fighter_info_clickable_rect_3.rect_inside(mouse_pos) {
                                    gameinfo.player1_move_info = nate_fighter_moves[2]; 
                                    game.state = GameStates::MoveInfo;
                                }
        
                                //select move and go to next state
                                if next_button_clickable_rect.rect_inside(mouse_pos) && player1_move_selected {
                                    player1_finish_selecting_move = true; 
                                }
                            }
                        }
                        if gameinfo.current_player1 == FighterType::Chloe{
                            vulkan_state.fb2d.clear((255_u8, 242_u8, 0_u8, 100_u8));
                            if player1_selected{
                                if gameinfo.player1_current_move.fighter_move_type == FighterMoveType::ChloeMove1{
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_1,
                                    );
                                }

                                if gameinfo.player1_current_move.fighter_move_type == FighterMoveType::ChloeMove2{
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_2,
                                    );
                                }

                                if gameinfo.player1_current_move.fighter_move_type == FighterMoveType::ChloeMove3{
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_3,
                                    );
                                }
                                
                        }
                            //chloemove1
                            vulkan_state.fb2d.bitblt(
                                &fighter_rect,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_1,
                            );

                            //chloemove2
                            vulkan_state.fb2d.bitblt(
                                &fighter_rect,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_2,
                            );

                            //chloemove3
                            vulkan_state.fb2d.bitblt(
                                &fighter_rect,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_3,
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
        
                                //select the first move
                                if fighter_rect_clickable_rect_1.rect_inside(mouse_pos) {
                                    player1_move_selected = true;
                                    gameinfo.player1_current_move = chloe_fighter_moves[0];
                                }
        
                                //select the second move
                                if fighter_rect_clickable_rect_2.rect_inside(mouse_pos) {
                                    player1_move_selected = true;
                                    gameinfo.player1_current_move = chloe_fighter_moves[1]; 
                                }
        
                                //select the third move 
                                if fighter_rect_clickable_rect_3.rect_inside(mouse_pos) {
                                    player1_move_selected = true;
                                    gameinfo.player1_current_move = chloe_fighter_moves[2]; 
                                }
        
                                //info on first move
                                if fighter_info_clickable_rect_1.rect_inside(mouse_pos) {
                                    gameinfo.player1_move_info = chloe_fighter_moves[0]; 
                                    game.state = GameStates::MoveInfo;
                                }
        
                                //info on second move
                                if fighter_info_clickable_rect_2.rect_inside(mouse_pos) {
                                    gameinfo.player1_move_info = chloe_fighter_moves[1]; 
                                    game.state = GameStates::MoveInfo;
                                }
        
                                //info on third move
                                if fighter_info_clickable_rect_3.rect_inside(mouse_pos) {
                                    gameinfo.player1_move_info = chloe_fighter_moves[2]; 
                                    game.state = GameStates::MoveInfo;
                                }
        
                                //select move and go to next state
                                if next_button_clickable_rect.rect_inside(mouse_pos) && player1_move_selected {
                                    player1_finish_selecting_move = true; 
                                }
                            }
                        }
                        if gameinfo.current_player1 == FighterType::Grace{
                            vulkan_state.fb2d.clear((0_u8, 255_u8, 242_u8, 100_u8));
                            if player2_selected{
                                if gameinfo.player1_current_move.fighter_move_type == FighterMoveType::NateMove1{
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_1,
                                    );
                                }

                                if gameinfo.player1_current_move.fighter_move_type == FighterMoveType::NateMove2{
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_2,
                                    );
                                }

                                if gameinfo.player1_current_move.fighter_move_type == FighterMoveType::NateMove3{
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_3,
                                    );
                                }
                                
                        }
                            //grace move 1
                            vulkan_state.fb2d.bitblt(
                                &fighter_rect,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_1,
                            );

                            //grace move 2
                            vulkan_state.fb2d.bitblt(
                                &fighter_rect,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_2,
                            );

                            //grace move 3
                            vulkan_state.fb2d.bitblt(
                                &fighter_rect,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_3,
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
        
                                //select the first move
                                if fighter_rect_clickable_rect_1.rect_inside(mouse_pos) {
                                    player1_move_selected = true;
                                    gameinfo.player1_current_move = nate_fighter_moves[0];
                                }
        
                                //select the second move
                                if fighter_rect_clickable_rect_2.rect_inside(mouse_pos) {
                                    player1_move_selected = true;
                                    gameinfo.player1_current_move = nate_fighter_moves[1]; 
                                }
        
                                //select the third move 
                                if fighter_rect_clickable_rect_3.rect_inside(mouse_pos) {
                                    player1_move_selected = true;
                                    gameinfo.player1_current_move = nate_fighter_moves[2]; 
                                }
        
                                //info on first move
                                if fighter_info_clickable_rect_1.rect_inside(mouse_pos) {
                                    gameinfo.player1_move_info = grace_fighter_moves[0]; 
                                    game.state = GameStates::MoveInfo;
                                }
        
                                //info on second move
                                if fighter_info_clickable_rect_2.rect_inside(mouse_pos) {
                                    gameinfo.player1_move_info = nate_fighter_moves[1]; 
                                    game.state = GameStates::MoveInfo;
                                }
        
                                //info on third move
                                if fighter_info_clickable_rect_3.rect_inside(mouse_pos) {
                                    gameinfo.player1_move_info = grace_fighter_moves[2]; 
                                    game.state = GameStates::MoveInfo;
                                }
        
                                //select move and go to next state
                                if next_button_clickable_rect.rect_inside(mouse_pos) && player1_move_selected {
                                    player1_finish_selecting_move = true; 
                                }
                            }
                        }
                    }

                    else if !player2_finish_selecting_move
                    {
                        if gameinfo.current_player2 == FighterType::Nate{
                            vulkan_state.fb2d.clear((242_u8, 0_u8, 255_u8, 100_u8));

                            //add highlight rectangle outside the selected move 
                            if player2_move_selected{
                                if gameinfo.player2_current_move.fighter_move_type == FighterMoveType::NateMove1{
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_1,
                                    );
                                }

                                if gameinfo.player2_current_move.fighter_move_type == FighterMoveType::NateMove2{
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_2,
                                    );
                                }

                                if gameinfo.player2_current_move.fighter_move_type == FighterMoveType::NateMove3{
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_3,
                                    );
                                }
                                
                        }
                            //nate move 1
                            vulkan_state.fb2d.bitblt(
                                &fighter_rect,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_1,
                            );

                            //nate move 2
                            vulkan_state.fb2d.bitblt(
                                &fighter_rect,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_2,
                            );

                            //nate move 3
                            vulkan_state.fb2d.bitblt(
                                &fighter_rect,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_3,
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
        
                                // // ENEMY SELECT MOVE (need to figure out random move)
                                // let mut rng = rand::thread_rng();
                                // let ai_move = moves[rng.gen_range(0, 3)];

                                //select the first move
                                if fighter_rect_clickable_rect_1.rect_inside(mouse_pos) {
                                    player2_move_selected = true;
                                    gameinfo.player2_current_move = nate_fighter_moves[0];
                                }
        
                                //select the second move
                                if fighter_rect_clickable_rect_2.rect_inside(mouse_pos) {
                                    player2_move_selected = true;
                                    gameinfo.player2_current_move = nate_fighter_moves[1]; 
                                }
        
                                //select the third move 
                                if fighter_rect_clickable_rect_3.rect_inside(mouse_pos) {
                                    player2_move_selected = true;
                                    gameinfo.player2_current_move = nate_fighter_moves[2]; 
                                }
        
                                //info on first move
                                if fighter_info_clickable_rect_1.rect_inside(mouse_pos) {
                                    gameinfo.player2_move_info = nate_fighter_moves[0]; 
                                    game.state = GameStates::MoveInfo;
                                }
        
                                //info on second move
                                if fighter_info_clickable_rect_2.rect_inside(mouse_pos) {
                                    gameinfo.player2_move_info = nate_fighter_moves[1]; 
                                    game.state = GameStates::MoveInfo;
                                }
        
                                //info on third move
                                if fighter_info_clickable_rect_3.rect_inside(mouse_pos) {
                                    gameinfo.player2_move_info = nate_fighter_moves[2]; 
                                    game.state = GameStates::MoveInfo;
                                }
        
                                //select move and go to next state
                                if next_button_clickable_rect.rect_inside(mouse_pos) && player2_move_selected {
                                    player2_finish_selecting_move = true; 
                                    game.state = GameStates::ShowPick;
                                }
                            }
                        }
                        if gameinfo.current_player2 == FighterType::Chloe{
                            vulkan_state.fb2d.clear((255_u8, 242_u8, 0_u8, 100_u8));
                            if player2_selected{
                                if gameinfo.player2_current_move.fighter_move_type == FighterMoveType::ChloeMove1{
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_1,
                                    );
                                }

                                if gameinfo.player2_current_move.fighter_move_type == FighterMoveType::ChloeMove2{
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_2,
                                    );
                                }

                                if gameinfo.player2_current_move.fighter_move_type == FighterMoveType::ChloeMove3{
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_3,
                                    );
                                }
                                
                        }
                            //chloemove1
                            vulkan_state.fb2d.bitblt(
                                &fighter_rect,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_1,
                            );

                            //chloemove2
                            vulkan_state.fb2d.bitblt(
                                &fighter_rect,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_2,
                            );

                            //chloemove3
                            vulkan_state.fb2d.bitblt(
                                &fighter_rect,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_3,
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
        
                                //select the first move
                                if fighter_rect_clickable_rect_1.rect_inside(mouse_pos) {
                                    player2_move_selected = true;
                                    gameinfo.player2_current_move = chloe_fighter_moves[0];
                                }
        
                                //select the second move
                                if fighter_rect_clickable_rect_2.rect_inside(mouse_pos) {
                                    player2_move_selected = true;
                                    gameinfo.player2_current_move = chloe_fighter_moves[1]; 
                                }
        
                                //select the third move 
                                if fighter_rect_clickable_rect_3.rect_inside(mouse_pos) {
                                    player2_move_selected = true;
                                    gameinfo.player2_current_move = chloe_fighter_moves[2]; 
                                }
        
                                //info on first move
                                if fighter_info_clickable_rect_1.rect_inside(mouse_pos) {
                                    gameinfo.player2_move_info = chloe_fighter_moves[0]; 
                                    game.state = GameStates::MoveInfo;
                                }
        
                                //info on second move
                                if fighter_info_clickable_rect_2.rect_inside(mouse_pos) {
                                    gameinfo.player2_move_info = chloe_fighter_moves[1]; 
                                    game.state = GameStates::MoveInfo;
                                }
        
                                //info on third move
                                if fighter_info_clickable_rect_3.rect_inside(mouse_pos) {
                                    gameinfo.player2_move_info = chloe_fighter_moves[2]; 
                                    game.state = GameStates::MoveInfo;
                                }
        
                                //select move and go to next state
                                if next_button_clickable_rect.rect_inside(mouse_pos) && player2_move_selected {
                                    player2_finish_selecting_move = true; 
                                    game.state = GameStates::ShowPick; 
                                }
                            }
                        }
                        if gameinfo.current_player2 == FighterType::Grace{
                            vulkan_state.fb2d.clear((0_u8, 255_u8, 242_u8, 100_u8));
                            if player2_selected{
                                if gameinfo.player2_current_move.fighter_move_type == FighterMoveType::NateMove1{
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_1,
                                    );
                                }

                                if gameinfo.player2_current_move.fighter_move_type == FighterMoveType::NateMove2{
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_2,
                                    );
                                }

                                if gameinfo.player2_current_move.fighter_move_type == FighterMoveType::NateMove3{
                                    vulkan_state.fb2d.bitblt(
                                        &highlight_rect,
                                        &highlight_rect_rect,
                                        highlight_rect_draw_to_3,
                                    );
                                }
                                
                        }
                            //grace move 1
                            vulkan_state.fb2d.bitblt(
                                &fighter_rect,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_1,
                            );

                            //grace move 2
                            vulkan_state.fb2d.bitblt(
                                &fighter_rect,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_2,
                            );

                            //grace move 3
                            vulkan_state.fb2d.bitblt(
                                &fighter_rect,
                                &fighter_rect_rect,
                                fighter_rect_draw_to_3,
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
        
                                //select the first move
                                if fighter_rect_clickable_rect_1.rect_inside(mouse_pos) {
                                    player2_move_selected = true;
                                    gameinfo.player2_current_move = nate_fighter_moves[0];
                                }
        
                                //select the second move
                                if fighter_rect_clickable_rect_2.rect_inside(mouse_pos) {
                                    player2_move_selected = true;
                                    gameinfo.player2_current_move = nate_fighter_moves[1]; 
                                }
        
                                //select the third move 
                                if fighter_rect_clickable_rect_3.rect_inside(mouse_pos) {
                                    player2_move_selected = true;
                                    gameinfo.player2_current_move = nate_fighter_moves[2]; 
                                }
        
                                //info on first move
                                if fighter_info_clickable_rect_1.rect_inside(mouse_pos) {
                                    gameinfo.player2_move_info = grace_fighter_moves[0]; 
                                    game.state = GameStates::MoveInfo;
                                }
        
                                //info on second move
                                if fighter_info_clickable_rect_2.rect_inside(mouse_pos) {
                                    gameinfo.player2_move_info = nate_fighter_moves[1]; 
                                    game.state = GameStates::MoveInfo;
                                }
        
                                //info on third move
                                if fighter_info_clickable_rect_3.rect_inside(mouse_pos) {
                                    gameinfo.player2_move_info = grace_fighter_moves[2]; 
                                    game.state = GameStates::MoveInfo;
                                }
        
                                //select move and go to next state
                                if next_button_clickable_rect.rect_inside(mouse_pos) && player2_move_selected {
                                    player2_finish_selecting_move = true; 
                                    game.state = GameStates::ShowPick;
                                }
                            }
                        }

                    }
                }

                else if game.state == GameStates::MoveInfo {
                    
                    vulkan_state.fb2d.draw_filled_rect(&mut back_button_rect, (155, 252, 232, 1));

                    if mouse_click == true && prev_mouse_click == false {
                        let mouse_pos = engine::image::Vec2i {
                            x: mouse_x as i32,
                            y: mouse_y as i32,
                        };

                        if back_button_rect.rect_inside(mouse_pos){
                            game.state = GameStates::ChooseMove;
                        }

                    }

                    //if player_info == nate, bit blit certain images
                    if gameinfo.player1_move_info.fighter_move_type == FighterMoveType::NateMove1 {
                        vulkan_state.fb2d.write_to(
                            "NATE",
                            &mut titlefontsheet_sprite,
                            title_draw_to,
                            titlefont_size,
                            title_box_dim,
                        );
                        vulkan_state.fb2d.write_to(
                            "let's pretend there is a description about our characters on the screen or something\
                            i need a really long string to test this text width thing. \
                            we can also use this to test character descriptions later. Lorem ipsum dolor sit amet, \
                            consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
                            &mut fontsheet_sprite,
                            description_draw_to,
                            fontsize,
                            description_box_dim
                        );
                    } 

                    if gameinfo.player1_move_info.fighter_move_type == FighterMoveType::NateMove2 {
                        vulkan_state.fb2d.write_to(
                            "NATE",
                            &mut titlefontsheet_sprite,
                            title_draw_to,
                            titlefont_size,
                            title_box_dim,
                        );
                        vulkan_state.fb2d.write_to(
                            "let's pretend there is a description about our characters on the screen or something\
                            i need a really long string to test this text width thing. \
                            we can also use this to test character descriptions later. Lorem ipsum dolor sit amet, \
                            consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
                            &mut fontsheet_sprite,
                            description_draw_to,
                            fontsize,
                            description_box_dim
                        );
                    } 

                    if gameinfo.player1_move_info.fighter_move_type == FighterMoveType::NateMove3 {
                        vulkan_state.fb2d.write_to(
                            "NATE",
                            &mut titlefontsheet_sprite,
                            title_draw_to,
                            titlefont_size,
                            title_box_dim,
                        );
                        vulkan_state.fb2d.write_to(
                            "let's pretend there is a description about our characters on the screen or something\
                            i need a really long string to test this text width thing. \
                            we can also use this to test character descriptions later. Lorem ipsum dolor sit amet, \
                            consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
                            &mut fontsheet_sprite,
                            description_draw_to,
                            fontsize,
                            description_box_dim
                        );
                    } 
                    
                    //if player_info == chloe, bit blit certain images
                    if gameinfo.player1_move_info.fighter_move_type == FighterMoveType::ChloeMove1 {
                        vulkan_state.fb2d.write_to(
                            "CHLOE",
                            &mut titlefontsheet_sprite,
                            title_draw_to,
                            titlefont_size,
                            title_box_dim,
                        );
                        vulkan_state.fb2d.write_to(
                            "let's pretend there is a description about our characters on the screen or something\
                            i need a really long string to test this text width thing. \
                            we can also use this to test character descriptions later. Lorem ipsum dolor sit amet, \
                            consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
                            &mut fontsheet_sprite,
                            description_draw_to,
                            fontsize,
                            description_box_dim
                        );
                    }

                    if gameinfo.player1_move_info.fighter_move_type == FighterMoveType::ChloeMove2 {
                        vulkan_state.fb2d.write_to(
                            "CHLOE",
                            &mut titlefontsheet_sprite,
                            title_draw_to,
                            titlefont_size,
                            title_box_dim,
                        );
                        vulkan_state.fb2d.write_to(
                            "let's pretend there is a description about our characters on the screen or something\
                            i need a really long string to test this text width thing. \
                            we can also use this to test character descriptions later. Lorem ipsum dolor sit amet, \
                            consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
                            &mut fontsheet_sprite,
                            description_draw_to,
                            fontsize,
                            description_box_dim
                        );
                    }

                    if gameinfo.player1_move_info.fighter_move_type == FighterMoveType::ChloeMove3 {
                        vulkan_state.fb2d.write_to(
                            "CHLOE",
                            &mut titlefontsheet_sprite,
                            title_draw_to,
                            titlefont_size,
                            title_box_dim,
                        );
                        vulkan_state.fb2d.write_to(
                            "let's pretend there is a description about our characters on the screen or something\
                            i need a really long string to test this text width thing. \
                            we can also use this to test character descriptions later. Lorem ipsum dolor sit amet, \
                            consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
                            &mut fontsheet_sprite,
                            description_draw_to,
                            fontsize,
                            description_box_dim
                        );
                    }
                    //if player_info == grace, bit blit certain images into the rectangles
                    if gameinfo.player1_move_info.fighter_move_type == FighterMoveType::GraceMove1 {
                        vulkan_state.fb2d.write_to(
                            "GRACE",
                            &mut titlefontsheet_sprite,
                            title_draw_to,
                            titlefont_size,
                            title_box_dim,
                        );
                        vulkan_state.fb2d.write_to(
                            "let's pretend there is a description about our characters on the screen or something\
                            i need a really long string to test this text width thing. \
                            we can also use this to test character descriptions later. Lorem ipsum dolor sit amet, \
                            consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
                            &mut fontsheet_sprite,
                            description_draw_to,
                            fontsize,
                            description_box_dim
                        );
                    }

                    if gameinfo.player1_move_info.fighter_move_type == FighterMoveType::GraceMove2 {
                        vulkan_state.fb2d.write_to(
                            "GRACE",
                            &mut titlefontsheet_sprite,
                            title_draw_to,
                            titlefont_size,
                            title_box_dim,
                        );
                        vulkan_state.fb2d.write_to(
                            "let's pretend there is a description about our characters on the screen or something\
                            i need a really long string to test this text width thing. \
                            we can also use this to test character descriptions later. Lorem ipsum dolor sit amet, \
                            consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
                            &mut fontsheet_sprite,
                            description_draw_to,
                            fontsize,
                            description_box_dim
                        );
                    }

                    if gameinfo.player1_move_info.fighter_move_type == FighterMoveType::GraceMove3 {
                        vulkan_state.fb2d.write_to(
                            "GRACE",
                            &mut titlefontsheet_sprite,
                            title_draw_to,
                            titlefont_size,
                            title_box_dim,
                        );
                        vulkan_state.fb2d.write_to(
                            "let's pretend there is a description about our characters on the screen or something\
                            i need a really long string to test this text width thing. \
                            we can also use this to test character descriptions later. Lorem ipsum dolor sit amet, \
                            consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
                            &mut fontsheet_sprite,
                            description_draw_to,
                            fontsize,
                            description_box_dim
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
                    //execute player2's moves
                    if f1.health + player2_move_damage < 0 
                    {
                        println!("go to final screen 1");
                        game.state = GameStates::FinalScreen; 
                        //and gameinfo.winning player = blank 
                        // player wins if die at same time?
                    }
                    else 
                    {
                        f1.health += player2_move_damage;
                    }

                    if f2.mana + player2_move_mana < 0 
                    {
                        println!("go to final screen 2");
                        game.state = GameStates::FinalScreen; 
                        //and gameinfo.winning player = blank 
                    }
                
                    else {
                        f2.mana += player2_move_mana; 
                    }
                

                    //checking so that they don't go above 100 health 
                    if f2.health + player2_move_health > 100
                    {
                        f2.health = 100; 
                    }
                    else 
                    {
                        f2.health += player2_move_health; 
                    }
                    //Apply mana generation
                    f1.mana += player1_mana_generation;
                    f2.mana += player2_mana_generation;

                    //execute player 2 moves 
                    
                    if f2.health + player1_move_damage < 0 
                    {
                        println!("go to final screen 3");
                        game.state = GameStates::FinalScreen; 
                        //and gameinfo.winning player = blank 
                    }
                    else 
                    {
                        f2.health += player1_move_damage; 
                    }

                    if f1.mana + player1_move_mana < 0 
                    {
                        println!("go to final screen 4");
                        game.state = GameStates::FinalScreen; 
                        //and gameinfo.winning player = blank 
                    }
                
                    else {
                        f1.mana += player1_move_mana; 
                    }
                

                    //checking so that they don't go above 100 health 
                    if f1.health + player1_move_health > 100
                    {
                        f1.health = 100; 
                    }
                    else 
                    {
                        f1.health += player1_move_health; 
                    }
                    done_execute_move = true;
                    //print out updated mana's and health's 
                    println!("Player1 move: {:?}", gameinfo.player1_current_move.fighter_move_type); 
                    println!("Player1 current health: {:?}", f1.health);
                    println!("Player1 current mana: {:?}", f1.mana);
                    println!("Player2 move: {:?}", gameinfo.player2_current_move.fighter_move_type); 
                    println!("Player2 current health: {:?}", f2.health);
                    println!("Player2 current mana: {:?}", f2.mana);
                } 

                // this is weird but basically if we don't have this, it'll animate one frame before moving to final screen
                // the more refined solution would be to separate this into its own game state, i.e., CalculatePick vs ShowPick
                // but i am lazy
                if game.state != GameStates::FinalScreen{
                //else if we are out of the while loop... 
                

                //okay having some type errors so this isn't the most elegant code 
                //we are going to have two fighters with info that is just their health and mana
                //the fighter choice and fighter moves are all contained in game info 

                /*
                let mut f1 = Fighter::<FighterType> {name: FighterType::None, is_cpu:false, is_turn: true, health: 100, mana: 100, current_move: None}; 
                let mut f2 = Fighter::<FighterType> {name: FighterType::None, is_cpu:true,  is_turn: false, health: 100, mana: 100, current_move: None}; 
                    */
                //IF STATEMENT THAT CHECKS THAT ONE OF THE MOVES WILL BE KILLER OR NOT 
                //IF THE MOVE IS KILLER THEN HAVE THAT MOVE EXECUTE FIRST 
                //execute the first move
                //animate the health loss 

                //execute teh seond move 
                //animate the health loss 


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
                        } else if p2_initial_health < f2.health{
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
                        && p2_initial_mana == f2.mana {
                            pick_anim_done = true;
                        };
        
                    };
                } 

                // drawing the p1 and p2 sprites
                // i put them before the bars bc i think i prefer the overlap of the bars
                // rather than the sprite on top of the bars
                let mut p1_src_img = &nate_fighter_rect;
                if gameinfo.current_player1 == FighterType:: Chloe {
                    p1_src_img = &chloe_fighter_rect;
                } else if gameinfo.current_player1 == FighterType::Grace {
                    p1_src_img = &nate_fighter_rect; // change tograce later
                }

                let mut p2_src_img = &nate_fighter_rect;
                if gameinfo.current_player2 == FighterType:: Chloe {
                    p2_src_img = &chloe_fighter_rect;
                } else if gameinfo.current_player2 == FighterType::Grace {
                    p2_src_img = &nate_fighter_rect; // change tograce later
                }

                vulkan_state.fb2d.bitblt(
                    p1_src_img,
                    &fighter_rect_rect,
                    p1_draw_to,
                );

                vulkan_state.fb2d.bitblt(
                    p2_src_img,
                    &fighter_rect_rect,
                    p2_draw_to,
                );

                // p1 labels and bars are all on the upper right corner
                // maybe we should swap p1 and p2 positions though, could be more intuitive
                vulkan_state.fb2d.write_to(
                    "HP",
                    &mut titlefontsheet_sprite,
                    Vec2i{x: hp_draw_to.x - (titlefont_size as i32) * 4, y: (HEIGHT as i32) - (mana_draw_to.y + titlefont_size as i32)},
                    titlefont_size,
                    Vec2i{x:(titlefont_size as i32) * 2, y:titlefont_size as i32},
                );

                vulkan_state.fb2d.write_to(
                    "MANA",
                    &mut titlefontsheet_sprite,
                    Vec2i{x: mana_draw_to.x - (titlefont_size as i32) * 4, y: (HEIGHT as i32) - (hp_draw_to.y + titlefont_size as i32)},
                    titlefont_size,
                    Vec2i{x:(titlefont_size as i32) * 4, y:titlefont_size as i32},
                );
                let mut f1_health_rect = engine::image::Rect::new(hp_draw_to.x, HEIGHT as i32 - (mana_draw_to.y + hp_y as i32), p1_initial_health as u32, hp_y);
                vulkan_state.fb2d.draw_filled_rect(&mut f1_health_rect, hp_color);
                let mut f1_mana_rect = engine::image::Rect::new(mana_draw_to.x, HEIGHT as i32 - (hp_draw_to.y + mana_y as i32), p1_initial_mana as u32, mana_y);
                vulkan_state.fb2d.draw_filled_rect(&mut f1_mana_rect, mana_color);


                // p2 labels and bars
                vulkan_state.fb2d.write_to(
                    "HP",
                    &mut titlefontsheet_sprite,
                    Vec2i{x: 10, y: hp_draw_to.y-2},
                    titlefont_size,
                    Vec2i{x:(titlefont_size as i32) * 6, y:titlefont_size as i32},
                );
                vulkan_state.fb2d.write_to(
                    "MANA",
                    &mut titlefontsheet_sprite,
                    Vec2i{x: 10, y: mana_draw_to.y-2},
                    titlefont_size,
                    Vec2i{x:(titlefont_size as i32) * 6, y:titlefont_size as i32},
                );
                
                
                let mut f2_health_rect = engine::image::Rect::new(10 + (titlefont_size as i32) * 4, hp_draw_to.y, p2_initial_health as u32, hp_y);
                vulkan_state.fb2d.draw_filled_rect(&mut f2_health_rect, hp_color);
                let mut f2_mana_rect = engine::image::Rect::new(10 + (titlefont_size as i32) * 4, mana_draw_to.y, p2_initial_mana as u32, mana_y);
                vulkan_state.fb2d.draw_filled_rect(&mut f2_mana_rect, mana_color);

                // if done, we can go to next move
                if now_keys[VirtualKeyCode::Return as usize] && pick_anim_done{
                    game.state = GameStates::ChooseMove;
                    player1_finish_selecting_move = false;
                    player2_finish_selecting_move = false;
                    pick_anim_done = false;
                    playing_anim = false;
                }
                }

                
                
            } else if game.state == GameStates::FinalScreen {
                vulkan_state.fb2d.write_to(
                    "GAME OVER",
                    &mut titlefontsheet_sprite,
                    Vec2i{x:20, y:20},
                    titlefont_size,
                    Vec2i{x: WIDTH as i32- 20, y: HEIGHT as i32 - 20}
                )
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