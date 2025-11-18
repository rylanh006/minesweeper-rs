use rand::Rng;
use std::io::{self, Write};

#[derive(Clone)]
struct Cell {
    is_mine: bool,
    is_revealed: bool,
    is_flagged: bool,
    neighbor_mines: u8,
}

impl Cell {
    fn new() -> Self {
        Cell {
            is_mine: false,
            is_revealed: false,
            is_flagged: false,
            neighbor_mines: 0,
        }
    }
}

struct Board {
    width: usize,
    height: usize,
    mine_count: usize,
    cells: Vec<Vec<Cell>>,
    game_over: bool,
    win: bool,
}

impl Board {
    fn new(width: usize, height: usize, mine_count: usize) -> Self {
        let mut board = Board {
            width,
            height,
            mine_count,
            cells: vec![vec![Cell::new(); width]; height],
            game_over: false,
            win: false,
        };
        board.place_mines();
        board.compute_neighbor_counts();
        board
    }

    fn in_bounds(&self, x: isize, y: isize) -> bool {
        x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height
    }

    fn place_mines(&mut self) {
        let mut rng = rand::thread_rng();
        let mut placed = 0;

        while placed < self.mine_count {
            let idx = rng.gen_range(0..self.width * self.height);
            let x = idx % self.width;
            let y = idx / self.width;

            if !self.cells[y][x].is_mine {
                self.cells[y][x].is_mine = true;
                placed += 1;
            }
        }
    }

    fn compute_neighbor_counts(&mut self) {
        let dirs = [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ];

        for y in 0..self.height {
            for x in 0..self.width {
                if self.cells[y][x].is_mine {
                    self.cells[y][x].neighbor_mines = 0;
                    continue;
                }

                let mut count = 0;
                for (dx, dy) in &dirs {
                    let nx = x as isize + dx;
                    let ny = y as isize + dy;
                    if self.in_bounds(nx, ny) && self.cells[ny as usize][nx as usize].is_mine {
                        count += 1;
                    }
                }
                self.cells[y][x].neighbor_mines = count;
            }
        }
    }

    fn reveal_cell(&mut self, x: usize, y: usize) {
        if x >= self.width || y >= self.height {
            return;
        }

        if self.cells[y][x].is_revealed || self.cells[y][x].is_flagged {
            return;
        }

        self.cells[y][x].is_revealed = true;

        if self.cells[y][x].is_mine {
            self.game_over = true;
            self.win = false;
            return;
        }

        // If no neighboring mines, flood reveal neighbors
        if self.cells[y][x].neighbor_mines == 0 {
            self.flood_reveal(x, y);
        }

        // After each reveal, check win condition
        if self.check_win() {
            self.game_over = true;
            self.win = true;
        }
    }

    fn flood_reveal(&mut self, x: usize, y: usize) {
        let dirs = [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ];

        for (dx, dy) in &dirs {
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            if self.in_bounds(nx, ny) {
                let (ux, uy) = (nx as usize, ny as usize);
                if !self.cells[uy][ux].is_revealed && !self.cells[uy][ux].is_flagged {
                    self.cells[uy][ux].is_revealed = true;
                    if self.cells[uy][ux].neighbor_mines == 0 && !self.cells[uy][ux].is_mine {
                        self.flood_reveal(ux, uy);
                    }
                }
            }
        }
    }

    fn toggle_flag(&mut self, x: usize, y: usize) {
        if x >= self.width || y >= self.height {
            return;
        }
        if self.cells[y][x].is_revealed {
            return;
        }
        self.cells[y][x].is_flagged = !self.cells[y][x].is_flagged;
    }

    fn check_win(&self) -> bool {
        for y in 0..self.height {
            for x in 0..self.width {
                let cell = &self.cells[y][x];
                if !cell.is_mine && !cell.is_revealed {
                    return false;
                }
            }
        }
        true
    }

    fn reveal_all(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.cells[y][x].is_revealed = true;
            }
        }
    }

    fn print(&self) {
        // column header
        print!("   ");
        for x in 0..self.width {
            print!("{:2} ", x);
        }
        println!();

        for y in 0..self.height {
            // row header
            print!("{:2} ", y);
            for x in 0..self.width {
                let cell = &self.cells[y][x];
                let ch = if cell.is_revealed {
                    if cell.is_mine {
                        '*'
                    } else if cell.neighbor_mines == 0 {
                        ' '
                    } else {
                        char::from_digit(cell.neighbor_mines as u32, 10).unwrap()
                    }
                } else if cell.is_flagged {
                    'F'
                } else {
                    '#'
                };
                print!(" {} ", ch);
            }
            println!();
        }
        println!();
    }
}

fn read_line_trimmed() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    input.trim().to_string()
}

fn main() {
    println!("=== Minesweeper (Rust CLI) ===");
    println!("Controls:");
    println!("  r x y  -> reveal cell at (x, y)");
    println!("  f x y  -> toggle flag at (x, y)");
    println!("Coordinates are zero-based.\n");

    // Choose a difficulty (classic beginner board)
    let width = 9;
    let height = 9;
    let mine_count = 10;

    let mut board = Board::new(width, height, mine_count);

    loop {
        board.print();

        if board.game_over {
            if board.win {
                println!("🎉 You win! All safe cells revealed.");
            } else {
                println!("💥 Boom! You hit a mine.");
                board.reveal_all();
                board.print();
            }
            break;
        }

        print!("Enter command (e.g., 'r 3 4' or 'f 2 1'): ");
        io::stdout().flush().unwrap();

        let line = read_line_trimmed();
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() != 3 {
            println!("Invalid command format. Use: r x y or f x y");
            continue;
        }

        let cmd = parts[0];
        let x: usize = match parts[1].parse() {
            Ok(v) => v,
            Err(_) => {
                println!("Invalid x coordinate");
                continue;
            }
        };
        let y: usize = match parts[2].parse() {
            Ok(v) => v,
            Err(_) => {
                println!("Invalid y coordinate");
                continue;
            }
        };

        match cmd {
            "r" | "R" => {
                board.reveal_cell(x, y);
            }
            "f" | "F" => {
                board.toggle_flag(x, y);
            }
            _ => {
                println!("Unknown command '{}'. Use 'r' or 'f'.", cmd);
            }
        }
    }

    println!("Thanks for playing!");
}
