use std::{
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
const MAX_FPS: u32 = 0;
const TIME_PER_FRAME_MICROSECONDS: u64 = (1_000_000.0 / MAX_FPS as f32) as u64;
const ACCELERATION: f32 = 0.2;
const TERMINAL_VELOCITY: f32 = 5.0;

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
    velocity: f32,
}

impl From<CellType> for Cell {
    fn from(cell_type: CellType) -> Self {
        Self {
            ty: cell_type,
            moved: false,
            velocity: 1.0,
        }
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

    let cell = cells[x][y];

    if cell.moved {
        return;
    }

    match cell.ty {
        CellType::Sand => update_sand(cells, x, y, rng),
        CellType::Water => update_water(cells, x, y, rng),
        CellType::Fire => update_fire(cells, x, y, rng),
        _ => (),
    }
}

fn update_fire(cells: &mut [Vec<Cell>], x: usize, y: usize, rng: &Rng) {
    if rng.f32() < 0.5 {
        return;
    }

    let empty_types = [CellType::Wood];

    if in_bounds_left(x as isize - 1) && is_empty(cells, x - 1, y, &empty_types) {
        spread_to_cell(cells, (x, y), (x - 1, y))
    }

    if in_bounds_right(x + 1) && is_empty(cells, x + 1, y, &empty_types) {
        spread_to_cell(cells, (x, y), (x + 1, y))
    }

    if in_bounds_top(y as isize - 1) && is_empty(cells, x, y - 1, &empty_types) {
        spread_to_cell(cells, (x, y), (x, y - 1))
    }

    if in_bounds_bottom(y + 1) && is_empty(cells, x, y + 1, &empty_types) {
        spread_to_cell(cells, (x, y), (x, y + 1))
    }

    if in_bounds_left(x as isize - 1)
        && in_bounds_top(y as isize - 1)
        && is_empty(cells, x - 1, y - 1, &empty_types)
    {
        spread_to_cell(cells, (x, y), (x - 1, y - 1))
    }

    if in_bounds_left(x as isize - 1)
        && in_bounds_bottom(y + 1)
        && is_empty(cells, x - 1, y + 1, &empty_types)
    {
        spread_to_cell(cells, (x, y), (x - 1, y + 1))
    }

    if in_bounds_right(x + 1)
        && in_bounds_top(y as isize - 1)
        && is_empty(cells, x + 1, y - 1, &empty_types)
    {
        spread_to_cell(cells, (x, y), (x + 1, y - 1))
    }

    if in_bounds_right(x + 1)
        && in_bounds_bottom(y + 1)
        && is_empty(cells, x + 1, y + 1, &empty_types)
    {
        spread_to_cell(cells, (x, y), (x + 1, y + 1))
    }

    cells[x][y] = Cell::from(CellType::Air);
}

fn update_sand(cells: &mut [Vec<Cell>], x: usize, y: usize, rng: &Rng) {
    generic_fall(cells, x, y, rng, &[CellType::Air, CellType::Water]);
}

fn update_water(cells: &mut [Vec<Cell>], x: usize, y: usize, rng: &Rng) {
    // todo something like: if the cell has a low velocity falling down then randomly spread to the side, will stop some water cells standing on top of others without spreading i think

    let empty_types = [CellType::Air];

    if generic_fall(cells, x, y, rng, &[CellType::Air]) {
        return;
    }

    let spread_factor = 10;

    let furthest_left = furthest_by_vector(cells, x, y, spread_factor, &empty_types, (-1, 0));
    let furthest_right = furthest_by_vector(cells, x, y, spread_factor, &empty_types, (1, 0));

    if furthest_left.is_some() && furthest_right.is_some() {
        if rng.bool() {
            swap_cells(cells, (x, y), furthest_right.unwrap());
        } else {
            swap_cells(cells, (x, y), furthest_left.unwrap());
        }
    } else if furthest_left.is_some() {
        swap_cells(cells, (x, y), furthest_left.unwrap());
    } else if furthest_right.is_some() {
        swap_cells(cells, (x, y), furthest_right.unwrap());
    }
}

fn generic_fall(
    cells: &mut [Vec<Cell>],
    x: usize,
    y: usize,
    rng: &Rng,
    fall_through_types: &[CellType],
) -> bool {
    if let Some(furthest_down) = furthest_by_vector(
        cells,
        x,
        y,
        cells[x][y].velocity as usize,
        fall_through_types,
        (0, 1),
    ) {
        cells[x][y].velocity = (cells[x][y].velocity + ACCELERATION).min(TERMINAL_VELOCITY);
        swap_cells(cells, (x, y), furthest_down);

        return true;
    }

    let furthest_down_left = furthest_by_vector(
        cells,
        x,
        y,
        cells[x][y].velocity as usize,
        fall_through_types,
        (-1, 1),
    );
    let furthest_down_right = furthest_by_vector(
        cells,
        x,
        y,
        cells[x][y].velocity as usize,
        fall_through_types,
        (1, 1),
    );

    if furthest_down_left.is_some() && furthest_down_right.is_some() {
        if rng.bool() {
            cells[x][y].velocity = (cells[x][y].velocity + ACCELERATION).min(TERMINAL_VELOCITY);
            swap_cells(cells, (x, y), furthest_down_left.unwrap());
        } else {
            cells[x][y].velocity = (cells[x][y].velocity + ACCELERATION).min(TERMINAL_VELOCITY);
            swap_cells(cells, (x, y), furthest_down_right.unwrap());
        }

        return true;
    } else if furthest_down_left.is_some() {
        cells[x][y].velocity = (cells[x][y].velocity + ACCELERATION).min(TERMINAL_VELOCITY);
        swap_cells(cells, (x, y), furthest_down_left.unwrap());
        return true;
    } else if furthest_down_right.is_some() {
        cells[x][y].velocity = (cells[x][y].velocity + ACCELERATION).min(TERMINAL_VELOCITY);
        swap_cells(cells, (x, y), furthest_down_right.unwrap());
        return true;
    }

    // if we didnt move then turn down velocity
    cells[x][y].velocity /= 2.0;

    false
}

fn swap_cells(cells: &mut [Vec<Cell>], cell_1_pos: (usize, usize), cell_2_pos: (usize, usize)) {
    let temp_cell = cells[cell_1_pos.0][cell_1_pos.1];
    cells[cell_1_pos.0][cell_1_pos.1] = cells[cell_2_pos.0][cell_2_pos.1];
    cells[cell_2_pos.0][cell_2_pos.1] = temp_cell;
    cells[cell_1_pos.0][cell_1_pos.1].moved = false;
    cells[cell_2_pos.0][cell_2_pos.1].moved = true;
}

fn spread_to_cell(cells: &mut [Vec<Cell>], cell_1_pos: (usize, usize), cell_2_pos: (usize, usize)) {
    cells[cell_2_pos.0][cell_2_pos.1] = cells[cell_1_pos.0][cell_1_pos.1];
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
) -> Option<(usize, usize)> {
    assert!(direction.0.abs() <= 1 && direction.1.abs() <= 1);

    let mut closest = None;
    // leave the + 1; trust me
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
            closest = Some((current_cell.0 as usize, current_cell.1 as usize));
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
    // let starting_pixel_2d = (5, 5);
    // let spacing = 2;
    // let starting_pixel_1d = starting_pixel_2d.1 * WIDTH + starting_pixel_2d.0;

    // for (cell_index, cell) in all::<CellType>().enumerate() {
    // pixels.nth(starting_pixel_1d + spacing * WIDTH * cell_index).unwrap().copy_from_slice(&[0xff, 0xff, 0xff, 0xff]);
    // }

    // for i in 0..WIDTH * 4 {
    //     frame[i] = 0xff;
    // pixels[2].unwrap().copy_from_slice(&[0xff, 0xff, 0xff, 0xff])
    // }
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
                CellType::Fire => [0xc3, 0x3e, 0x05, 0xff],
            };

            pixels.next().unwrap().copy_from_slice(&color);
        }
    }

    draw_menu(frame, selected_cell_type)
}

fn put_cell(
    cells: &mut [Vec<Cell>],
    selected_cell_type: CellType,
    mouse_pos: (usize, usize),
    cursor_radius: f32,
    rng: &Rng,
) {
    let mouse_pos = (mouse_pos.0 as i32, mouse_pos.1 as i32);
    let cursor_radius = cursor_radius as i32;
    let cell = Cell::from(selected_cell_type);

    for x in
        ((mouse_pos.0 - cursor_radius).max(0))..((mouse_pos.0 + cursor_radius).min(WIDTH as i32))
    {
        for y in ((mouse_pos.1 - cursor_radius).max(0))
            ..((mouse_pos.1 + cursor_radius).min(HEIGHT as i32))
        {
            if (mouse_pos.0 - x).pow(2) + (mouse_pos.1 - y).pow(2) <= cursor_radius.pow(2) {
                match selected_cell_type {
                    CellType::Sand | CellType::Water => {
                        if rng.f32() > 0.125 {
                            continue;
                        }
                    }
                    _ => (),
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

    let mut cells = vec![vec![Cell::from(CellType::Air); HEIGHT]; WIDTH];
    let mut cursor_radius = 3_f32;
    let mut current_cell_type = CellType::Sand;
    let rng = Rng::new();
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

                draw_frame(&mut pixels, &cells, current_cell_type);
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
