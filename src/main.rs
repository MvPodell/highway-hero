use std::{borrow::Cow, mem, path::Path};
use rand::Rng;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
mod input;
mod gpu;
mod sprites;
use sprites::{GPUCamera, SpriteOption, GPUSprite};


#[cfg(all(not(feature = "uniforms"), not(feature = "vbuf")))]
const SPRITES: SpriteOption = SpriteOption::Storage;
#[cfg(feature = "uniforms")]
const SPRITES: SpriteOption = SpriteOption::Uniform;
#[cfg(feature = "vbuf")]
const SPRITES: SpriteOption = SpriteOption::VertexBuffer;
#[cfg(all(feature = "vbuf", feature = "uniform"))]
compile_error!("Can't choose both vbuf and uniform sprite features");

// get the width and height of the whole game screen
// pub const  WINDOW_WIDTH: f32 = 1024.0;
// pub const  WINDOW_HEIGHT: f32 = 768.0;

// pub const NUMBER_OF_CELLS: i32 = 16;

// // here divide by a number to create the number of grids
// pub const CELL_WIDTH: f32 = WINDOW_WIDTH / NUMBER_OF_CELLS as f32;
// pub const CELL_HEIGHT: f32 = WINDOW_HEIGHT / NUMBER_OF_CELLS as f32;


pub const WINDOW_WIDTH: f32 = 1024.0;
pub const WINDOW_HEIGHT: f32 = 768.0;

pub const DESIRED_COLUMNS: i32 = 3;
pub const DESIRED_ROWS: i32 = 9;

pub const NUMBER_OF_CELLS: i32 = DESIRED_COLUMNS * DESIRED_ROWS;

pub const CELL_WIDTH: f32 = WINDOW_WIDTH / DESIRED_COLUMNS as f32;
pub const CELL_HEIGHT: f32 = WINDOW_HEIGHT / DESIRED_ROWS as f32;

async fn run(event_loop: EventLoop<()>, window: Window) {

    log::info!("Use sprite mode {:?}", SPRITES);
    
    let mut gpu = gpu::WGPU::new(&window).await;
    
    // Load the shaders from disk
    let shader = gpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });

    let shader2 = gpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader2.wgsl"))),
    });

    let texture_bind_group_layout =
        gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            // It needs the first entry for the texture and the second for the sampler.
            // This is like defining a type signature.
            entries: &[
                // The texture binding
                wgpu::BindGroupLayoutEntry {
                    // This matches the binding in the shader
                    binding: 0,
                    // Only available in the fragment shader
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    // It's a texture binding
                    ty: wgpu::BindingType::Texture {
                        // We can use it with float samplers
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        // It's being used as a 2D texture
                        view_dimension: wgpu::TextureViewDimension::D2,
                        // This is not a multisampled texture
                        multisampled: false,
                    },
                    count: None,
                },
                // The sampler binding
                wgpu::BindGroupLayoutEntry {
                    // This matches the binding in the shader
                    binding: 1,
                    // Only available in the fragment shader
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    // It's a sampler
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    // No count
                    count: None,
                },
            ],
        });
    // The camera binding
    let camera_layout_entry = wgpu::BindGroupLayoutEntry {
        // This matches the binding in the shader
        binding: 0,
        // Available in vertex shader
        visibility: wgpu::ShaderStages::VERTEX,
        // It's a buffer
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        // No count, not a buffer array binding
        count: None,
    };
    let sprite_bind_group_layout = match SPRITES {
        SpriteOption::Storage => {
            gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    camera_layout_entry,
                    wgpu::BindGroupLayoutEntry {
                        // This matches the binding in the shader
                        binding: 1,
                        // Available in vertex shader
                        visibility: wgpu::ShaderStages::VERTEX,
                        // It's a buffer
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        // No count, not a buffer array binding
                        count: None,
                    },
                ],
            })
        }
        SpriteOption::Uniform => {
            gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    camera_layout_entry,
                    wgpu::BindGroupLayoutEntry {
                        // This matches the binding in the shader
                        binding: 1,
                        // Available in vertex shader
                        visibility: wgpu::ShaderStages::VERTEX,
                        // It's a buffer
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(SPRITE_UNIFORM_SIZE),
                        },
                        // No count, not a buffer array binding
                        count: None,
                    },
                ],
            })
        }
        SpriteOption::VertexBuffer => {
            gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[camera_layout_entry],
            })
        }
    };
    let pipeline_layout = gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&sprite_bind_group_layout, &texture_bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline_layout_over = gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&texture_bind_group_layout],
        push_constant_ranges: &[],
    });

    let render_pipeline_full = gpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout_over),
        vertex: wgpu::VertexState {
            module: &shader2,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader2,
            entry_point: "fs_main",
            targets: &[Some(gpu.config.format.into())],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    let render_pipeline = gpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: match SPRITES {
                SpriteOption::Storage => "vs_storage_main",
                SpriteOption::Uniform => "vs_uniform_main",
                SpriteOption::VertexBuffer => "vs_vbuf_main",
            },
            buffers: match SPRITES {
                SpriteOption::VertexBuffer => &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<GPUSprite>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: std::mem::size_of::<[f32; 4]>() as u64,
                            shader_location: 1,
                        },
                    ],
                }],
                _ => &[],
            },
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(gpu.config.format.into())],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    gpu.surface.configure(&gpu.device, &gpu.config);
    let path_sprites = Path::new("content/sprites.png");
    let (sprite_tex, _sprite_img) = gpu.load_texture(path_sprites, None)
        .await
        .expect("Couldn't load spritesheet texture");
    let view_sprite = sprite_tex.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler_sprite = gpu.device.create_sampler(&wgpu::SamplerDescriptor::default());
    let texture_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &texture_bind_group_layout,
        entries: &[
            // One for the texture, one for the sampler
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view_sprite),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler_sprite),
            },
        ],
    });
    let camera = GPUCamera {
        screen_pos: [0.0, 0.0],
        screen_size: [768.0, 768.0],
    };
    let buffer_camera = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: bytemuck::bytes_of(&camera).len() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut sprites: Vec<GPUSprite> = sprites::create_sprites();

    // Initialize sprite position within the grid
    let mut sprite_position: [f32; 2] = [512.0, 0.0];  

    const SPRITE_UNIFORM_SIZE: u64 = 512 * mem::size_of::<GPUSprite>() as u64;
    let buffer_sprite = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: if SPRITES == SpriteOption::Uniform {
            SPRITE_UNIFORM_SIZE
        } else {
            sprites.len() as u64 * std::mem::size_of::<GPUSprite>() as u64
        },
        usage: match SPRITES {
            SpriteOption::Storage => wgpu::BufferUsages::STORAGE,
            SpriteOption::Uniform => wgpu::BufferUsages::UNIFORM,
            SpriteOption::VertexBuffer => wgpu::BufferUsages::VERTEX,
        } | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let sprite_bind_group = match SPRITES {
        SpriteOption::VertexBuffer => gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &sprite_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer_camera.as_entire_binding(),
            }],
        }),
        _ => gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &sprite_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer_camera.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buffer_sprite.as_entire_binding(),
                },
            ],
        }),
    };

    // create background stuff
    let path_bgnd = Path::new("content/space.jpeg");
    let (tex_bgnd, _over_image) = gpu.load_texture(path_bgnd,None)
        .await
        .expect("Couldn't load space img");

    let view_bgnd = tex_bgnd.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler_bgnd = gpu.device.create_sampler(&wgpu::SamplerDescriptor::default());
        
    let mut texture_bind_group_bgnd = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &texture_bind_group_layout,
        entries: &[
            // One for the texture, one for the sampler
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view_bgnd),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler_bgnd),
            },
        ],
    });

    gpu.queue.write_buffer(&buffer_camera, 0, bytemuck::bytes_of(&camera));
    gpu.queue.write_buffer(&buffer_sprite, 0, bytemuck::cast_slice(&sprites));
    let mut input = input::Input::default();
    let mut game_over = false; 
    let mut you_won = false;
    let mut show_end_screen = false;

    let path_win = Path::new("content/youWin.png");

   //LOAD TEXTURE
   let (tex_win, _win_image) = gpu.load_texture(path_win,None)
        .await
        .expect("Couldn't load game over img");
    
    let path_over = Path::new("content/gameOver.png");
    let (tex_over, _over_image) = gpu.load_texture(path_over,None)
        .await
        .expect("Couldn't load game over img");

    event_loop.run(move |event, _, control_flow| {

        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                gpu.resize(size);
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                if game_over {
                    sprites[0].screen_region[1] -= 5.0;
                    if sprites[0].screen_region[1] < 0.0 {
                        show_end_screen = true;
                    }
                }

                else if you_won {
                    // enemy sprites fall!
                    let mut enemies = sprites.len()-1;
                    for i in 1..sprites.len(){
                        sprites[i].screen_region[1] -= 5.0;
                        if sprites[i].screen_region[1] < 0.0 {
                            enemies -= 1;
                        }
                    }

                    if enemies == 0 {
                        show_end_screen = true;
                    }
                }

                else {

                    // collision sprites
                    // let corners = vec![(sprites[0].screen_region[0], sprites[0].screen_region[1]), 
                    //                                     (sprites[0].screen_region[0] + sprites[0].screen_region[2], sprites[0].screen_region[1]),
                    //                                     (sprites[0].screen_region[0], sprites[0].screen_region[1]+ sprites[0].screen_region[3]),
                    //                                     (sprites[0].screen_region[0] + sprites[0].screen_region[2], sprites[0].screen_region[1]+ sprites[0].screen_region[3])];

    
                // sprites moving horizontally
                    // for i in 1..sprites.len(){
                        
                    //     if sprites[i].direction ==0{
                    //         // If direction is 0, move right
                    //         if sprites[i].screen_region[0] < WINDOW_WIDTH {
                    //             sprites[i].screen_region[0] += 1.0;
                    //         } else {
                    //             sprites[i].screen_region[0] = 0.0;
                    //         }
                    //     } else {
                    //         // If direction is 1, move left
                    //         if sprites[i].screen_region[0] > 0.0 {
                    //             sprites[i].screen_region[0] -= 1.0;
                    //         } else {
                    //             sprites[i].screen_region[0] = WINDOW_WIDTH;
                    //         }
                    //     }

                    // let mut direction_switch_counter = 0;
                    let mut current_direction = 0; // Start with direction 0 (right)

                    for i in 0..sprites.len() {
                        sprites[i].screen_region[0] = i as f32 * CELL_WIDTH; // Each sprite in a separate column
                        sprites[i].screen_region[1] = 0.0; // Start at the top of the screen
                    }

                    // Update sprite positions based on direction
                    for i in 0..sprites.len() {
                        if sprites[i].screen_region[1] < WINDOW_HEIGHT - CELL_HEIGHT {
                            // Move sprites down within the valid range
                            sprites[i].screen_region[1] += CELL_HEIGHT;
                        } else {
                            // Reset to the top of the screen if at the bottom
                            sprites[i].screen_region[1] = 0.0;
                        }
                    
                    // for i in 1..sprites.len() {
                    //     if current_direction == 0 {
                    //         // If direction is 0 (right), move right
                    //         if sprites[i].screen_region[1] < WINDOW_HEIGHT {
                    //             sprites[i].screen_region[1] += 1.0;
                    //         } else {
                    //             sprites[i].screen_region[1] = 0.0;
                    //         }
                    //     } else {
                    //         // If direction is 1 (left), move left
                    //         if sprites[i].screen_region[1] > 0.0 {
                    //             sprites[i].screen_region[1] -= 1.0;
                    //         } else {
                    //             sprites[i].screen_region[1] = WINDOW_HEIGHT;
                    //         }
                    //     }

                        // direction_switch_counter += 1;

                        // if direction_switch_counter == 9 {
                        //     // Switch the direction after every 3 sprites
                        //     direction_switch_counter = 0;
                        //     current_direction = 1 - current_direction; // Toggle between 0 and 1
                        // }
                    }


                        // if even move right
                        // if i%2==0{
                        //     if sprites[i].screen_region[0] < WINDOW_WIDTH{
                        //         sprites[i].screen_region[0] += 5.0;
                        //     }else{
                        //         let num = rand::thread_rng().gen_range(1..10); 
                        //         sprites[i].screen_region[0] = 0.0;
                        //         sprites[i].screen_region[1] =  num as f32 * CELL_HEIGHT;
                        //     }
                        // } else { // odd move left
                        //     if sprites[i].screen_region[0] > 0.0{
                        //         sprites[i].screen_region[0] -= 5.0;
                        //     } else {
                        //         let num = rand::thread_rng().gen_range(1..10); 
                        //         sprites[i].screen_region[0] = WINDOW_WIDTH;
                        //         sprites[i].screen_region[1] =  num as f32 * CELL_HEIGHT;
                        //     }
                        // }
                        
                    // }

                    // for i in 1..sprites.len() {
                    //     for (cx, cy) in corners.iter(){
                    //         if cx >= &sprites[i].screen_region[0] && cx <= &(sprites[i].screen_region[0] + sprites[0].screen_region[2]) && cy >= &sprites[i].screen_region[1] && cy <= &(sprites[i].screen_region[1] + sprites[0].screen_region[3]) {
                    //             print!("COLLIDED");
                    //             game_over = true;  
                    //         }
                    //     }
                    // }
                    
                    // move sprite based on input
                    sprite_position = sprites::move_sprite_input(&input, sprite_position);

                    if sprite_position[1] + CELL_HEIGHT >= WINDOW_HEIGHT {
                        you_won = true;
                    }

                    //update sprite position
                    sprites[0].screen_region[0] = sprite_position[0];
                    sprites[0].screen_region[1] = sprite_position[1];
                }
                
                // Then send the data to the GPU!
                input.next_frame();

                gpu.queue.write_buffer(&buffer_camera, 0, bytemuck::bytes_of(&camera));
                gpu.queue.write_buffer(&buffer_sprite, 0, bytemuck::cast_slice(&sprites));

                let frame = gpu.surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture");
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                
                let mut encoder =
                    gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                    {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    });
                    if show_end_screen{
                        let tex_end = 
                        if game_over {
                            &tex_over
                        } else {
                            &tex_win
                        };
                        let view_end = tex_end.create_view(&wgpu::TextureViewDescriptor::default());
                        let sampler_end = gpu.device.create_sampler(&wgpu::SamplerDescriptor::default());
                            
                        texture_bind_group_bgnd = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: None,
                            layout: &texture_bind_group_layout,
                            entries: &[
                                // One for the texture, one for the sampler
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(&view_end),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::Sampler(&sampler_end),
                                },
                            ],
                        });

                        // Draw end game screen
                        rpass.set_pipeline(&render_pipeline_full);
                        rpass.set_bind_group(0, &texture_bind_group_bgnd, &[]);
                        rpass.draw(0..6, 0..1);
                    } else {
                        
                        // Draw space background
                        rpass.set_pipeline(&render_pipeline_full);
                        rpass.set_bind_group(0, &texture_bind_group_bgnd, &[]);
                        rpass.draw(0..6, 0..1);
                        {
                            rpass.set_pipeline(&render_pipeline);
                            if SPRITES == SpriteOption::VertexBuffer {
                                rpass.set_vertex_buffer(0, buffer_sprite.slice(..));
                            }
                            rpass.set_bind_group(0, &sprite_bind_group, &[]);
                            rpass.set_bind_group(1, &texture_bind_group, &[]);
                            // draw two triangles per sprite, and sprites-many sprites.
                            // this uses instanced drawing, but it would also be okay
                            // to draw 6 * sprites.len() vertices and use modular arithmetic
                            // to figure out which sprite we're drawing.
                            rpass.draw(0..6, 0..(sprites.len() as u32));
                        }
                    }
                }
                gpu.queue.submit(Some(encoder.finish()));
                frame.present();
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            // WindowEvent->KeyboardInput: Keyboard input!
            Event::WindowEvent {
                // Note this deeply nested pattern match
                event: WindowEvent::KeyboardInput { input: key_ev, .. },
                ..
            } => {
                input.handle_key_event(key_ev);
            }
            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, button, .. },
                ..
            } => {
                input.handle_mouse_button(state, button);
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                input.handle_mouse_move(position);
            }
            _ => {}
        }
    });
}

fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
        pollster::block_on(run(event_loop, window));
    }
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Trace).expect("could not initialize logger");
        use winit::platform::web::WindowExtWebSys;
        // On wasm, append the canvas to the document body
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");
        wasm_bindgen_futures::spawn_local(run(event_loop, window));
    }
}