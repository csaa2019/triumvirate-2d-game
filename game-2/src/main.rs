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

    //GameState::ChooseFighter

    //temporary placeholder for our fighter images
    let fighter_rect_w = 86 as u32;
    let fighter_rect_h = 120 as u32;
    let fighter_rect_rect = engine::image::Rect::new(0, 0, fighter_rect_w, fighter_rect_h);
    let fighter_rect = engine::image::Image::from_png(
        "content/playerimagerect.png",
        fighter_rect_w,
        fighter_rect_h,
    );

    //a draw to for each chloe, nate, grace
    //for ease lets have 1 as chloe, 2 as nate, 3 as grace
    let fighter_rect_draw_to_1 = engine::image::Vec2i { x: 15, y: 50 };
    let fighter_rect_draw_to_2 = engine::image::Vec2i { x: 115, y: 50 };
    let fighter_rect_draw_to_3 = engine::image::Vec2i { x: 215, y: 50 };

    //temporary placeholder for our "read info" images
    let fighter_info_w = 86 as u32;
    let fighter_info_h = 24 as u32;
    let fighter_info_rect = engine::image::Rect::new(0, 0, fighter_info_w, fighter_info_h);
    let fighter_info = engine::image::Image::from_png(
        "content/playerinforect.png",
        fighter_info_w,
        fighter_info_h,
    );

    //a draw to for each chloe, nate, grace
    //for ease lets have 1 as chloe, 2 as nate, 3 as grace
    let fighter_info_draw_to_1 = engine::image::Vec2i { x: 15, y: 185 };
    let fighter_info_draw_to_2 = engine::image::Vec2i { x: 115, y: 185 };
    let fighter_info_draw_to_3 = engine::image::Vec2i { x: 215, y: 185 };

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
        state: GameStates::ChooseFighter,
    };

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

                if game.state == GameStates::ChooseFighter {
                    //nate image
                    vulkan_state.fb2d.bitblt(
                        &fighter_rect,
                        &fighter_rect_rect,
                        fighter_rect_draw_to_1,
                    );

                    //chloe image
                    vulkan_state.fb2d.bitblt(
                        &fighter_rect,
                        &fighter_rect_rect,
                        fighter_rect_draw_to_2,
                    );

                    //grace image
                    vulkan_state.fb2d.bitblt(
                        &fighter_rect,
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

                    if mouse_click == true && prev_mouse_click == false {
                        let mouse_pos = engine::image::Vec2i {
                            x: mouse_x as i32,
                            y: mouse_y as i32,
                        };

                        if fighter_info_clickable_rect_1.rect_inside(mouse_pos) {
                            game.state = GameStates::NateInfo;
                        }

                        if fighter_info_clickable_rect_2.rect_inside(mouse_pos) {
                            game.state = GameStates::ChloeInfo;
                        }

                        if fighter_info_clickable_rect_3.rect_inside(mouse_pos) {
                            game.state = GameStates::GraceInfo;
                        }
                    }
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