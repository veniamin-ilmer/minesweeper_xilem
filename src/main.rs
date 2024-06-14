use xilem;
use xilem::view;
use masonry::{dpi, widget};
use winit::window;

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
}

#[derive(Clone, Copy)]
struct Cell {
  status: CellStatus,
  value: CellValue,
}

struct Game {
	board: [[Cell; CELL_ROWS]; CELL_COLUMNS],
	playing: bool,
	revealed_count: usize,
}

fn with_surrounding_cells<F>(x: usize, y: usize, mut f: F) where F: FnMut(usize, usize) {
  let first_y = y == 0;
  let last_y = y == CELL_ROWS - 1;
  let first_x = x == 0;
  let last_x = x == CELL_COLUMNS - 1;
  
  if !first_x && !first_y { f(x - 1, y - 1) }
  if !first_x { f(x - 1, y) }
  if !first_y { f(x, y - 1) }
  if !last_x && !last_y { f(x + 1, y + 1) }
  if !last_x { f(x + 1, y) }
  if !last_y { f(x, y + 1) }
  if !first_x && !last_y { f(x - 1, y + 1) }
  if !last_x && !first_y { f(x + 1, y - 1) }
}

impl Game {

	fn new() -> Game {
		let mut game = Game {
			board: [[Cell {status: CellStatus::Covered, value: CellValue::Number(0)}; CELL_ROWS]; CELL_COLUMNS],
			playing: true,
			revealed_count: 0,
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
        self.playing = false;
        return;
      }

      self.revealed_count += 1;
      if self.revealed_count >= CELL_ROWS * CELL_COLUMNS - MINE_COUNT {
        //All numbers were revealed
        self.playing = false;
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
}

fn main() {
	let app = xilem::Xilem::new(Game::new(), app_logic);
  let window_attributes = window::Window::default_attributes()
    .with_title("Minesweeper")
    .with_inner_size(dpi::LogicalSize::new(34 * CELL_COLUMNS as u32, 45 + 34 * CELL_ROWS as u32));
	app.run_windowed_in(xilem::EventLoop::with_user_event(), window_attributes).unwrap();
}

fn app_logic(game: &mut Game) -> impl xilem::WidgetView<Game> {
  let top_row = view::flex(view::button("New Game", move |game: &mut Game| {*game = Game::new()}))
    .main_axis_alignment(widget::MainAxisAlignment::Center)
    .direction(xilem::Axis::Horizontal);
	let mut rows = vec![];
	for y in 0..CELL_ROWS {
		let mut columns = vec![];
	  for x in 0..CELL_COLUMNS {
			let cell: Box<xilem::AnyWidgetView<_>> = match game.board[x][y] {
				Cell {status: CellStatus::Covered, .. } => if game.playing {
					Box::new(view::button("  ", move |game: &mut Game| game.reveal_multiple(x, y)))
        } else {
					Box::new(view::label(
						if game.board[x][y].value == CellValue::Mined { "  B  " } else { "      " }
					))
				}
				Cell {status: CellStatus::Revealed, value: CellValue::Mined} => Box::new(view::label("  B  ")),
				Cell {status: CellStatus::Revealed, value: CellValue::Number(0)} => Box::new(view::label("      ")),
				Cell {status: CellStatus::Revealed, value: CellValue::Number(number)} => Box::new(view::label(format!("  {}  ", number))),
			};
			columns.push(cell);
		}
		rows.push(view::flex(columns).direction(xilem::Axis::Horizontal));
	}
	view::flex((top_row, rows)).direction(xilem::Axis::Vertical)
}