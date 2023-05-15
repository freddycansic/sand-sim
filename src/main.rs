use std::{cell, mem::swap, ptr::swap_nonoverlapping, slice::ChunksExactMut, time::Instant, vec};

use enum_iterator::{all, Sequence};
use fastrand::Rng;
use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

const WIDTH: usize = 400;
const HEIGHT: usize = 300;
const FPS: u32 = 120;
const TIME_PER_FRAME: u128 = ((1.0 / FPS as f32) * 1000.0 * 1000.0) as u128;

#[derive(PartialEq, Default, Clone, Copy, Sequence)]
enum CellType {
    #[default]
    Air,
    Sand,
    Water,
    Wood,
    Fire,
}

#[derive(PartialEq, Clone, Copy)]
struct Cell {
    ty: CellType,
    moved: bool,
    velocity: f32
}

impl From<CellType> for Cell {
    fn from(cell_type: CellType) -> Self {
        Self { ty: cell_type, moved: false, velocity: 1.0 }
    }
}

#[derive(Clone, Copy)]
enum Direction {
    Down,
    DownLeft,
    DownRight,
    Up,
    UpLeft,
    UpRight,
    Left,
    Right
}

fn update_cells(cells: &mut [Vec<Cell>], rng: &Rng) {
    let mut left_priority = true;

    for x in 0..WIDTH {
    for y in 0..HEIGHT {
            let cell = cells[x][y];

            if cell.moved {
                continue;
            }

            match cell.ty {
                CellType::Sand => update_sand(cells, x, y, rng),
                // CellType::Water => update_water(cells, x, y, rng),
                // CellType::Fire => update_fire(cells, x, y, rng),
                _ => (),
            }
        }
    }

    for cell_col in cells.iter_mut() {
        for cell in cell_col.iter_mut() {
            cell.moved = false;
        }
    }
}

fn update_sand(cells: &mut [Vec<Cell>], x: usize, y: usize, rng: &Rng) {
    let in_bounds_bottom = in_bounds_bottom(y + 1);
    
    if in_bounds_bottom && is_empty(cells, x, y + 1, &[CellType::Air]) {
        swap_cells(cells, (x, y), (x, y + 1));
        return;
    } 
    
    let can_go_down_left = in_bounds_bottom && x.checked_sub(1).is_some() && is_empty(cells, x - 1, y + 1, &[CellType::Air]);
    let can_go_down_right = in_bounds_bottom && in_bounds_right(x + 1) && is_empty(cells, x + 1, y + 1, &[CellType::Air]);

    if can_go_down_left && can_go_down_right {
        if rng.bool() {
            swap_cells(cells, (x, y), (x - 1, y + 1))
        } else {
            swap_cells(cells, (x, y), (x + 1, y + 1))
        }
    } else if can_go_down_left {
        swap_cells(cells, (x, y), (x - 1, y + 1))
    } else if can_go_down_right {
        swap_cells(cells, (x, y), (x + 1, y + 1))
    }
}

fn swap_cells(cells: &mut [Vec<Cell>], cell_1_pos: (usize, usize), cell_2_pos: (usize, usize)) {
    let temp_cell = cells[cell_1_pos.0][cell_1_pos.1];
    cells[cell_1_pos.0][cell_1_pos.1] = cells[cell_2_pos.0][cell_2_pos.1];
    cells[cell_2_pos.0][cell_2_pos.1] = temp_cell;
    cells[cell_1_pos.0][cell_1_pos.1].moved = false;
    cells[cell_2_pos.0][cell_2_pos.1].moved = true;
}

fn is_empty(cells: &[Vec<Cell>], x: usize, y: usize, empty_types: &[CellType]) -> bool {
    empty_types.contains(&cells[x][y].ty)
}

fn closest_by_vector(cells: &[Vec<Cell>], x: usize, y: usize, empty_types: &[CellType], direction: Direction) -> (usize, usize) {
    let mut closest = (x as isize, y as isize);

    match direction {
        Direction::Down => {
            for i in 0..cells[x][y].velocity as usize {
                let current_cell = (x as isize, y + i);
                if in_bounds_bottom(current_cell.1) && empty_types.contains(&cells[current_cell.0][current_cell.1].ty) {
                    closest = current_cell;
                }
            }
        },
        Direction::Up => {
            for i in 0..cells[x][y].velocity as isize {
                let current_cell = (x, y - i);
                if in_bounds_bottom(current_cell.1) && empty_types.contains(&cells[current_cell.0][current_cell.1].ty) {
                    closest = current_cell;
                }
            }
        },
        Direction::Down => {
            for i in 0..cells[x][y].velocity as usize {
                let current_cell = (x, y + i);
                if in_bounds_bottom(current_cell.1) && empty_types.contains(&cells[current_cell.0][current_cell.1].ty) {
                    closest = current_cell;
                }
            }
        },
        Direction::Down => {
            for i in 0..cells[x][y].velocity as usize {
                let current_cell = (x, y + i);
                if in_bounds_bottom(current_cell.1) && empty_types.contains(&cells[current_cell.0][current_cell.1].ty) {
                    closest = current_cell;
                }
            }
        },
        Direction::Down => {
            for i in 0..cells[x][y].velocity as usize {
                let current_cell = (x, y + i);
                if in_bounds_bottom(current_cell.1) && empty_types.contains(&cells[current_cell.0][current_cell.1].ty) {
                    closest = current_cell;
                }
            }
        },
        Direction::Down => {
            for i in 0..cells[x][y].velocity as usize {
                let current_cell = (x, y + i);
                if in_bounds_bottom(current_cell.1) && empty_types.contains(&cells[current_cell.0][current_cell.1].ty) {
                    closest = current_cell;
                }
            }
        },
        Direction::Down => {
            for i in 0..cells[x][y].velocity as usize {
                let current_cell = (x, y + i);
                if in_bounds_bottom(current_cell.1) && empty_types.contains(&cells[current_cell.0][current_cell.1].ty) {
                    closest = current_cell;
                }
            }
        },
        Direction::Down => {
            for i in 0..cells[x][y].velocity as usize {
                let current_cell = (x, y + i);
                if in_bounds_bottom(current_cell.1) && empty_types.contains(&cells[current_cell.0][current_cell.1].ty) {
                    closest = current_cell;
                }
            }
        }
    }

    closest
}

#[inline(always)]
fn in_bounds_bottom(y: usize) -> bool {
    y < HEIGHT
}

#[inline(always)]
fn in_bounds_right(x: usize) -> bool {
    x < WIDTH
}

fn draw_menu(frame: &mut [u8], selected_cell_type: CellType) {
    let starting_pixel_2d = (5, 5);
    let spacing = 2;
    let starting_pixel_1d = starting_pixel_2d.1 * WIDTH + starting_pixel_2d.0;

    for (cell_index, cell) in all::<CellType>().enumerate() {
        // pixels.nth(starting_pixel_1d + spacing * WIDTH * cell_index).unwrap().copy_from_slice(&[0xff, 0xff, 0xff, 0xff]);
    }

    for i in 0..WIDTH * 4 {
        frame[i] = 0xff;
        // pixels[2].unwrap().copy_from_slice(&[0xff, 0xff, 0xff, 0xff])
    }
}

fn draw_frame(pixels: &mut Pixels, cells: &[Vec<Cell>], selected_cell_type: CellType) {
    let frame = pixels.frame_mut();
    let mut pixels = frame.chunks_exact_mut(4);

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let cell = cells[x][y];

            let color = match cell.ty {
                CellType::Sand => [0xf2, 0xd2, 0xa9, 0xff],
                CellType::Water => [0x23, 0x89, 0xda, 0xff],
                CellType::Air => [0x00, 0x00, 0x00, 0xff],
                CellType::Wood => [0x50, 0x29, 0x00, 0xff],
                CellType::Fire => [0xc3, 0x3e, 0x05, 0xff]
            };

            pixels.next().unwrap().copy_from_slice(&color);
        }
    }

    draw_menu(frame, selected_cell_type)
}

fn put_cell(cells: &mut [Vec<Cell>], selected_cell_type: CellType, mouse_pos: (usize, usize), cursor_radius: f32, rng: &Rng) {
    let mouse_pos = (mouse_pos.0 as i32, mouse_pos.1 as i32);
    let cursor_radius = cursor_radius as i32;
    let cell = Cell::from(selected_cell_type);

    for x in
        ((mouse_pos.0 - cursor_radius).max(0))..((mouse_pos.0 + cursor_radius).min(WIDTH as i32))
    {
        for y in ((mouse_pos.1 - cursor_radius).max(0))
            ..((mouse_pos.1 + cursor_radius).min(HEIGHT as i32))
        {
            if (mouse_pos.0 - x).pow(2) + (mouse_pos.1 - y).pow(2) <= cursor_radius.pow(2){
                match selected_cell_type {
                    CellType::Sand | CellType::Water => {
                        if rng.f32() > 0.125 {
                            continue;
                        }
                    }
                    _ => () 
                }
                
                cells[x as usize][y as usize] = cell;
            }
        }
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        let scaled_size = LogicalSize::new(WIDTH as f64 * 2.0, HEIGHT as f64 * 2.0);
        WindowBuilder::new()
            .with_title("Hello pixels")
            .with_inner_size(scaled_size)
            .with_min_inner_size(size)
            .with_resizable(false)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture).unwrap()
    };

    let mut cells = vec![vec![Cell::from(CellType::Air); HEIGHT]; WIDTH];
    let mut cursor_radius = 3_f32;
    let mut current_cell_type = CellType::Sand;
    let rng = Rng::new();
    let mut last_frame_time = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        let current_frame_time = Instant::now();
        let delta_time = current_frame_time
            .duration_since(last_frame_time)
            .as_micros();

        if let Event::RedrawRequested(_) = event {
            draw_frame(&mut pixels, &cells, current_cell_type);

            if let Err(err) = pixels.render() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            if delta_time > TIME_PER_FRAME {
                update_cells(&mut cells, &rng);
                last_frame_time = current_frame_time;
            }
        }

        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape)
                || input.close_requested()
                || input.destroyed()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }

            if input.mouse_held(0) {
                let mouse_cell = input
                    .mouse()
                    .map(|(mx, my)| {
                        let (mx_i, my_i) = pixels
                            .window_pos_to_pixel((mx, my))
                            .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));

                        (mx_i, my_i)
                    })
                    .unwrap_or_default();

                put_cell(
                    &mut cells,
                    current_cell_type,
                    mouse_cell,
                    cursor_radius,
                    &rng,
                );
            }

            if input.key_pressed(VirtualKeyCode::Key1) {
                current_cell_type = CellType::Sand;
            } else if input.key_pressed(VirtualKeyCode::Key2) {
                current_cell_type = CellType::Water;
            } else if input.key_pressed(VirtualKeyCode::Key3) {
                current_cell_type = CellType::Wood;
            } else if input.key_pressed(VirtualKeyCode::Key4) {
                current_cell_type = CellType::Fire;
            }

            let scroll_diff = input.scroll_diff();
            let cursor_radius_step = 3.0;

            if scroll_diff != 0.0 {
                cursor_radius += scroll_diff * cursor_radius_step;
                cursor_radius = cursor_radius_step.max(cursor_radius);
            }
        }

        window.request_redraw()
    });
}
