use eframe::egui;
use rand::Rng;

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
    pub width: usize,
    pub height: usize,
    pub mine_count: usize,
    cells: Vec<Vec<Cell>>,
    pub game_over: bool,
    pub win: bool,
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
                    if self.in_bounds(nx, ny)
                        && self.cells[ny as usize][nx as usize].is_mine
                    {
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

        let cell = &mut self.cells[y][x];

        if cell.is_revealed || cell.is_flagged {
            return;
        }

        cell.is_revealed = true;

        if cell.is_mine {
            self.game_over = true;
            self.win = false;
            return;
        }

        if cell.neighbor_mines == 0 {
            self.flood_reveal(x, y);
        }

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

        for (dx, dy) in dirs {
            let nx = x as isize + dx;
            let ny = y as isize + dy;

            if !self.in_bounds(nx, ny) {
                continue;
            }

            let (ux, uy) = (nx as usize, ny as usize);
            let cell = &mut self.cells[uy][ux];

            if !cell.is_revealed && !cell.is_flagged {
                cell.is_revealed = true;

                if cell.neighbor_mines == 0 && !cell.is_mine {
                    self.flood_reveal(ux, uy);
                }
            }
        }
    }

    fn toggle_flag(&mut self, x: usize, y: usize) {
        if x < self.width && y < self.height {
            let cell = &mut self.cells[y][x];
            if !cell.is_revealed {
                cell.is_flagged = !cell.is_flagged;
            }
        }
    }

    fn check_win(&self) -> bool {
        for row in &self.cells {
            for cell in row {
                if !cell.is_mine && !cell.is_revealed {
                    return false;
                }
            }
        }
        true
    }

    fn reveal_all(&mut self) {
        for row in &mut self.cells {
            for cell in row {
                cell.is_revealed = true;
            }
        }
    }

    /// Return the string to show for a cell: "■", "🚩", "💣", "1", "2", ...
    fn cell_label(&self, x: usize, y: usize) -> String {
        let cell = &self.cells[y][x];
        if cell.is_revealed {
            if cell.is_mine {
                "💣".to_string()
            } else if cell.neighbor_mines == 0 {
                " ".to_string()
            } else {
                cell.neighbor_mines.to_string()
            }
        } else if cell.is_flagged {
            "🚩".to_string()
        } else {
            "■".to_string()
        }
    }
}

// ---------------- DIFFICULTY ----------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum Difficulty {
    Beginner,
    Intermediate,
    Expert,
}

impl Difficulty {
    fn params(self) -> (usize, usize, usize) {
        match self {
            Difficulty::Beginner => (9, 9, 10),
            Difficulty::Intermediate => (16, 16, 40),
            Difficulty::Expert => (25, 25, 99),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Difficulty::Beginner => "Beginner (9x9)",
            Difficulty::Intermediate => "Intermediate (16x16)",
            Difficulty::Expert => "Expert (25x25)",
        }
    }
}

// ---------------- CONFETTI ----------------

struct Particle {
    pos: egui::Pos2,
    vel: egui::Vec2,
    color: egui::Color32,
    lifetime: f32,
}

// ---------------- GUI APP ----------------

struct MinesweeperApp {
    board: Board,
    difficulty: Difficulty,
    celebrating: bool,
    confetti: Vec<Particle>,
    in_game: bool, // false = start menu, true = playing
}

impl MinesweeperApp {
    fn new() -> Self {
        let difficulty = Difficulty::Beginner;
        let (width, height, mines) = difficulty.params();
        Self {
            board: Board::new(width, height, mines),
            difficulty,
            celebrating: false,
            confetti: Vec::new(),
            in_game: false, // start on menu screen
        }
    }

    fn reset(&mut self) {
        let (w, h, m) = self.difficulty.params();
        self.board = Board::new(w, h, m);
        self.celebrating = false;
        self.confetti.clear();
    }

    fn set_difficulty(&mut self, difficulty: Difficulty) {
        self.difficulty = difficulty;
        self.reset();
    }

    fn start_game_with(&mut self, difficulty: Difficulty) {
        self.set_difficulty(difficulty);
        self.in_game = true;
    }

    fn back_to_menu(&mut self) {
        self.in_game = false;
        self.celebrating = false;
        self.confetti.clear();
        // keep last selected difficulty
    }

    fn start_celebration(&mut self, ctx: &egui::Context) {
        self.celebrating = true;
        self.confetti.clear();

        // spawn confetti from top of screen
        let rect = ctx.screen_rect();
        let mut rng = rand::thread_rng();

        for _ in 0..200 {
            let x = rng.gen_range(rect.left()..rect.right());
            let y = rng.gen_range(rect.top()..(rect.top() + 40.0));
            let vx = rng.gen_range(-40.0..40.0);
            let vy = rng.gen_range(50.0..150.0);

            let colors = [
                egui::Color32::RED,
                egui::Color32::GREEN,
                egui::Color32::BLUE,
                egui::Color32::YELLOW,
                egui::Color32::from_rgb(255, 0, 255),
                egui::Color32::from_rgb(0, 255, 255),
            ];
            let color = colors[rng.gen_range(0..colors.len())];

            self.confetti.push(Particle {
                pos: egui::pos2(x, y),
                vel: egui::vec2(vx, vy),
                color,
                lifetime: rng.gen_range(1.0..3.0),
            });
        }
    }

    fn update_confetti(&mut self, ctx: &egui::Context) {
        if !self.celebrating {
            return;
        }

        // In your egui version, stable_dt is already f32
        let dt = ctx.input(|i| i.stable_dt);
        let gravity = 200.0;

        for p in &mut self.confetti {
            p.vel.y += gravity * dt;
            p.pos += p.vel * dt;
            p.lifetime -= dt;
        }

        self.confetti.retain(|p| p.lifetime > 0.0);

        if self.confetti.is_empty() {
            self.celebrating = false;
        }

        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("confetti"),
        ));

        for p in &self.confetti {
            let size = egui::vec2(4.0, 8.0);
            let rect = egui::Rect::from_center_size(p.pos, size);
            painter.rect_filled(rect, 1.0, p.color);
        }
    }
}

impl Default for MinesweeperApp {
    fn default() -> Self {
        Self::new()
    }
}

impl eframe::App for MinesweeperApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.in_game {
            // -------- START MENU --------
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.heading("Bosnia Simulator");
                    ui.label("Version 1.0");
                    ui.label("By Rylan Hillman");
                    ui.add_space(20.0);
                    ui.separator();
                    ui.add_space(10.0);
                    ui.label("Select difficulty to start:");

                    ui.add_space(10.0);
                    if ui.button("Beginner (9x9)").clicked() {
                        self.start_game_with(Difficulty::Beginner);
                    }
                    if ui.button("Intermediate (16x16)").clicked() {
                        self.start_game_with(Difficulty::Intermediate);
                    }
                    if ui.button("Expert (25x25)").clicked() {
                        self.start_game_with(Difficulty::Expert);
                    }
                });
            });
            return;
        }

        // -------- GAME SCREEN --------
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Bosnia Simulator");

            // Difficulty row (can change mid-game)
            ui.horizontal(|ui| {
                ui.label("Difficulty:");

                for diff in [Difficulty::Beginner, Difficulty::Intermediate, Difficulty::Expert] {
                    let selected = self.difficulty == diff;
                    if ui
                        .selectable_label(selected, diff.label())
                        .clicked()
                    {
                        self.set_difficulty(diff);
                    }
                }

                if ui.button("Back to Menu").clicked() {
                    self.back_to_menu();
                }
            });

            // Controls row
            ui.horizontal(|ui| {
                if ui.button("New Game").clicked() {
                    self.reset();
                }

                ui.label(format!("Mines: {}", self.board.mine_count));

                if self.board.game_over {
                    if self.board.win {
                        ui.colored_label(egui::Color32::GREEN, "You win! 🎉");
                    } else {
                        ui.colored_label(egui::Color32::RED, "You hit a mine! 💥");
                    }
                }
            });

            ui.separator();

            // Board grid
            for y in 0..self.board.height {
                ui.horizontal(|ui| {
                    for x in 0..self.board.width {
                        let label = self.board.cell_label(x, y);
                        let button = egui::Button::new(label)
                            .min_size(egui::vec2(28.0, 28.0));
                        let response = ui.add(button);

                        if !self.board.game_over {
                            // Left click = reveal
                            if response.clicked() {
                                self.board.reveal_cell(x, y);
                            }
                            // Right click = flag
                            if response.secondary_clicked() {
                                self.board.toggle_flag(x, y);
                            }
                        }
                    }
                });
            }

            // Trigger confetti once on win
            if self.board.game_over && self.board.win && !self.celebrating {
                self.start_celebration(ctx);
            }

            if self.board.game_over && !self.board.win {
                if ui.button("Reveal all").clicked() {
                    self.board.reveal_all();
                }
            }
        });

        // Draw and animate confetti on top
        self.update_confetti(ctx);
    }
}

// ---------------- ENTRY POINT ----------------

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "Bosnia Simulator", // window title
        options,
        Box::new(|_cc| Ok(Box::new(MinesweeperApp::default()))),
    )
}

