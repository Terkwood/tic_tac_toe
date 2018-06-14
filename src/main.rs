//! A collection of semi-random shape and image drawing examples.

extern crate ggez;

use ggez::conf;
use ggez::event;
use ggez::event::MouseButton;
use ggez::graphics;
use ggez::graphics::{DrawMode, Font, Point2, Text};
use ggez::timer;
use ggez::{Context, GameResult};
use std::env;
use std::path;

mod pixel_math {
    pub const SCREEN_SIZE: (f32, f32) = (800.0, 600.0);

    const DRAWABLE_SIZE: f32 = SCREEN_SIZE.0 / 8.0 * 5.0;

    pub const MARGIN: (f32, f32) = (
        (SCREEN_SIZE.0 - DRAWABLE_SIZE) / 2.0,
        (SCREEN_SIZE.1 - DRAWABLE_SIZE) / 2.0,
    );

    pub const POSITION_SIZE: (f32, f32) = (
        (SCREEN_SIZE.0 - MARGIN.0 * 2.0) / 3.0,
        (SCREEN_SIZE.1 - MARGIN.1 * 2.0) / 3.0,
    );

    pub const COLUMNS: (f32, f32) = (MARGIN.0 + POSITION_SIZE.0, MARGIN.0 + POSITION_SIZE.0 * 2.0);

    pub const ROWS: (f32, f32) = (MARGIN.1 + POSITION_SIZE.1, MARGIN.1 + POSITION_SIZE.1 * 2.0);

    pub const X_PIECE_OFFSET: (f32, f32) =
        (POSITION_SIZE.0 / 2.0 * 0.70, POSITION_SIZE.1 / 2.0 * 0.70);

    pub fn screen_to_board(x: f32, y: f32) -> Option<(u16, u16)> {
        use pixel_math::{MARGIN, SCREEN_SIZE};
        if x < MARGIN.0 || x > SCREEN_SIZE.0 - MARGIN.0 {
            None
        } else if y < MARGIN.1 || y > SCREEN_SIZE.1 - MARGIN.1 {
            None
        } else {
            let i = (x - MARGIN.0) / POSITION_SIZE.0;
            let j = (y - MARGIN.1) / POSITION_SIZE.1;
            Some((i as u16, j as u16))
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Piece {
    X,
    O,
}

struct Board {
    contents: Vec<Vec<Option<Piece>>>,
}

impl Board {
    fn new() -> Board {
        let v: Vec<Vec<Option<Piece>>> = vec![vec![None; 3]; 3];
        Board { contents: v }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Winner {
    piece: Piece,
    start: (u16, u16),
    end: (u16, u16),
}

struct MainState {
    board: Board,
    current_turn: Piece,
    winner: Option<Winner>,
    font: Font,
    message: Option<Text>,
}

impl MainState {
    fn new(_ctx: &mut Context) -> GameResult<MainState> {
        let font = graphics::Font::new(_ctx, "/DejaVuSerif.ttf", 26).unwrap();

        let s = MainState {
            board: Board::new(),
            current_turn: Piece::X,
            winner: None,
            font: font,
            message: None,
        };

        Ok(s)
    }
}

fn add_background_to_mesh(mb: &mut graphics::MeshBuilder) {
    use pixel_math::{COLUMNS, MARGIN, ROWS, SCREEN_SIZE};

    const LINE_WIDTH: f32 = 4.0;

    // board
    mb.line(
        &[
            Point2::new(MARGIN.0, ROWS.0),
            Point2::new(SCREEN_SIZE.0 - MARGIN.0, ROWS.0),
        ],
        LINE_WIDTH,
    );

    mb.line(
        &[
            Point2::new(MARGIN.0, ROWS.1),
            Point2::new(SCREEN_SIZE.0 - MARGIN.0, ROWS.1),
        ],
        LINE_WIDTH,
    );

    mb.line(
        &[
            Point2::new(COLUMNS.0, MARGIN.1),
            Point2::new(COLUMNS.0, SCREEN_SIZE.1 - MARGIN.1),
        ],
        LINE_WIDTH,
    );

    mb.line(
        &[
            Point2::new(COLUMNS.1, MARGIN.1),
            Point2::new(COLUMNS.1, SCREEN_SIZE.1 - MARGIN.1),
        ],
        LINE_WIDTH,
    );
}

fn piece_x_stroke(i: u16, j: u16, left: bool) -> Vec<Point2> {
    use pixel_math::{MARGIN, POSITION_SIZE, X_PIECE_OFFSET};
    let pi = i as f32 + 0.5;
    let pj = j as f32 + 0.5;
    let coeffs = if left {
        (-1.0, -1.0, 1.0, 1.0)
    } else {
        (1.0, -1.0, -1.0, 1.0)
    };

    vec![
        Point2::new(
            MARGIN.0 + POSITION_SIZE.0 * pi + coeffs.0 * X_PIECE_OFFSET.0,
            MARGIN.1 + POSITION_SIZE.1 * pj + coeffs.1 * X_PIECE_OFFSET.1,
        ),
        Point2::new(
            MARGIN.0 + POSITION_SIZE.0 * pi + coeffs.2 * X_PIECE_OFFSET.0,
            MARGIN.1 + POSITION_SIZE.1 * pj + coeffs.3 * X_PIECE_OFFSET.1,
        ),
    ]
}

fn piece_x_left_stroke(i: u16, j: u16) -> Vec<Point2> {
    piece_x_stroke(i, j, true)
}

fn piece_x_right_stroke(i: u16, j: u16) -> Vec<Point2> {
    piece_x_stroke(i, j, false)
}

fn piece_o_center(i: u16, j: u16) -> Point2 {
    use pixel_math::{MARGIN, POSITION_SIZE};

    Point2::new(
        MARGIN.0 + POSITION_SIZE.0 * (i as f32 + 0.5),
        MARGIN.1 + POSITION_SIZE.1 * (j as f32 + 0.5),
    )
}

fn add_pieces_to_mesh(mb: &mut graphics::MeshBuilder, board: &Board) {
    const LINE_WIDTH: f32 = 4.0;
    const O_DRAW_MODE: DrawMode = DrawMode::Line(LINE_WIDTH);
    const O_PIECE_RADIUS: f32 = 60.0;

    for (i, js) in board.contents.iter().enumerate() {
        for (j, p) in js.iter().enumerate() {
            match p {
                None => {}
                Some(Piece::X) => {
                    mb.line(&piece_x_left_stroke(i as u16, j as u16), LINE_WIDTH);
                    mb.line(&piece_x_right_stroke(i as u16, j as u16), LINE_WIDTH);
                }
                Some(Piece::O) => {
                    mb.circle(
                        O_DRAW_MODE,
                        piece_o_center(i as u16, j as u16),
                        O_PIECE_RADIUS,
                        1.0,
                    );
                }
            }
        }
    }
}

mod game_math {
    // rust % is just remainder, can yield negatives
    // implement arithmetic modulo to avoid a negative answer
    pub fn modulo(a: i16, b: i16) -> i16 {
        ((a % b) + b) % b
    }
    pub fn vec_min(v: &Vec<usize>) -> Option<usize> {
        v.iter().fold(None, move |min, x| match min {
            None => Some(*x),
            Some(y) => Some(if x < &y { *x } else { y }),
        })
    }

    pub fn vec_max(v: &Vec<usize>) -> Option<usize> {
        v.iter().fold(None, move |max, x| match max {
            None => Some(*x),
            Some(y) => Some(if x > &y { *x } else { y }),
        })
    }
}

fn find_winner(board: &Board, i: u16, j: u16) -> Option<Winner> {
    use game_math::{modulo, vec_max, vec_min};
    let iu = i as usize;
    let ju = j as usize;
    let ii = i as i16;
    let ji = j as i16;
    if let Some(piece) = board.contents[iu][ju] {
        // neighbors in row
        let left = modulo(ii - 1, 3) as usize;
        let right = modulo(ii + 1, 3) as usize;
        let left_piece = board.contents[left][ju];
        let right_piece = board.contents[right][ju];

        // neighbors in column
        let up = modulo(ji - 1, 3) as usize;
        let down = modulo(ji + 1, 3) as usize;
        let up_piece = board.contents[iu][up];
        let down_piece = board.contents[iu][down];

        // neighbors in left diag
        let lu_piece = board.contents[0][0];
        let rd_piece = board.contents[2][2];

        // neighbors in right diag
        let ru_piece = board.contents[2][0];
        let ld_piece = board.contents[0][2];

        let center_piece = board.contents[1][1];

        if left_piece == Some(piece) && right_piece == Some(piece) {
            let s = vec![left, iu, right];
            let min = vec_min(&s);
            let max = vec_max(&s);

            Some(Winner {
                piece: piece,
                start: (min.unwrap() as u16, j),
                end: (max.unwrap() as u16, j),
            })
        } else if up_piece == Some(piece) && down_piece == Some(piece) {
            let s = vec![up, ju, down];
            let min = vec_min(&s);
            let max = vec_max(&s);

            Some(Winner {
                piece: piece,
                start: (i, min.unwrap() as u16),
                end: (i, max.unwrap() as u16),
            })
        } else if lu_piece == Some(piece) && rd_piece == Some(piece) && center_piece == Some(piece)
        {
            Some(Winner {
                piece: piece,
                start: (0, 0),
                end: (2, 2),
            })
        } else if ru_piece == Some(piece) && ld_piece == Some(piece) && center_piece == Some(piece)
        {
            Some(Winner {
                piece: piece,
                start: (0, 2),
                end: (2, 0),
            })
        } else {
            None
        }
    } else {
        None
    }
}

fn build_winner_mesh(ctx: &mut Context, winner: &Winner) -> GameResult<graphics::Mesh> {
    let mb = &mut graphics::MeshBuilder::new();
    mb.line(
        &[
            Point2::new(
                (winner.start.0 as f32 + 0.5) * pixel_math::POSITION_SIZE.0 + pixel_math::MARGIN.0,
                (winner.start.1 as f32 + 0.5) * pixel_math::POSITION_SIZE.1 + pixel_math::MARGIN.1,
            ),
            Point2::new(
                (winner.end.0 as f32 + 0.5) * pixel_math::POSITION_SIZE.0 + pixel_math::MARGIN.0,
                (winner.end.1 as f32 + 0.5) * pixel_math::POSITION_SIZE.1 + pixel_math::MARGIN.1,
            ),
        ],
        12.0,
    );
    mb.build(ctx)
}

fn build_game_mesh(
    ctx: &mut Context,
    board: &Board
) -> GameResult<graphics::Mesh> {
    let mb = &mut graphics::MeshBuilder::new();

    add_background_to_mesh(mb);

    add_pieces_to_mesh(mb, board);

    mb.build(ctx)
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        const DESIRED_FPS: u32 = 60;

        while timer::check_update_time(ctx, DESIRED_FPS) {
            // hello
        }
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        _button: MouseButton,
        x: i32,
        y: i32,
    ) {
        if self.winner.is_some() {
            return;
        }
        if let Some((i, j)) = pixel_math::screen_to_board(x as f32, y as f32) {
            if let None = self.board.contents[i as usize][j as usize] {
                self.board.contents[i as usize][j as usize] = Some(self.current_turn);
            }

            if let Some(winner) = find_winner(&self.board, i, j) {
                self.winner = Some(winner);
                let msg = match winner.piece {
                    Piece::X => "X rides high above the clouds",
                    Piece::O => "O expands its kingdom beyond the horizon",
                };
                self.message = Some(Text::new(_ctx, msg, &self.font).unwrap());
            }
        }

        if self.winner.is_none() {
            self.current_turn = match self.current_turn {
                Piece::X => Piece::O,
                Piece::O => Piece::X,
            };
        }
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        const PURPLE: (u8, u8, u8) = (218, 112, 214);
        const CYAN: (u8, u8, u8) = (0, 255, 255);
        graphics::clear(ctx);

        let game_mesh = build_game_mesh(ctx, &self.board)?;
        graphics::set_color(ctx, CYAN.into())?;
        graphics::draw_ex(ctx, &game_mesh, Default::default())?;

        if let Some(msg) = &self.message {
            graphics::set_color(ctx, PURPLE.into())?;
            let dest_point = graphics::Point2::new(0.0, 0.0);
            graphics::draw(ctx, msg, dest_point, 0.0)?;
        }

        if let Some(winner) = &self.winner {
            let winner_mesh = build_winner_mesh(ctx, &winner)?;
            graphics::set_color(ctx, PURPLE.into())?;
            graphics::draw_ex(ctx, &winner_mesh, Default::default())?;
        }

        graphics::present(ctx);
        Ok(())
    }
}

pub fn main() {
    let c = conf::Conf::new();
    let ctx = &mut Context::load_from_conf("Tic Tac Toe", "ggez", c).unwrap();

    // We add the CARGO_MANIFEST_DIR/resources do the filesystems paths so
    // we we look in the cargo project for files.
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        ctx.filesystem.mount(&path, true);
    }

    println!("{}", graphics::get_renderer_info(ctx).unwrap());
    let state = &mut MainState::new(ctx).unwrap();
    if let Err(e) = event::run(ctx, state) {
        println!("Error encountered: {}", e);
    } else {
        println!("Game exited cleanly.");
    }
}
