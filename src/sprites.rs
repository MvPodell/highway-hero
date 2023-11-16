use bytemuck::{Pod, Zeroable};
use rand::Rng;
use crate::{WINDOW_WIDTH, WINDOW_HEIGHT, NUMBER_OF_CELLS, CELL_WIDTH, CELL_HEIGHT, DESIRED_COLUMNS, DESIRED_ROWS};
use crate::input::Input;

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
pub struct GPUSprite {
    pub screen_region: [f32; 4],
    pub sheet_region: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
pub struct GPUCamera {
    pub screen_pos: [f32; 2],
    pub screen_size: [f32; 2],
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SpriteOption {
    Storage,
    Uniform,
    VertexBuffer,
}

pub fn create_sprites() ->  Vec<GPUSprite> {
    // screen region: 
    // sheet region: left x, top y,  width, height
    let mut sprites: Vec<GPUSprite> = vec![GPUSprite {
        screen_region: [WINDOW_WIDTH/2.0, 32.0, 70.0, 70.0],
        sheet_region: [0.0, 0.750, 0.5, 0.250], // guy
    }];


    for y in (2..NUMBER_OF_CELLS - 1).step_by(2) {
        // Create a vertical line of coins, cars, and potholes
        for x in (0..DESIRED_COLUMNS) {
            let x_value = x as f32 * CELL_WIDTH;
            let y_value = y as f32 * CELL_HEIGHT;

            // COINS
            sprites.push(GPUSprite {
                screen_region: [x_value, y_value, 50.0, 50.0],
                sheet_region: [0.0, 0.50, 0.5, 0.250], // coin
            });

            // CAR
            sprites.push(GPUSprite {
                screen_region: [x_value, y_value + CELL_HEIGHT, 50.0, 50.0],
                sheet_region: [0.0, 0.0, 0.5, 0.250], // car
            });

            // POT HOLES
            sprites.push(GPUSprite {
                screen_region: [x_value,  y_value + 2.0 * CELL_HEIGHT, 90.0, 90.0],
                sheet_region: [0.5, 0.50, 1.0, 0.250], // pothole
            });
        }
    }
    sprites

    // for y in (2..NUMBER_OF_CELLS-1).step_by(2) {
    //     // Create a horizontal line of stars, asteroids, and bombs
    //     // for x in 1..3 {
    //         let y_value = y as f32 * CELL_HEIGHT;

    //         // STARS
    //         sprites.push(GPUSprite {
    //             screen_region: [1 as f32 * CELL_WIDTH, y_value, 45.0, 45.0],
    //             sheet_region: [0.125, 0.625, 0.25, 0.25], // star
    //         });
    //         sprites.push(GPUSprite {
    //             screen_region: [2 as f32 * CELL_WIDTH, y_value, 45.0, 45.0],
    //             sheet_region: [0.125, 0.625, 0.25, 0.25], // star
    //         });
    //         sprites.push(GPUSprite {
    //             screen_region: [3 as f32 * CELL_WIDTH, y_value, 45.0, 45.0],
    //             sheet_region: [0.125, 0.625, 0.25, 0.25], // star
    //         });

    //         // ASTEROIDS
    //         sprites.push(GPUSprite {
    //             screen_region: [6 as f32 * CELL_WIDTH, y_value, 50.0, 50.0],
    //             sheet_region: [0.5625, 0.6, 0.375, 0.25], // asteroid
    //         });
    //         sprites.push(GPUSprite {
    //             screen_region: [7 as f32 * CELL_WIDTH, y_value, 50.0, 50.0],
    //             sheet_region: [0.5625, 0.6, 0.375, 0.25], // asteroid
    //         });
    //         sprites.push(GPUSprite {
    //             screen_region: [8 as f32 * CELL_WIDTH, y_value, 50.0, 50.0],
    //             sheet_region: [0.5625, 0.6, 0.375, 0.25], // asteroid
    //         });

    //         // BOMBS
    //         sprites.push(GPUSprite {
    //             screen_region: [11 as f32 * CELL_WIDTH, y_value, 45.0, 45.0],
    //             sheet_region: [0.625, 0.125, 0.25, 0.25], // bomb
    //         });
    //         sprites.push(GPUSprite {
    //             screen_region: [12 as f32 * CELL_WIDTH, y_value, 45.0, 45.0],
    //             sheet_region: [0.625, 0.125, 0.25, 0.25], // bomb
    //         });
    //         sprites.push(GPUSprite {
    //             screen_region: [13 as f32 * CELL_WIDTH, y_value, 45.0, 45.0],
    //             sheet_region: [0.625, 0.125, 0.25, 0.25], // bomb
    //         });

    // }
    // sprites

}

pub fn move_sprite_input(input: &Input, mut sprite_position: [f32; 2]) -> [f32; 2] {
        // Update sprite position based on keyboard input
        if input.is_key_pressed(winit::event::VirtualKeyCode::Up) {
            if sprite_position[1] + CELL_HEIGHT < WINDOW_HEIGHT {
                sprite_position[1] += CELL_HEIGHT;
            } else {
                sprite_position[1] = WINDOW_HEIGHT - CELL_HEIGHT;
            }
        }
        
        if input.is_key_pressed(winit::event::VirtualKeyCode::Down) {
            sprite_position[1] -= CELL_HEIGHT;

            if sprite_position[1] < 0.0 {
                sprite_position[1] = 0.0;
            }
        }
        if input.is_key_pressed(winit::event::VirtualKeyCode::Left) {
            sprite_position[0] -= CELL_WIDTH;

            if sprite_position[0] < 0.0 {
                sprite_position[0] = 0.0;
            }
        }
        if input.is_key_pressed(winit::event::VirtualKeyCode::Right) {
            if sprite_position[0] + CELL_WIDTH < WINDOW_WIDTH {
                sprite_position[0] += CELL_WIDTH;
            } else {
                sprite_position[0] = WINDOW_WIDTH - CELL_WIDTH;
            }
        }  
        sprite_position
}

