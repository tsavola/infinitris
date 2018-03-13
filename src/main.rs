extern crate rand;
extern crate sdl2;

use std::f64;
use std::fs::File;
use std::fs::rename;
use std::io::Read;
use std::io::Write;
use std::time::Duration;
use std::time::Instant;

use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::BlendMode;

const GAME_WIDTH: usize = 10;
const CELL_SIZE: usize = 32;
const GAP_WIDTH: usize = 4;
const PIECE_POS: usize = 3;
const START_HEIGHT: usize = 10;
const ZOOM: usize = 4;
const CELL_BORDER: i32 = 1;
const WIN_WIDTH: usize = GAME_WIDTH * CELL_SIZE + GAP_WIDTH + GAME_WIDTH * CELL_SIZE / ZOOM;
const WIN_HEIGHT: usize = 960;

static BACKGROUND_COLOR: Color = Color {
    r: 0,
    g: 0,
    b: 0,
    a: 255,
};

struct Piece {
    width: usize,
    height: usize,
    cells: [[bool; 4]; 4],
}

static PIECES: [Piece; 7] = [
    Piece {
        width: 4,
        height: 1,
        cells: [
            [true, true, true, true],
            [false, false, false, false],
            [false, false, false, false],
            [false, false, false, false],
        ],
    },
    Piece {
        width: 3,
        height: 2,
        cells: [
            [true, true, true, false],
            [false, false, true, false],
            [false, false, false, false],
            [false, false, false, false],
        ],
    },
    Piece {
        width: 3,
        height: 2,
        cells: [
            [true, true, true, false],
            [true, false, false, false],
            [false, false, false, false],
            [false, false, false, false],
        ],
    },
    Piece {
        width: 2,
        height: 2,
        cells: [
            [true, true, false, false],
            [true, true, false, false],
            [false, false, false, false],
            [false, false, false, false],
        ],
    },
    Piece {
        width: 3,
        height: 2,
        cells: [
            [false, true, true, false],
            [true, true, false, false],
            [false, false, false, false],
            [false, false, false, false],
        ],
    },
    Piece {
        width: 3,
        height: 2,
        cells: [
            [true, true, true, false],
            [false, true, false, false],
            [false, false, false, false],
            [false, false, false, false],
        ],
    },
    Piece {
        width: 3,
        height: 2,
        cells: [
            [true, true, false, false],
            [false, true, true, false],
            [false, false, false, false],
            [false, false, false, false],
        ],
    },
];

static COLORS: [Color; 7] = [
    Color {
        r: 0,
        g: 255,
        b: 255,
        a: 255,
    },
    Color {
        r: 0,
        g: 0,
        b: 255,
        a: 255,
    },
    Color {
        r: 255,
        g: 165,
        b: 0,
        a: 255,
    },
    Color {
        r: 255,
        g: 255,
        b: 0,
        a: 255,
    },
    Color {
        r: 0,
        g: 255,
        b: 0,
        a: 255,
    },
    Color {
        r: 170,
        g: 0,
        b: 255,
        a: 255,
    },
    Color {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    },
];

struct Game {
    world: Vec<[u32; GAME_WIDTH]>,
    next_gen: u32,
    piece_index: usize,
    orient: u8,
    y: usize,
    x: usize,
}

impl Game {
    fn effective_piece(&self) -> Piece {
        effective_piece(self.piece_index, self.orient)
    }
}

fn effective_piece(index: usize, orient: u8) -> Piece {
    let orig_piece = &PIECES[index];
    let mut width = orig_piece.width;
    let mut height = orig_piece.height;
    let mut cells = orig_piece.cells;

    for _ in 0..orient {
        let orig = cells;
        cells = [
            [orig[0][3], orig[1][3], orig[2][3], orig[3][3]],
            [orig[0][2], orig[1][2], orig[2][2], orig[3][2]],
            [orig[0][1], orig[1][1], orig[2][1], orig[3][1]],
            [orig[0][0], orig[1][0], orig[2][0], orig[3][0]],
        ];

        while cells[0].iter().all(|cell| !*cell) {
            let orig = cells;
            cells = [orig[1], orig[2], orig[3], [false; 4]];
        }

        while !cells[0][0] && !cells[1][0] && !cells[2][0] && !cells[3][0] {
            let orig = cells;
            cells = [
                [orig[0][1], orig[0][2], orig[0][3], false],
                [orig[1][1], orig[1][2], orig[1][3], false],
                [orig[2][1], orig[2][2], orig[2][3], false],
                [orig[3][1], orig[3][2], orig[3][3], false],
            ];
        }

        let temp = height;
        height = width;
        width = temp;
    }

    Piece {
        width: width,
        height: height,
        cells: cells,
    }
}

fn move_piece(game: &mut Game, delta: isize) {
    let piece = game.effective_piece();

    if delta < 0 {
        if game.x == 0 {
            return;
        }
    } else {
        if game.x + piece.width == GAME_WIDTH {
            return;
        }
    };

    for j in 0..piece.height {
        if game.y + j < game.world.len() {
            for i in 0..piece.width {
                if piece.cells[piece.height - j - 1][i]
                    && game.world[game.y + j][((game.x + i) as isize + delta) as usize] != 0
                {
                    return;
                }
            }
        }
    }

    game.x = (game.x as isize + delta) as usize;
}

fn rotate_piece(game: &mut Game) {
    let new_orient = (game.orient + 3) % 4;
    let new_piece = effective_piece(game.piece_index, new_orient);

    if game.x + new_piece.width > GAME_WIDTH {
        return;
    }

    for j in 0..new_piece.height {
        if game.y + j < game.world.len() {
            for i in 0..new_piece.width {
                if new_piece.cells[new_piece.height - j - 1][i]
                    && game.world[game.y + j][game.x + i] != 0
                {
                    return;
                }
            }
        }
    }

    game.orient = new_orient;
}

fn detect_collision(game: &Game, piece_y: usize, piece: &Piece) -> bool {
    if piece_y == 0 {
        true
    } else {
        for j in 0..piece.height {
            if piece_y + j <= game.world.len() {
                for i in 0..piece.width {
                    if piece.cells[piece.height - j - 1][i]
                        && game.world[piece_y + j - 1][game.x + i] != 0
                    {
                        return true;
                    }
                }
            }
        }

        false
    }
}

fn advance_game(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, game: &mut Game) -> bool {
    let piece = game.effective_piece();
    let collision = detect_collision(game, game.y, &piece);

    if collision {
        for j in 0..piece.height {
            if game.y + j == game.world.len() {
                game.world.push([0; GAME_WIDTH]);
            };

            let mut row = &mut game.world[game.y + j];

            for (i, cell) in piece.cells[piece.height - j - 1].iter().enumerate() {
                if *cell {
                    row[game.x + i] = game.next_gen;
                }
            }
        }

        for j in (0..piece.height).rev() {
            let mut gaps = false;

            for cell in game.world[game.y + j].iter() {
                if *cell == 0 {
                    gaps = true;
                    break;
                }
            }

            if !gaps {
                game.world.remove(game.y + j);
                render_game(canvas, game);
            }
        }

        game.next_gen += 1;
        game.y = game.world.len() + START_HEIGHT;
        game.x = (GAME_WIDTH - 4) / 2;
        game.orient = 0;

        let mut file = File::create(".infinitris.state.tmp").unwrap();
        file.write(&[1, 0, 0, 0, 0, 0, 0, 0]).unwrap();
        for row in game.world.iter() {
            for cell in row.iter() {
                file.write(&[
                    ((*cell >> 24) & 0xff) as u8,
                    ((*cell >> 16) & 0xff) as u8,
                    ((*cell >> 8) & 0xff) as u8,
                    (*cell & 0xff) as u8,
                ]).unwrap();
            }
        }
        file.flush().unwrap();
        rename(".infinitris.state.tmp", "infinitris.state").unwrap();
    } else {
        game.y -= 1;
    }

    collision
}

fn drop_piece(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, game: &mut Game) {
    while !advance_game(canvas, game) {
        render_game(canvas, game);
    }
}

fn render_game(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, game: &Game) {
    canvas.set_draw_color(BACKGROUND_COLOR);
    canvas.clear();

    let piece = game.effective_piece();
    let piece_x = game.x * CELL_SIZE;
    let piece_y = (PIECE_POS + 4 - piece.height) * CELL_SIZE;

    let world_y = ((PIECE_POS + 4) * CELL_SIZE) as i32
        + (game.y as i32 - game.world.len() as i32) * CELL_SIZE as i32;

    for (j, row) in game.world.iter().rev().enumerate() {
        for (i, cell) in row.iter().enumerate() {
            if *cell != 0 {
                let age = *cell as f64 / game.next_gen as f64;

                let color = Color::RGB(
                    (64.0 + (0.5 * f64::consts::PI * age).sin() * 127.0) as u8,
                    (160.0 * age + 32.0 * (64.0 * f64::consts::PI * age).sin()) as u8,
                    (64.0 + (0.5 * f64::consts::PI * age).cos() * 127.0) as u8,
                );

                render_block(
                    canvas,
                    CELL_SIZE as u32,
                    (i * CELL_SIZE) as i32,
                    world_y + (j * CELL_SIZE) as i32,
                    color,
                );

                render_block(
                    canvas,
                    (CELL_SIZE / ZOOM) as u32,
                    (GAME_WIDTH * CELL_SIZE + GAP_WIDTH + i * CELL_SIZE / ZOOM) as i32,
                    ((PIECE_POS + j) * CELL_SIZE / ZOOM) as i32,
                    color,
                );
            }
        }
    }

    canvas.set_draw_color(Color::RGB(31, 31, 31));

    canvas
        .fill_rect(Rect::new(
            0,
            world_y + (game.world.len() * CELL_SIZE) as i32,
            (GAME_WIDTH * CELL_SIZE) as u32,
            WIN_HEIGHT as u32,
        ))
        .unwrap();

    canvas
        .fill_rect(Rect::new(
            (GAME_WIDTH * CELL_SIZE) as i32,
            0,
            GAP_WIDTH as u32,
            WIN_HEIGHT as u32,
        ))
        .unwrap();

    canvas
        .fill_rect(Rect::new(
            (GAME_WIDTH * CELL_SIZE + GAP_WIDTH) as i32,
            ((PIECE_POS + game.world.len()) * CELL_SIZE / ZOOM) as i32,
            (GAME_WIDTH * CELL_SIZE / ZOOM) as u32,
            WIN_HEIGHT as u32,
        ))
        .unwrap();

    let mut shadow_distance: usize = 0;
    loop {
        if detect_collision(game, game.y - shadow_distance, &piece) {
            break;
        }
        shadow_distance += 1;
    }

    render_piece(
        canvas,
        CELL_SIZE,
        piece_x as i32,
        (piece_y + shadow_distance * CELL_SIZE) as i32,
        &piece,
        Color::RGBA(63, 63, 63, 15),
    );

    render_piece(
        canvas,
        CELL_SIZE,
        piece_x as i32,
        piece_y as i32,
        &piece,
        COLORS[game.piece_index],
    );

    canvas.present();
}

fn render_piece(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    size: usize,
    x: i32,
    y: i32,
    piece: &Piece,
    color: Color,
) {
    for (j, row) in piece.cells.iter().enumerate() {
        for (i, cell) in row.iter().enumerate() {
            if *cell {
                render_block(
                    canvas,
                    size as u32,
                    x + (i * size) as i32,
                    y + (j * size) as i32,
                    color,
                );
            }
        }
    }
}

fn render_block(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    size: u32,
    x: i32,
    y: i32,
    color: Color,
) {
    canvas.set_draw_color(Color::RGBA(color.r, color.g, color.b, 127));

    canvas.fill_rect(Rect::new(x, y, size, size)).unwrap();

    canvas.set_draw_color(color);

    canvas
        .fill_rect(Rect::new(
            x + CELL_BORDER / 2,
            y + CELL_BORDER / 2,
            size - CELL_BORDER as u32,
            size - CELL_BORDER as u32,
        ))
        .unwrap();
}

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("sy", WIN_WIDTH as u32, WIN_HEIGHT as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window
        .into_canvas()
        .target_texture()
        .present_vsync()
        .build()
        .unwrap();

    canvas.set_blend_mode(BlendMode::Blend);
    canvas.set_draw_color(BACKGROUND_COLOR);
    canvas.clear();
    canvas.present();

    let mut rng = rand::thread_rng();
    let mut piece_pool = Vec::new();
    let mut next_piece_index = || {
        if piece_pool.len() == 0 {
            for piece in 0..7 {
                for _ in 0..4 {
                    piece_pool.push(piece);
                }
            }
        }

        *rng.choose(&piece_pool).unwrap()
    };

    let mut game = Game {
        world: Vec::new(),
        next_gen: 1,
        piece_index: next_piece_index(),
        orient: 0,
        y: START_HEIGHT,
        x: (GAME_WIDTH - 4) / 2,
    };

    match File::open("infinitris.state") {
        Ok(mut file) => {
            let mut header = [0u8; 8];

            let size = file.read(&mut header).unwrap();
            if size < header.len() {
                panic!("Invalid state (no header)");
            }

            let version = header[0];
            match version {
                1 => {
                    let mut max_gen: u32 = 0;

                    loop {
                        let mut bytes = [0u8; GAME_WIDTH * 4];

                        let size = file.read(&mut bytes).unwrap();
                        if size == 0 {
                            break;
                        }

                        if size != GAME_WIDTH * 4 {
                            panic!("Invalid state (read length {})", size);
                        }

                        let mut row = [0u32; GAME_WIDTH];

                        for (i, cell) in row.iter_mut().enumerate() {
                            *cell = ((bytes[i * 4 + 0] as u32) << 24)
                                | ((bytes[i * 4 + 1] as u32) << 16)
                                | ((bytes[i * 4 + 2] as u32) << 8)
                                | (bytes[i * 4 + 3] as u32);
                            max_gen = u32::max(max_gen, *cell);
                        }

                        game.world.push(row);
                    }

                    game.next_gen = max_gen + 1;
                }

                _ => panic!("Invalid state (version {})", version),
            }

            game.y = game.world.len() + START_HEIGHT;
        }

        Err(_) => {}
    };

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut interaction = false;

    let interval = Duration::new(0, 500000000);
    let mut next_step = Instant::now() + interval;

    'running: loop {
        let mut pause = false;

        let now = Instant::now();
        if now >= next_step {
            if advance_game(&mut canvas, &mut game) {
                if !interaction {
                    pause = true;
                }
                interaction = false;
                game.piece_index = next_piece_index();
                game.orient = 0;
            }
            next_step = now + interval;
        }

        render_game(&mut canvas, &game);

        for event in event_pump.poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    move_piece(&mut game, -1);
                    interaction = true;
                }

                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    move_piece(&mut game, 1);
                    interaction = true;
                }

                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    rotate_piece(&mut game);
                    interaction = true;
                }

                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    interaction = true;
                    next_step = now;
                }

                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    interaction = true;
                    drop_piece(&mut canvas, &mut game);
                    game.piece_index = next_piece_index();
                    game.orient = 0;
                    next_step = Instant::now() + interval;
                }

                Event::KeyDown {
                    keycode: Some(Keycode::P),
                    ..
                } => {
                    pause = true;
                }

                Event::Quit { .. } => {
                    break 'running;
                }

                _ => {}
            }
        }

        if pause {
            canvas.set_draw_color(Color::RGBA(0, 0, 0, 191));
            canvas
                .fill_rect(Rect::new(0, 0, WIN_WIDTH as u32, WIN_HEIGHT as u32))
                .unwrap();

            'paused: loop {
                canvas.present();

                match event_pump.wait_event() {
                    Event::KeyDown {
                        keycode: Some(Keycode::P),
                        ..
                    } => {
                        break 'paused;
                    }

                    Event::Quit { .. } => {
                        break 'running;
                    }

                    _ => {}
                }
            }

            interaction = true;
            next_step = Instant::now() + interval;
        }
    }
}
