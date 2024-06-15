use masonry;
use masonry::{dpi, widget};
use winit::window;
use xilem;
use xilem::view;

const CELL_ROWS: usize = 16;
const CELL_COLUMNS: usize = 30;
const MINE_COUNT: usize = 99;

#[derive(Clone, Copy, PartialEq)]
enum CellValue {
    Mined,
    Number(u8),
}

#[derive(Clone, Copy, PartialEq)]
enum CellStatus {
    Covered,
    Revealed,
    Flagged,
}

#[derive(Clone, Copy)]
struct Cell {
    status: CellStatus,
    value: CellValue,
}

#[derive(PartialEq, Clone, Copy)]
enum GameStatus {
    Playing,
    Lost,
    Won,
}

struct Game {
    board: [[Cell; CELL_ROWS]; CELL_COLUMNS],
    status: GameStatus,
    revealed_count: usize,
    flag_count: usize,
}

fn with_surrounding_cells<F>(x: usize, y: usize, mut f: F)
where
    F: FnMut(usize, usize),
{
    let first_y = y == 0;
    let last_y = y == CELL_ROWS - 1;
    let first_x = x == 0;
    let last_x = x == CELL_COLUMNS - 1;

    if !first_x && !first_y {
        f(x - 1, y - 1)
    }
    if !first_x {
        f(x - 1, y)
    }
    if !first_y {
        f(x, y - 1)
    }
    if !last_x && !last_y {
        f(x + 1, y + 1)
    }
    if !last_x {
        f(x + 1, y)
    }
    if !last_y {
        f(x, y + 1)
    }
    if !first_x && !last_y {
        f(x - 1, y + 1)
    }
    if !last_x && !first_y {
        f(x + 1, y - 1)
    }
}

impl Game {
    fn new() -> Game {
        let mut game = Game {
            board: [[Cell {
                status: CellStatus::Covered,
                value: CellValue::Number(0),
            }; CELL_ROWS]; CELL_COLUMNS],
            status: GameStatus::Playing,
            revealed_count: 0,
            flag_count: 0,
        };
        game.add_mines();
        game.add_numbers();
        game
    }

    fn add_mines(&mut self) {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();

        // Create a Vec of all possible positions.
        let mut positions = Vec::new();
        for y in 0..CELL_ROWS {
            for x in 0..CELL_COLUMNS {
                positions.push((x, y));
            }
        }

        // Shuffle the Vec of positions.
        positions.shuffle(&mut rng);

        // Mine some positions.
        for &(x, y) in positions.iter().take(MINE_COUNT) {
            self.board[x][y].value = CellValue::Mined;
        }
    }

    fn add_numbers(&mut self) {
        for y in 0..CELL_ROWS {
            for x in 0..CELL_COLUMNS {
                if self.board[x][y].value == CellValue::Mined {
                    continue;
                }
                //Count up all bombs at sides and corners
                let mut count = 0;
                with_surrounding_cells(x, y, |new_x, new_y| {
                    if self.board[new_x][new_y].value == CellValue::Mined {
                        count += 1;
                    }
                });
                self.board[x][y].value = CellValue::Number(count);
            }
        }
    }

    fn reveal_multiple(&mut self, x: usize, y: usize) {
        let mut reveal_vec = vec![(x, y)];

        while let Some(cell) = reveal_vec.pop() {
            let x = cell.0;
            let y = cell.1;

            //Only reveal cells which haven't been revealed. Else we will be counting too many.
            if self.board[x][y].status != CellStatus::Covered {
                continue;
            }

            self.board[x][y].status = CellStatus::Revealed;

            if self.board[x][y].value == CellValue::Mined {
                self.board[x][y].status = CellStatus::Revealed;
                self.status = GameStatus::Lost;
                return;
            }

            self.revealed_count += 1;
            if self.revealed_count >= CELL_ROWS * CELL_COLUMNS - MINE_COUNT {
                //All numbers were revealed
                self.status = GameStatus::Won;
                return;
            }

            //Clicked on a blank piece? Reveal all sides and corners.
            if self.board[x][y].value == CellValue::Number(0) {
                with_surrounding_cells(x, y, |new_x, new_y| {
                    if self.board[new_x][new_y].status == CellStatus::Covered {
                        reveal_vec.push((new_x, new_y));
                    }
                });
            }
        }
    }

    fn flag(&mut self, x: usize, y: usize) {
        match self.board[x][y].status {
            CellStatus::Covered => {
                if MINE_COUNT == self.flag_count {
                    //Too many flags! Don't add an extra flag. (Else MNE_COUNT - self.flag_count < 0, which will cause an exception because they are unsigned.)
                    return;
                }
                self.board[x][y].status = CellStatus::Flagged;
                self.flag_count += 1;
            }
            CellStatus::Flagged => {
                self.board[x][y].status = CellStatus::Covered;
                self.flag_count -= 1;
            }
            CellStatus::Revealed => (), //If it's already revealed, it can't be flagged.
        };
    }
}

fn main() {
    let app = xilem::Xilem::new(Game::new(), app_logic);
    let window_attributes = window::Window::default_attributes()
        .with_title("Minesweeper")
        .with_inner_size(dpi::LogicalSize::new(
            32 * CELL_COLUMNS as u32,
            45 + 32 * CELL_ROWS as u32,
        ));
    app.run_windowed_in(xilem::EventLoop::with_user_event(), window_attributes)
        .unwrap();
}

fn app_logic(game: &mut Game) -> impl xilem::WidgetView<Game> {
    let face = match game.status {
        GameStatus::Playing => "^_^",
        GameStatus::Lost => "T_T",
        GameStatus::Won => "^o^",
    };
    //HACK: Adding sized boxes to make the Mines count appear left aligned and the button appear center aligned.
    let top_row = view::flex((
        view::sized_box::<Game, (), _>(view::label(format!(
            "Mines: {}",
            MINE_COUNT - game.flag_count
        )))
        .width(410.),
        view::button(face, move |game: &mut Game| *game = Game::new()),
        view::sized_box::<Game, (), _>(view::label("")).width(410.),
    ))
    .main_axis_alignment(widget::MainAxisAlignment::Center)
    .direction(xilem::Axis::Horizontal);
    let mut rows = vec![];
    for y in 0..CELL_ROWS {
        let mut columns = vec![];
        for x in 0..CELL_COLUMNS {
            let cell: Box<xilem::AnyWidgetView<_>> =
                match (game.board[x][y].status, game.board[x][y].value, game.status) {
                    (CellStatus::Flagged, _, GameStatus::Playing) => Box::new(
                        view::button_any_pointer("!", move |game: &mut Game, button| {
                            if button == masonry::PointerButton::Secondary {
                                game.flag(x, y)
                            }
                        }),
                    ),
                    (CellStatus::Flagged, _, GameStatus::Won | GameStatus::Lost) => {
                        Box::new(view::button("!", |_| {}))
                    }
                    (CellStatus::Covered, _, GameStatus::Playing) => Box::new(
                        view::button_any_pointer("", move |game: &mut Game, button| match button {
                            masonry::PointerButton::Primary => game.reveal_multiple(x, y),
                            masonry::PointerButton::Secondary => game.flag(x, y),
                            _ => (),
                        }),
                    ),
                    (CellStatus::Covered, CellValue::Mined, GameStatus::Won) => {
                        Box::new(view::button("!", |_| {}))
                    }
                    (CellStatus::Covered, CellValue::Mined, GameStatus::Lost) => {
                        Box::new(view::button("X", |_| {}))
                    }
                    (
                        CellStatus::Covered,
                        CellValue::Number(_),
                        GameStatus::Won | GameStatus::Lost,
                    ) => Box::new(view::button("", |_| {})),
                    (CellStatus::Revealed, CellValue::Mined, _) => Box::new(view::label(" X")),
                    (CellStatus::Revealed, CellValue::Number(0), _) => Box::new(view::label("")),
                    (CellStatus::Revealed, CellValue::Number(number), _) => {
                        Box::new(view::label(format!(" {}", number)))
                    }
                };
            columns.push(view::sized_box(cell).width(22.).height(22.));
        }
        rows.push(view::flex(columns).direction(xilem::Axis::Horizontal));
    }
    view::flex((top_row, rows)).direction(xilem::Axis::Vertical)
}
