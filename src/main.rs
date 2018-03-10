extern crate rand;
extern crate sdl2;

use std::time::Duration;
use std::time::Instant;

use rand::distributions::IndependentSample;
use rand::distributions::Range;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

const WIN_WIDTH: usize = 512;
const WIN_HEIGHT: usize = 768;
const GAME_WIDTH: usize = 16;
const CELL_SIZE: usize = WIN_WIDTH / GAME_WIDTH;
const GAP_WIDTH: i32 = 1;

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
    world: Vec<[u8; GAME_WIDTH]>,
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

fn advance_game(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, game: &mut Game) -> bool {
    let piece = game.effective_piece();
    let mut collision = false;

    if game.y == 0 {
        collision = true;
    } else {
        for j in 0..piece.height {
            if game.y + j <= game.world.len() {
                for i in 0..piece.width {
                    if piece.cells[piece.height - j - 1][i]
                        && game.world[game.y + j - 1][game.x + i] != 0
                    {
                        collision = true;
                        break;
                    }
                }
            }
        }
    }

    if collision {
        for j in 0..piece.height {
            if game.y + j == game.world.len() {
                game.world.push([0; GAME_WIDTH]);
            };

            let mut row = &mut game.world[game.y + j];

            for (i, cell) in piece.cells[piece.height - j - 1].iter().enumerate() {
                if *cell {
                    row[game.x + i] = game.piece_index as u8 + 1;
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

        game.y = 24 - 4;
        game.x = 6;
        game.orient = 0;
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
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    for (j, row) in game.world.iter().rev().enumerate() {
        for (i, cell) in row.iter().enumerate() {
            if *cell != 0 {
                render_block(
                    canvas,
                    (i * CELL_SIZE) as i32,
                    ((24 - game.world.len() + j) * CELL_SIZE) as i32,
                    COLORS[(*cell - 1) as usize],
                );
            }
        }
    }

    let piece = game.effective_piece();
    let x = (game.x * CELL_SIZE) as i32;
    let y = WIN_HEIGHT as i32 - ((game.y + piece.height) * CELL_SIZE) as i32;
    render_piece(canvas, x, y, piece, COLORS[game.piece_index]);

    canvas.present();
}

fn render_piece(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    x: i32,
    y: i32,
    piece: Piece,
    color: Color,
) {
    for (j, row) in piece.cells.iter().enumerate() {
        for (i, cell) in row.iter().enumerate() {
            if *cell {
                render_block(
                    canvas,
                    x + (i * CELL_SIZE) as i32,
                    y + (j * CELL_SIZE) as i32,
                    color,
                );
            }
        }
    }
}

fn render_block(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    x: i32,
    y: i32,
    color: Color,
) {
    canvas.set_draw_color(Color::RGB(color.r / 2, color.g / 2, color.b / 2));

    canvas
        .fill_rect(Rect::new(x, y, CELL_SIZE as u32, CELL_SIZE as u32))
        .unwrap();

    canvas.set_draw_color(color);

    canvas
        .fill_rect(Rect::new(
            x + GAP_WIDTH / 2,
            y + GAP_WIDTH / 2,
            CELL_SIZE as u32 - GAP_WIDTH as u32,
            CELL_SIZE as u32 - GAP_WIDTH as u32,
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

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let num_pieces = Range::new(0, 7);
    let mut rng = rand::thread_rng();

    let mut game = Game {
        world: Vec::new(),
        piece_index: num_pieces.ind_sample(&mut rng),
        orient: 0,
        y: 24 - 4,
        x: 6,
    };

    let mut event_pump = sdl_context.event_pump().unwrap();

    let interval = Duration::new(0, 500000000);
    let mut next_step = Instant::now() + interval;

    'running: loop {
        let now = Instant::now();
        if now >= next_step {
            if advance_game(&mut canvas, &mut game) {
                game.piece_index = num_pieces.ind_sample(&mut rng);
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
                } => move_piece(&mut game, -1),

                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => move_piece(&mut game, 1),

                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => rotate_piece(&mut game),

                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    next_step = now;
                }

                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    drop_piece(&mut canvas, &mut game);
                    game.piece_index = num_pieces.ind_sample(&mut rng);
                    game.orient = 0;
                    next_step = Instant::now() + interval;
                }

                Event::Quit { .. } => break 'running,

                _ => {}
            }
        }
    }
}
