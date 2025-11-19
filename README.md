# Bosnia Simulator (Rust Minesweeper GUI)

**Version 1.0**  
_By Rylan Hillman_

A native desktop Minesweeper-style game built in Rust using `egui`/`eframe`.  
Choose your difficulty, flag mines, avoid the bombs, and win to see a confetti celebration!

![Screenshot of game](assets/screenshot.png)

---

## Features

- Beginner (9×9, 10 mines)  
- Intermediate (16×16, 40 mines)  
- Expert (25×25, 99 mines)  
- Start menu with difficulty selection  
- GUI with clickable grid, right-click flags  
- Celebration confetti animation on win  
- “Bosnia Simulator” theming (title + version + author)  

---

## Getting Started

### Prerequisites

- Rust (stable)  
- On Windows: Visual Studio Build Tools with **Desktop development with C++** (required for eframe native builds)

### Build & Run

```bash
cargo run
