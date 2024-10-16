use std::time::{Duration, Instant};
use std::{f32::consts::PI, usize, vec};

use enum_iterator::{all, Sequence};
use fastrand::Rng;
use pixels::{Pixels, SurfaceTexture};
use winit::event::{ElementState, KeyboardInput, MouseButton, MouseScrollDelta, WindowEvent};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::EventLoop,
    window::WindowBuilder,
};

const WIDTH: usize = 400;
const HEIGHT: usize = 300;

const ACCELERATION: f32 = 0.2;
const MAX_VELOCITY: f32 = 10.0;

const SMOKE_MAX_VELOCITY: f32 = 2.0;
const SMOKE_ACCELERATION: f32 = 0.1;
const STEAM_MAX_VELOCITY: f32 = 2.0;
const STEAM_ACCELERATION: f32 = 0.1;

const SMOKE_LIFETIME: u32 = 100;
const STEAM_LIFETIME: u32 = 50;

const AIR_COLOR: [u8; 3] = [0x00, 0x00, 0x00];
const SAND_COLORS: [[u8; 3]; 4] = [
    [0xf6, 0xd7, 0xb0],
    [0xf2, 0xd2, 0xa9],
    [0xec, 0xcc, 0xa2],
    [0xe7, 0xc4, 0x96],
];
const WATER_COLORS: [[u8; 3]; 4] = [
    [0x18, 0x56, 0xdc],
    [0x1f, 0x59, 0xd6],
    [0x25, 0x5b, 0xd0],
    [0x27, 0x5c, 0xcd],
];
const WOOD_COLORS: [[u8; 3]; 4] = [
    [0x77, 0x4f, 0x3c],
    [0x71, 0x4b, 0x39],
    [0x6b, 0x47, 0x36],
    [0x65, 0x43, 0x33],
];
const FIRE_COLORS: [[u8; 3]; 6] = [
    // weighted colors = more red less yellow
    // reds
    [0xc3, 0x3e, 0x05],
    [0xc3, 0x3e, 0x05],
    [0xc2, 0x34, 0x05],
    [0xc2, 0x34, 0x05],
    // yellow orange
    [0xf9, 0x61, 0x1f],
    [0xf0, 0xa1, 0x2b],
];
const SMOKE_COLOR_LIGHT: [u8; 3] = [0x47, 0x47, 0x47];
const SMOKE_COLOR_DARK: [u8; 3] = [0x00, 0x00, 0x00];
const STEAM_COLOR_LIGHT: [u8; 3] = [0xf5, 0xf5, 0xf5];
const STEAM_COLOR_DARK: [u8; 3] = [0x00, 0x00, 0x00];

#[derive(PartialEq, Default, Clone, Copy, Sequence)]
enum CellType {
    #[default]
    Air,
    Sand,
    Water,
    Wood,
    Fire,
    Smoke,
    Steam,
}

#[derive(PartialEq, Clone)]
struct Cell {
    ty: CellType,
    moved: bool,
    velocity: f32,
    lifetime: u32,
    color: [u8; 3],
}

impl Cell {
    fn from(cell_type: CellType, rng: &Rng) -> Self {
        let mut cell = Cell {
            ty: cell_type,
            moved: false,
            velocity: 1.0,
            lifetime: cell_type_lifetime(cell_type),
            color: [0; 3],
        };

        // HEEELPPPPPPPP!
        cell.color = cell_type_color_random(&cell, rng);

        cell
    }
}

fn update_cells(cells: &mut [Vec<Cell>], rng: &Rng) {
    // traverse the odd indices left to right and the even indices left to right, removes any sort of cell movement priority
    for i in 0..=1 {
        for y in (0..HEIGHT).rev() {
            // forward pass, odds only
            if i == 1 {
                for x in 0..WIDTH {
                    update_cell(i, cells, x, y, rng)
                }
            // reverse pass, evens only
            } else {
                for x in (0..WIDTH).rev() {
                    update_cell(i, cells, x, y, rng)
                }
            }
        }
    }

    for cell_col in cells.iter_mut() {
        for cell in cell_col.iter_mut() {
            cell.moved = false;
        }
    }
}

fn update_cell(i: usize, cells: &mut [Vec<Cell>], x: usize, y: usize, rng: &Rng) {
    if x % 2 == i {
        return;
    }

    let cell = &cells[x][y];

    if cell.moved {
        return;
    }

    match cell.ty {
        CellType::Sand => update_sand(
            cells,
            x,
            y,
            &[
                CellType::Air,
                CellType::Water,
                CellType::Steam,
                CellType::Smoke,
            ],
            rng,
        ),
        CellType::Water => update_water(
            cells,
            x,
            y,
            &[CellType::Air, CellType::Steam, CellType::Smoke],
            rng,
        ),
        CellType::Fire => update_fire(cells, x, y, &[CellType::Wood], rng),
        CellType::Smoke => update_smoke(cells, x, y, &[CellType::Air], rng),
        CellType::Steam => update_steam(cells, x, y, &[CellType::Air], rng),
        _ => (),
    }
}

fn update_fire(cells: &mut [Vec<Cell>], x: usize, y: usize, burn_types: &[CellType], rng: &Rng) {
    let should_spread = rng.f32() < 0.5_f32.powf(6.0);

    if in_bounds_left(x as isize - 1) {
        let left_cell_type = cells[x - 1][y].ty;

        if burn_types.contains(&left_cell_type) {
            if should_spread {
                cells[x][y].color = cell_type_color_random(&cells[x][y], rng);
                spread_to_cell(cells, (x, y), (x - 1, y))
            }
        } else if left_cell_type == CellType::Water {
            cells[x][y] = Cell::from(CellType::Steam, rng);
            return;
        }
    }

    if in_bounds_right(x + 1) {
        let right_cell_type = cells[x + 1][y].ty;

        if burn_types.contains(&right_cell_type) {
            if should_spread {
                cells[x][y].color = cell_type_color_random(&cells[x][y], rng);
                spread_to_cell(cells, (x, y), (x + 1, y))
            }
        } else if right_cell_type == CellType::Water {
            cells[x][y] = Cell::from(CellType::Steam, rng);
            return;
        }
    }

    if in_bounds_top(y as isize - 1) {
        let top_cell_type = cells[x][y - 1].ty;

        if burn_types.contains(&top_cell_type) {
            if should_spread {
                cells[x][y].color = cell_type_color_random(&cells[x][y], rng);
                spread_to_cell(cells, (x, y), (x, y - 1))
            }
        } else if top_cell_type == CellType::Water {
            cells[x][y] = Cell::from(CellType::Steam, rng);
            return;
        }
    }

    if in_bounds_bottom(y + 1) {
        let bottom_cell_type = cells[x][y + 1].ty;

        if burn_types.contains(&bottom_cell_type) {
            if should_spread {
                cells[x][y].color = cell_type_color_random(&cells[x][y], rng);
                spread_to_cell(cells, (x, y), (x, y + 1))
            }
        } else if bottom_cell_type == CellType::Water {
            cells[x][y] = Cell::from(CellType::Steam, rng);
            return;
        }
    }

    if in_bounds_left(x as isize - 1) && in_bounds_top(y as isize - 1) {
        let top_left_cell_type = cells[x - 1][y - 1].ty;

        if burn_types.contains(&top_left_cell_type) {
            if should_spread {
                cells[x][y].color = cell_type_color_random(&cells[x][y], rng);
                spread_to_cell(cells, (x, y), (x - 1, y - 1))
            }
        } else if top_left_cell_type == CellType::Water {
            cells[x][y] = Cell::from(CellType::Steam, rng);
            return;
        }
    }

    if in_bounds_left(x as isize - 1) && in_bounds_bottom(y + 1) {
        let bottom_left_cell_type = cells[x - 1][y + 1].ty;

        if burn_types.contains(&bottom_left_cell_type) {
            if should_spread {
                cells[x][y].color = cell_type_color_random(&cells[x][y], rng);
                spread_to_cell(cells, (x, y), (x - 1, y + 1))
            }
        } else if bottom_left_cell_type == CellType::Water {
            cells[x][y] = Cell::from(CellType::Steam, rng);
            return;
        }
    }

    if in_bounds_right(x + 1) && in_bounds_top(y as isize - 1) {
        let top_right_cell_type = cells[x + 1][y - 1].ty;

        if burn_types.contains(&top_right_cell_type) {
            if should_spread {
                cells[x][y].color = cell_type_color_random(&cells[x][y], rng);
                spread_to_cell(cells, (x, y), (x + 1, y - 1))
            }
        } else if top_right_cell_type == CellType::Water {
            cells[x][y] = Cell::from(CellType::Steam, rng);
            return;
        }
    }

    if in_bounds_right(x + 1)
        && in_bounds_bottom(y + 1)
        && is_empty(cells, x + 1, y + 1, burn_types)
    {
        let bottom_right_cell_type = cells[x + 1][y + 1].ty;

        if burn_types.contains(&bottom_right_cell_type) {
            if should_spread {
                cells[x][y].color = cell_type_color_random(&cells[x][y], rng);
                spread_to_cell(cells, (x, y), (x + 1, y + 1))
            }
        } else if bottom_right_cell_type == CellType::Water {
            cells[x][y] = Cell::from(CellType::Steam, rng);
            return;
        }
    }

    if !should_spread {
        return;
    }

    cells[x][y] = if rng.f32() < 0.125 {
        Cell::from(CellType::Smoke, rng)
    } else {
        Cell::from(CellType::Air, rng)
    };
}

fn update_sand(cells: &mut [Vec<Cell>], x: usize, y: usize, empty_types: &[CellType], rng: &Rng) {
    generic_fall(
        cells,
        (x, y),
        empty_types,
        MAX_VELOCITY,
        ACCELERATION,
        false,
        rng,
    );
}

fn update_water(cells: &mut [Vec<Cell>], x: usize, y: usize, empty_types: &[CellType], rng: &Rng) {
    if rng.f32() < 0.125 && cells[x][y].velocity < 0.1 {
        cells[x][y].color = cell_type_color_random(&cells[x][y], rng);
    }

    generic_fluid(
        cells,
        (x, y),
        empty_types,
        MAX_VELOCITY,
        ACCELERATION,
        false,
        rng,
    );
}

fn update_smoke(cells: &mut [Vec<Cell>], x: usize, y: usize, empty_types: &[CellType], rng: &Rng) {
    if cells[x][y].lifetime == 0 {
        cells[x][y] = Cell::from(CellType::Air, rng);
        return;
    }

    cells[x][y].lifetime -= 1;

    cells[x][y].color = interpolate_color(
        &SMOKE_COLOR_LIGHT,
        &SMOKE_COLOR_DARK,
        cells[x][y].lifetime as f32 / SMOKE_LIFETIME as f32,
    );

    generic_fluid(
        cells,
        (x, y),
        empty_types,
        SMOKE_MAX_VELOCITY,
        SMOKE_ACCELERATION,
        true,
        rng,
    );
}

fn update_steam(cells: &mut [Vec<Cell>], x: usize, y: usize, empty_types: &[CellType], rng: &Rng) {
    if cells[x][y].lifetime == 0 {
        if rng.f32() < 0.5_f32.powf(6.0) {
            cells[x][y] = Cell::from(CellType::Water, rng)
        } else {
            cells[x][y] = Cell::from(CellType::Air, rng)
        }
        return;
    }

    cells[x][y].lifetime -= 1;

    cells[x][y].color = interpolate_color(
        &STEAM_COLOR_LIGHT,
        &STEAM_COLOR_DARK,
        cells[x][y].lifetime as f32 / STEAM_LIFETIME as f32,
    );

    generic_fluid(
        cells,
        (x, y),
        empty_types,
        STEAM_MAX_VELOCITY,
        STEAM_ACCELERATION,
        true,
        rng,
    );
}

fn generic_fluid(
    cells: &mut [Vec<Cell>],
    cell_pos: (usize, usize),
    empty_types: &[CellType],
    max_velocity: f32,
    acceleration: f32,
    inverted: bool,
    rng: &Rng,
) -> Option<(usize, usize)> {
    // todo something like: if the cell has a low velocity falling down then randomly spread to the side, will stop some water cells standing on top of others without spreading i think
    if let Some(fall_result) = generic_fall(
        cells,
        cell_pos,
        empty_types,
        max_velocity,
        acceleration,
        inverted,
        rng,
    ) {
        return Some(fall_result);
    }

    let spread_factor = (cells[cell_pos.0][cell_pos.1].velocity + 1.0) as usize;

    let furthest_left = furthest_by_vector(cells, cell_pos, spread_factor, empty_types, (-1, 0));
    let furthest_right = furthest_by_vector(cells, cell_pos, spread_factor, empty_types, (1, 0));

    if let (Some(furthest_left), Some(furthest_right)) = (furthest_left, furthest_right) {
        if rng.bool() {
            swap_cells(cells, cell_pos, (furthest_right.0, furthest_right.1));
            return Some(furthest_right);
        } else {
            swap_cells(cells, cell_pos, (furthest_left.0, furthest_left.1));
            return Some(furthest_left);
        }
    } else if let Some(furthest_left) = furthest_left {
        swap_cells(cells, cell_pos, (furthest_left.0, furthest_left.1));
        return Some(furthest_left);
    } else if let Some(furthest_right) = furthest_right {
        swap_cells(cells, cell_pos, (furthest_right.0, furthest_right.1));
        return Some(furthest_right);
    }

    None
}

fn generic_fall(
    cells: &mut [Vec<Cell>],
    cell_pos: (usize, usize),
    fall_through_types: &[CellType],
    max_velocity: f32,
    acceleration: f32,
    inverted: bool,
    rng: &Rng,
) -> Option<(usize, usize)> {
    let down = if inverted { -1 } else { 1 };

    if let Some(furthest_down) = furthest_by_vector(
        cells,
        cell_pos,
        cells[cell_pos.0][cell_pos.1].velocity as usize,
        fall_through_types,
        (0, down),
    ) {
        cells[cell_pos.0][cell_pos.1].velocity =
            (cells[cell_pos.0][cell_pos.1].velocity + acceleration).min(max_velocity);
        swap_cells(cells, cell_pos, (furthest_down.0, furthest_down.1));
        // todo swap current with furthest, then current with furthest - 1 = put whatever was in furthest on top of current

        return Some(furthest_down);
    }

    let furthest_down_left = furthest_by_vector(
        cells,
        cell_pos,
        cells[cell_pos.0][cell_pos.1].velocity as usize,
        fall_through_types,
        (-1, down),
    );
    let furthest_down_right = furthest_by_vector(
        cells,
        cell_pos,
        cells[cell_pos.0][cell_pos.1].velocity as usize,
        fall_through_types,
        (1, down),
    );

    if let (Some(furthest_down_left), Some(furthest_down_right)) =
        (furthest_down_left, furthest_down_right)
    {
        if rng.bool() {
            cells[cell_pos.0][cell_pos.1].velocity =
                (cells[cell_pos.0][cell_pos.1].velocity + acceleration).min(max_velocity);
            swap_cells(
                cells,
                cell_pos,
                (furthest_down_left.0, furthest_down_left.1),
            );
            return Some(furthest_down_left);
        } else {
            cells[cell_pos.0][cell_pos.1].velocity =
                (cells[cell_pos.0][cell_pos.1].velocity + acceleration).min(max_velocity);
            swap_cells(
                cells,
                cell_pos,
                (furthest_down_right.0, furthest_down_right.1),
            );
            return Some(furthest_down_right);
        }
    } else if let Some(furthest_down_left) = furthest_down_left {
        cells[cell_pos.0][cell_pos.1].velocity =
            (cells[cell_pos.0][cell_pos.1].velocity + acceleration).min(max_velocity);
        swap_cells(
            cells,
            cell_pos,
            (furthest_down_left.0, furthest_down_left.1),
        );
        return Some(furthest_down_left);
    } else if let Some(furthest_down_right) = furthest_down_right {
        cells[cell_pos.0][cell_pos.1].velocity =
            (cells[cell_pos.0][cell_pos.1].velocity + acceleration).min(max_velocity);
        swap_cells(
            cells,
            cell_pos,
            (furthest_down_right.0, furthest_down_right.1),
        );
        return Some(furthest_down_right);
    }

    // if we didnt move then turn down velocity
    cells[cell_pos.0][cell_pos.1].velocity /= 2.0;

    None
}

fn swap_cells(cells: &mut [Vec<Cell>], cell_1_pos: (usize, usize), cell_2_pos: (usize, usize)) {
    let temp_cell = cells[cell_1_pos.0][cell_1_pos.1].clone();
    cells[cell_1_pos.0][cell_1_pos.1] = cells[cell_2_pos.0][cell_2_pos.1].clone();
    cells[cell_2_pos.0][cell_2_pos.1] = temp_cell;
    cells[cell_1_pos.0][cell_1_pos.1].moved = false;
    cells[cell_2_pos.0][cell_2_pos.1].moved = true;
}

fn spread_to_cell(cells: &mut [Vec<Cell>], cell_1_pos: (usize, usize), cell_2_pos: (usize, usize)) {
    cells[cell_2_pos.0][cell_2_pos.1] = cells[cell_1_pos.0][cell_1_pos.1].clone();
    cells[cell_2_pos.0][cell_2_pos.1].moved = true;
    cells[cell_1_pos.0][cell_1_pos.1].moved = true;
}

fn is_empty(cells: &[Vec<Cell>], x: usize, y: usize, empty_types: &[CellType]) -> bool {
    empty_types.contains(&cells[x][y].ty)
}

fn furthest_by_vector(
    cells: &[Vec<Cell>],
    cell_pos: (usize, usize),
    movement_magnitude: usize,
    empty_types: &[CellType],
    direction: (isize, isize),
) -> Option<(usize, usize)> {
    assert!(direction.0.abs() <= 1 && direction.1.abs() <= 1);

    let mut closest = None;
    for i in 1..=(movement_magnitude + 1) as isize {
        let current_cell = (
            cell_pos.0 as isize + direction.0 * i,
            cell_pos.1 as isize + direction.1 * i,
        );
        if in_bounds(current_cell.0, current_cell.1)
            && is_empty(
                cells,
                current_cell.0 as usize,
                current_cell.1 as usize,
                empty_types,
            )
        {
            closest = Some((current_cell.0 as usize, current_cell.1 as usize))
        }
        // not breaking causes clipping but breaking makes everything funny. idc im doing it
        // else {
        //     break
        // }
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

#[inline(always)]
fn in_bounds_top(y: isize) -> bool {
    y >= 0
}

#[inline(always)]
fn in_bounds_left(x: isize) -> bool {
    x >= 0
}

#[inline(always)]
fn in_bounds(x: isize, y: isize) -> bool {
    // check left and top = check greater than -1 before casting to usize, sketchy
    in_bounds_left(x)
        && in_bounds_top(y)
        && in_bounds_bottom(y as usize)
        && in_bounds_right(x as usize)
}

fn draw_menu(frame: &mut [u8], selected_cell_type: CellType) {
    let starting = (3, 3);
    let spacing = 3;
    let square_size = 15;

    // skip 1 = skip drawing the square for the air cell type
    for (cell_type_index, cell_type) in all::<CellType>().skip(1).enumerate() {
        if selected_cell_type == cell_type {
            draw_square(
                frame,
                (
                    starting.0 + spacing - 1,
                    starting.0 + (spacing + square_size) * cell_type_index - 1,
                ),
                square_size + 2,
                &[0xff, 0xea, 0x00],
                Some(&cell_type_color_fixed(cell_type)),
            );
        } else {
            draw_square(
                frame,
                (
                    starting.0 + spacing,
                    starting.0 + (spacing + square_size) * cell_type_index,
                ),
                square_size,
                &[0xff, 0xff, 0xff],
                Some(&cell_type_color_fixed(cell_type)),
            );
        }
    }
}

fn draw_square(
    frame: &mut [u8],
    top_left: (usize, usize),
    size: usize,
    border_color: &[u8; 3],
    fill_color: Option<&[u8; 3]>,
) {
    // unsafe function no bounds checking
    for y in 0..size {
        for x in 0..size {
            let current_pixel = to_1d_index_pixel_buffer(x + top_left.0, y + top_left.1);

            if y == size - 1 || y == 0 || x == size - 1 || x == 0 {
                write_to_pixel_buffer(frame, current_pixel, border_color)
            } else if let Some(color) = fill_color {
                write_to_pixel_buffer(frame, current_pixel, color)
            }
        }
    }
}

fn draw_cursor(frame: &mut [u8], cursor_position: (usize, usize), cursor_radius: f32) {
    for theta in (0..(2.0 * PI * 1000.0) as u32).step_by(10) {
        let theta = theta as f32 * 0.001;

        let current_pixel = (
            (cursor_position.0 as f32 + cursor_radius * theta.cos()) as usize,
            (cursor_position.1 as f32 + cursor_radius * theta.sin()) as usize,
        );

        if !in_bounds(current_pixel.0 as isize, current_pixel.1 as isize) {
            continue;
        }

        // ? i love working with 1d arrays
        let frame_index = to_1d_index_pixel_buffer(current_pixel.0, current_pixel.1).checked_sub(4);

        if frame_index.is_none() || frame_index.unwrap() >= frame.len() {
            continue;
        }

        let frame_index = frame_index.unwrap() + 4;

        write_to_pixel_buffer(frame, frame_index, &[0xe0, 0xe0, 0xe0])
    }
}

fn write_to_pixel_buffer(frame: &mut [u8], index: usize, color: &[u8; 3]) {
    frame[index] = color[0];
    frame[index + 1] = color[1];
    frame[index + 2] = color[2];
    frame[index + 3] = 0xff;
}

#[inline(always)]
fn to_1d_index_pixel_buffer(x: usize, y: usize) -> usize {
    y * WIDTH * 4 + x * 4
}

fn interpolate_color(color_1: &[u8; 3], color_2: &[u8; 3], factor: f32) -> [u8; 3] {
    let r_difference = color_1[0] as f32 - color_2[0] as f32;
    let g_difference = color_1[1] as f32 - color_2[1] as f32;
    let b_difference = color_1[2] as f32 - color_2[2] as f32;

    [
        (r_difference * factor + color_2[0] as f32) as u8,
        (g_difference * factor + color_2[1] as f32) as u8,
        (b_difference * factor + color_2[2] as f32) as u8,
    ]
}

// fn for the cell type picker menu
fn cell_type_color_fixed(cell_type: CellType) -> [u8; 3] {
    match cell_type {
        CellType::Sand => SAND_COLORS[0],
        CellType::Water => WATER_COLORS[0],
        CellType::Air => AIR_COLOR,
        CellType::Wood => WOOD_COLORS[0],
        CellType::Fire => FIRE_COLORS[0],
        CellType::Smoke => SMOKE_COLOR_LIGHT,
        CellType::Steam => STEAM_COLOR_LIGHT,
    }
}

fn cell_type_color_random(cell: &Cell, rng: &Rng) -> [u8; 3] {
    match cell.ty {
        CellType::Sand => SAND_COLORS[rng.usize(0..SAND_COLORS.len())],
        CellType::Water => WATER_COLORS[rng.usize(0..WATER_COLORS.len())],
        CellType::Air => AIR_COLOR,
        CellType::Wood => WOOD_COLORS[rng.usize(0..WOOD_COLORS.len())],
        CellType::Fire => FIRE_COLORS[rng.usize(0..FIRE_COLORS.len())],
        CellType::Smoke => SMOKE_COLOR_LIGHT,
        CellType::Steam => STEAM_COLOR_LIGHT,
    }
}

fn cell_type_lifetime(cell_type: CellType) -> u32 {
    match cell_type {
        CellType::Smoke => SMOKE_LIFETIME,
        CellType::Steam => STEAM_LIFETIME,
        _ => 0,
    }
}

fn draw_frame(
    pixels: &mut Pixels,
    cells: &[Vec<Cell>],
    selected_cell_type: CellType,
    cursor_position: (usize, usize),
    cursor_radius: f32,
) {
    let frame = pixels.frame_mut();
    let mut pixels = frame.chunks_exact_mut(4);

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let cell_color = &cells[x][y].color;

            let color = [cell_color[0], cell_color[1], cell_color[2], 0xff];

            pixels.next().unwrap().copy_from_slice(&color);
        }
    }

    draw_menu(frame, selected_cell_type);
    draw_cursor(frame, cursor_position, cursor_radius);
}

fn cursor_region_cell_coordinates(
    cursor_position: (usize, usize),
    cursor_radius: f32,
) -> impl Iterator<Item = (usize, usize)> {
    let cursor_position = (cursor_position.0 as i32, cursor_position.1 as i32);
    let cursor_radius = cursor_radius as i32;

    let x_start = (cursor_position.0 - cursor_radius).max(0);
    let x_end = (cursor_position.0 + cursor_radius).min(WIDTH as i32);
    let y_start = (cursor_position.1 - cursor_radius).max(0);
    let y_end = (cursor_position.1 + cursor_radius).min(HEIGHT as i32);

    (x_start..x_end)
        .flat_map(move |x| (y_start..y_end).map(move |y| (x, y)))
        .filter(move |&(x, y)| {
            (cursor_position.0 - x).pow(2) + (cursor_position.1 - y).pow(2) <= cursor_radius.pow(2)
        })
        .map(move |(x, y)| (x as usize, y as usize))
}

fn put_cell(
    cells: &mut [Vec<Cell>],
    selected_cell_type: CellType,
    cursor_position: (usize, usize),
    cursor_radius: f32,
    rng: &Rng,
) {
    for (x, y) in cursor_region_cell_coordinates(cursor_position, cursor_radius) {
        match selected_cell_type {
            CellType::Sand | CellType::Water | CellType::Fire | CellType::Smoke => {
                if rng.f32() > 0.125 {
                    continue;
                }
            }
            _ => (),
        }

        // place cells only in fluids
        if is_empty(
            cells,
            x,
            y,
            &[CellType::Air, CellType::Smoke, CellType::Water],
        ) {
            cells[x][y] = Cell::from(selected_cell_type, rng)
        }
    }
}

fn remove_cells(
    cells: &mut [Vec<Cell>],
    cursor_position: (usize, usize),
    cursor_radius: f32,
    rng: &Rng,
) {
    for (x, y) in cursor_region_cell_coordinates(cursor_position, cursor_radius) {
        cells[x][y] = Cell::from(CellType::Air, rng);
    }
}

fn main() {
    let event_loop = EventLoop::new();

    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        let scaled_size = LogicalSize::new(WIDTH as f64 * 2.0, HEIGHT as f64 * 2.0);
        WindowBuilder::new()
            .with_title("Sand Sim")
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

    let rng = Rng::new();

    let mut cells = vec![vec![Cell::from(CellType::Air, &rng); HEIGHT]; WIDTH];
    let mut cursor_radius = 3_f32;
    let mut cursor_position = (WIDTH / 2, HEIGHT / 2);
    let mut lmb_down = false;
    let mut rmb_down = false;
    let mut current_cell_type = CellType::Sand;

    let max_fps = if let Some(fps) = std::env::args().nth(1) {
        fps.parse::<u32>().unwrap()
    } else {
        // unlimited
        0
    };

    let time_per_frame_micros = (1_000_000.0 / max_fps as f32) as u64;

    let mut last_redraw = Instant::now();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { ref event, .. } => match event {
            WindowEvent::CloseRequested => control_flow.set_exit(),
            WindowEvent::MouseInput { button, state, .. } => match button {
                MouseButton::Left => lmb_down = *state == ElementState::Pressed,
                MouseButton::Right => rmb_down = *state == ElementState::Pressed,
                _ => (),
            },
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(_, dy),
                ..
            } => {
                let cursor_radius_step = 3.0;

                if *dy != 0.0 {
                    cursor_radius += dy * cursor_radius_step;
                    cursor_radius = cursor_radius_step.max(cursor_radius);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                cursor_position = pixels
                    .window_pos_to_pixel((position.x as f32, position.y as f32))
                    .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(virtual_keycode),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => match virtual_keycode {
                VirtualKeyCode::Escape => control_flow.set_exit(),
                VirtualKeyCode::Key1 => current_cell_type = CellType::Sand,
                VirtualKeyCode::Key2 => current_cell_type = CellType::Water,
                VirtualKeyCode::Key3 => current_cell_type = CellType::Wood,
                VirtualKeyCode::Key4 => current_cell_type = CellType::Fire,
                VirtualKeyCode::Key5 => current_cell_type = CellType::Smoke,
                VirtualKeyCode::Key6 => current_cell_type = CellType::Steam,
                _ => (),
            },
            _ => (),
        },
        Event::MainEventsCleared => window.request_redraw(),
        Event::RedrawRequested(_) => {
            let delta_micros = last_redraw.elapsed().as_micros() as u64;

            if delta_micros > time_per_frame_micros || max_fps == 0 {
                // tick the simulation
                if lmb_down {
                    put_cell(
                        &mut cells,
                        current_cell_type,
                        cursor_position,
                        cursor_radius,
                        &rng,
                    );
                }

                if rmb_down {
                    remove_cells(&mut cells, cursor_position, cursor_radius, &rng)
                }

                update_cells(&mut cells, &rng);

                draw_frame(
                    &mut pixels,
                    &cells,
                    current_cell_type,
                    cursor_position,
                    cursor_radius,
                );

                if let Err(error) = pixels.render() {
                    eprintln!("{error}");
                    control_flow.set_exit();
                }

                let delta_millis = delta_micros as f32 / 1000.0;
                window.set_title(
                    format!(
                        "Sand Sim: {:.2} FPS, {:.2} ms per frame",
                        1000.0 / delta_millis,
                        delta_millis
                    )
                    .as_str(),
                );

                last_redraw = Instant::now();
            } else {
                let deadline = last_redraw
                    .checked_add(Duration::from_micros(time_per_frame_micros))
                    .unwrap();

                control_flow.set_wait_until(deadline);
            }
        }
        _ => (),
    });
}
