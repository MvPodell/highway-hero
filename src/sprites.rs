use bytemuck::{Pod, Zeroable};
use rand::Rng;
use crate::{WINDOW_WIDTH, WINDOW_HEIGHT, NUMBER_OF_CELLS, CELL_WIDTH, CELL_HEIGHT, DESIRED_COLUMNS, DESIRED_ROWS, COLUMN_LOCS};
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
    // screen region: top left x, top left y, width, height
    // sheet region: divided by spritesheet width, divided by spritesheet height, divided by spritesheet width, divided by spritesheet height
    let warp = 17.0/13.0;

    // instantiate sprite vec with guy sprite inside
    let mut sprites: Vec<GPUSprite> = vec![GPUSprite {
        screen_region: [WINDOW_WIDTH/2.0, 32.0, 50.0 * warp, 50.0],
        sheet_region: [0.0, 0.50, 0.5, 0.250], // guy
    }];

    // for y in (0..NUMBER_OF_CELLS - 1) {
    //     // Create a vertical line of coins, cars, and potholes

        for &x in COLUMN_LOCS.iter() {
            // COINS
            sprites.push(GPUSprite {
                screen_region: [x, WINDOW_HEIGHT - 40 as f32, 50.0 * warp, 50.0],
                sheet_region: [0.0, 0.50, 0.5, 0.250], // coin
            });

            // CAR
            sprites.push(GPUSprite {
                screen_region: [x, WINDOW_HEIGHT - 40 as f32 + CELL_HEIGHT, 50.0 * warp, 50.0],
                sheet_region: [0.0, 0.0, 0.5, 0.250], // car
            });

            // POT HOLES
            sprites.push(GPUSprite {
                screen_region: [x,  WINDOW_HEIGHT- 40 as f32 + (2.0 * CELL_HEIGHT), 90.0 * warp, 90.0],
                sheet_region: [0.5, 0.50, 1.0, 0.250], // pothole
            });
        // }
    }
    sprites

}

pub fn move_sprite_input(input: &Input, mut sprite_position: [f32; 2]) -> [f32; 2] {
        if input.is_key_pressed(winit::event::VirtualKeyCode::Left) {
            if sprite_position[0] > (WINDOW_WIDTH / 2.0) - CELL_WIDTH {
                sprite_position[0] -= CELL_WIDTH;
            }
        }
        if input.is_key_pressed(winit::event::VirtualKeyCode::Right) {
            if sprite_position[0] + CELL_WIDTH < WINDOW_WIDTH {
                sprite_position[0] += CELL_WIDTH;
            } 
        }  
        sprite_position
}

