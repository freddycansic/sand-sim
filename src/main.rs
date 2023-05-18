use std::{
    f32::consts::PI,
    time::{Duration, Instant},
    usize, vec,
};

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
const MAX_FPS: u32 = 999999;
const TIME_PER_FRAME_MICROSECONDS: u64 = (1_000_000.0 / MAX_FPS as f32) as u64;
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
    [0x65, 0x43, 0x33]
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

#[derive(Clone, Copy)]
struct CellMoveResult {
    moved_to_type: CellType,
    x: usize,
    y: usize
}

#[derive(PartialEq, Default, Clone, Copy, Sequence)]
enum CellType {
    #[default]
    Air,
    Sand,
    Water,
    Wood,
    Fire,
    Smoke,
    Steam
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
        CellType::Sand => update_sand(cells, x, y, &[CellType::Air, CellType::Water, CellType::Steam, CellType::Smoke], rng),
        CellType::Water => update_water(cells, x, y, &[CellType::Air, CellType::Steam, CellType::Smoke], rng),
        CellType::Fire => update_fire(cells, x, y, &[CellType::Wood], rng),
        CellType::Smoke => update_smoke(cells, x, y, &[CellType::Air], rng),
        CellType::Steam => update_steam(cells, x, y, &[CellType::Air], rng),
        _ => (),
    }
}

fn update_fire(cells: &mut [Vec<Cell>], x: usize, y: usize, burn_types: &[CellType], rng: &Rng) {
    if rng.f32() > 0.5_f32.powf(6.0) {
        return;
    }

    if in_bounds_left(x as isize - 1) && is_empty(cells, x - 1, y, &burn_types) {
        cells[x][y].color = cell_type_color_random(&cells[x][y], rng);
        spread_to_cell(cells, (x, y), (x - 1, y))
    }

    if in_bounds_right(x + 1) && is_empty(cells, x + 1, y, &burn_types) {
        cells[x][y].color = cell_type_color_random(&cells[x][y], rng);
        spread_to_cell(cells, (x, y), (x + 1, y))
    }

    if in_bounds_top(y as isize - 1) && is_empty(cells, x, y - 1, &burn_types) {
        cells[x][y].color = cell_type_color_random(&cells[x][y], rng);
        spread_to_cell(cells, (x, y), (x, y - 1))
    }

    if in_bounds_bottom(y + 1) && is_empty(cells, x, y + 1, &burn_types) {
        cells[x][y].color = cell_type_color_random(&cells[x][y], rng);
        spread_to_cell(cells, (x, y), (x, y + 1))
    }

    if in_bounds_left(x as isize - 1)
        && in_bounds_top(y as isize - 1)
        && is_empty(cells, x - 1, y - 1, &burn_types)
    {
        cells[x][y].color = cell_type_color_random(&cells[x][y], rng);
        spread_to_cell(cells, (x, y), (x - 1, y - 1))
    }

    if in_bounds_left(x as isize - 1)
        && in_bounds_bottom(y + 1)
        && is_empty(cells, x - 1, y + 1, &burn_types)
    {
        cells[x][y].color = cell_type_color_random(&cells[x][y], rng);
        spread_to_cell(cells, (x, y), (x - 1, y + 1))
    }

    if in_bounds_right(x + 1)
        && in_bounds_top(y as isize - 1)
        && is_empty(cells, x + 1, y - 1, &burn_types)
    {
        cells[x][y].color = cell_type_color_random(&cells[x][y], rng);
        spread_to_cell(cells, (x, y), (x + 1, y - 1))
    }

    if in_bounds_right(x + 1)
        && in_bounds_bottom(y + 1)
        && is_empty(cells, x + 1, y + 1, &burn_types)
    {
        cells[x][y].color = cell_type_color_random(&cells[x][y], rng);
        spread_to_cell(cells, (x, y), (x + 1, y + 1))
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
        x,
        y,
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
        x,
        y,
        empty_types,
        MAX_VELOCITY,
        ACCELERATION,
        false,
        rng,
    )
}

fn update_smoke(cells: &mut [Vec<Cell>], x: usize, y: usize, empty_types: &[CellType], rng: &Rng) {
    if cells[x][y].lifetime <= 0 {
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
        x,
        y,
        empty_types,
        SMOKE_MAX_VELOCITY,
        SMOKE_ACCELERATION,
        true,
        rng,
    )
}

fn update_steam(cells: &mut [Vec<Cell>], x: usize, y: usize, empty_types: &[CellType], rng: &Rng) {
    if cells[x][y].lifetime <= 0 {
        cells[x][y] = Cell::from(CellType::Air, rng);
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
        x,
        y,
        empty_types,
        STEAM_MAX_VELOCITY,
        STEAM_ACCELERATION,
        true,
        rng,
    )
}

fn generic_fluid(
    cells: &mut [Vec<Cell>],
    x: usize,
    y: usize,
    empty_types: &[CellType],
    max_velocity: f32,
    acceleration: f32,
    inverted: bool,
    rng: &Rng,
) {
    // todo something like: if the cell has a low velocity falling down then randomly spread to the side, will stop some water cells standing on top of others without spreading i think
    if generic_fall(
        cells,
        x,
        y,
        &empty_types,
        max_velocity,
        acceleration,
        inverted,
        rng,
    ) {
        return;
    }

    let spread_factor = (cells[x][y].velocity + 1.0) as usize;

    let furthest_left = furthest_by_vector(cells, x, y, spread_factor, &empty_types, (-1, 0));
    let furthest_right = furthest_by_vector(cells, x, y, spread_factor, &empty_types, (1, 0));

    if let (Some(furthest_left), Some(furthest_right)) = (furthest_left, furthest_right) {
        if rng.bool() {
            swap_cells(cells, (x, y), (furthest_right.x, furthest_right.y));
        } else {
            swap_cells(cells, (x, y), (furthest_left.x, furthest_left.y));
        }
    } else if let Some(furthest_left) = furthest_left {
        swap_cells(cells, (x, y), (furthest_left.x, furthest_left.y));
    } else if let Some(furthest_right) = furthest_right {
        swap_cells(cells, (x, y), (furthest_right.x, furthest_right.y));
    }
}

fn generic_fall(
    cells: &mut [Vec<Cell>],
    x: usize,
    y: usize,
    fall_through_types: &[CellType],
    max_velocity: f32,
    acceleration: f32,
    inverted: bool,
    rng: &Rng,
) -> bool {
    let down = if inverted { -1 } else { 1 };

    if let Some(furthest_down) = furthest_by_vector(
        cells,
        x,
        y,
        cells[x][y].velocity as usize,
        fall_through_types,
        (0, down),
    ) {
        cells[x][y].velocity = (cells[x][y].velocity + acceleration).min(max_velocity);
        swap_cells(cells, (x, y), (furthest_down.x, furthest_down.y));
        // todo swap current with furthest, then current with furthest - 1 = put whatever was in furthest on top of current

        return true;
    }

    let furthest_down_left = furthest_by_vector(
        cells,
        x,
        y,
        cells[x][y].velocity as usize,
        fall_through_types,
        (-1, down),
    );
    let furthest_down_right = furthest_by_vector(
        cells,
        x,
        y,
        cells[x][y].velocity as usize,
        fall_through_types,
        (1, down),
    );

    if let (Some(furthest_down_left), Some(furthest_down_right)) = (furthest_down_left, furthest_down_right) {
        if rng.bool() {
            cells[x][y].velocity = (cells[x][y].velocity + acceleration).min(max_velocity);
            swap_cells(cells, (x, y), (furthest_down_left.x, furthest_down_left.y));
        } else {
            cells[x][y].velocity = (cells[x][y].velocity + acceleration).min(max_velocity);
            swap_cells(cells, (x, y), (furthest_down_right.x, furthest_down_right.y));
        }

        return true;
    } else if let Some(furthest_down_left) = furthest_down_left {
        cells[x][y].velocity = (cells[x][y].velocity + acceleration).min(max_velocity);
        swap_cells(cells, (x, y), (furthest_down_left.x, furthest_down_left.y));
        return true;
    } else if let Some(furthest_down_right) = furthest_down_right {
        cells[x][y].velocity = (cells[x][y].velocity + acceleration).min(max_velocity);
        swap_cells(cells, (x, y), (furthest_down_right.x, furthest_down_right.y));
        return true;
    }

    // if we didnt move then turn down velocity
    cells[x][y].velocity /= 2.0;

    false
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
    x: usize,
    y: usize,
    movement_magnitude: usize,
    empty_types: &[CellType],
    direction: (isize, isize),
) -> Option<CellMoveResult> {
    assert!(direction.0.abs() <= 1 && direction.1.abs() <= 1);

    let mut closest = None;
    for i in 1..=(movement_magnitude + 1) as isize {
        let current_cell = (x as isize + direction.0 * i, y as isize + direction.1 * i);
        if in_bounds(current_cell.0, current_cell.1)
            && is_empty(
                cells,
                current_cell.0 as usize,
                current_cell.1 as usize,
                empty_types,
            )
        {
            closest = Some(CellMoveResult {
                moved_to_type: cells[x][y].ty,
                x: current_cell.0 as usize,
                y: current_cell.1 as usize
            });
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

fn put_cell(
    cells: &mut [Vec<Cell>],
    selected_cell_type: CellType,
    cursor_position: (usize, usize),
    cursor_radius: f32,
    rng: &Rng,
) {
    let cursor_position = (cursor_position.0 as i32, cursor_position.1 as i32);
    let cursor_radius = cursor_radius as i32;

    for x in ((cursor_position.0 - cursor_radius).max(0))
        ..((cursor_position.0 + cursor_radius).min(WIDTH as i32))
    {
        for y in ((cursor_position.1 - cursor_radius).max(0))
            ..((cursor_position.1 + cursor_radius).min(HEIGHT as i32))
        {
            if (cursor_position.0 - x).pow(2) + (cursor_position.1 - y).pow(2)
                > cursor_radius.pow(2)
            {
                continue;
            }

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
                x as usize,
                y as usize,
                &[CellType::Air, CellType::Water, CellType::Smoke],
            ) {
                cells[x as usize][y as usize] = Cell::from(selected_cell_type, rng)
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
    let mut cursor_position = (0, 0);
    let mut current_cell_type = CellType::Sand;
    let mut last_redraw = Instant::now()
        .checked_sub(Duration::from_micros(TIME_PER_FRAME_MICROSECONDS))
        .unwrap();

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawEventsCleared = event {
            if let Err(err) = pixels.render() {
                println!("ERROR: {}", err.to_string());
                *control_flow = ControlFlow::Exit;
                return;
            }

            let elapsed = last_redraw.elapsed().as_micros() as u64;
            let delta_time_microseconds = Instant::now().duration_since(last_redraw).as_micros();

            if elapsed > TIME_PER_FRAME_MICROSECONDS || MAX_FPS == 0 {
                last_redraw = Instant::now();

                draw_frame(
                    &mut pixels,
                    &cells,
                    current_cell_type,
                    cursor_position,
                    cursor_radius,
                );
                update_cells(&mut cells, &rng);

                let delta_time_milliseconds = delta_time_microseconds as f32 / 1000.0;
                window.set_title(
                    format!(
                        "Sand Sim: {:.2} FPS, {:.2} ms per frame",
                        1000.0 / delta_time_milliseconds,
                        delta_time_milliseconds
                    )
                    .as_str(),
                )
            };

            let deadline = last_redraw
                .checked_add(Duration::from_micros(TIME_PER_FRAME_MICROSECONDS))
                .unwrap();
            *control_flow = ControlFlow::WaitUntil(deadline);
        }

        cursor_position = input
            .mouse()
            .map(|(mx, my)| {
                let (mx_i, my_i) = pixels
                    .window_pos_to_pixel((mx, my))
                    .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));

                (mx_i, my_i)
            })
            .unwrap_or_default();

        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape)
                || input.close_requested()
                || input.destroyed()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }

            if input.mouse_held(0) {
                put_cell(
                    &mut cells,
                    current_cell_type,
                    cursor_position,
                    cursor_radius,
                    &rng,
                );
            }

            // note: if statements must follow order of declaration in the CellType enum
            // i cant make this procedural since VirtualKeyCode cannot be constructed from an integer primitive
            if input.key_pressed(VirtualKeyCode::Key1) {
                current_cell_type = CellType::Sand;
            } else if input.key_pressed(VirtualKeyCode::Key2) {
                current_cell_type = CellType::Water;
            } else if input.key_pressed(VirtualKeyCode::Key3) {
                current_cell_type = CellType::Wood;
            } else if input.key_pressed(VirtualKeyCode::Key4) {
                current_cell_type = CellType::Fire;
            } else if input.key_pressed(VirtualKeyCode::Key5) {
                current_cell_type = CellType::Smoke
            } else if input.key_pressed(VirtualKeyCode::Key6) {
                current_cell_type = CellType::Steam
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
