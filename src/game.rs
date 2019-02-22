use piston_window as pw;

use std::collections::HashMap;

pub const TILE_SIZE: (u32, u32) = (128, 128);
pub const CELL_SIZE: (u32, u32) = (16, 16);

#[derive(Debug, PartialEq)]
pub enum CellState {
    Value(u8),
    Empty,
    Bomb,
    DeathBomb,
}

#[derive(Debug)]
pub struct Cell {
    state: CellState,
    hidden: bool,
}

pub struct Field {
    cells: HashMap<(u32, u32), Cell>,
    size: (u32, u32),
    n_bombs: usize,
    n_hidden: usize,

    mouse: (f64, f64),
    textures: Vec<pw::G2dTexture>,
}

impl Field {
    pub fn new<R: rand::Rng + ?Sized>(
        rng: &mut R,
        size: (u32, u32),
        bombs: usize,
        textures: Vec<pw::G2dTexture>,
    ) -> Field {
        let mut cells = HashMap::new();

        let bomb_indices =
            rand::seq::index::sample(rng, (size.0 * size.1) as usize, bombs).into_vec();

        for idx in 0..(size.0 * size.1) {
            cells.insert(
                (idx % size.0, idx / size.0),
                Cell {
                    state: if bomb_indices.contains(&(idx as usize)) {
                        CellState::Bomb
                    } else {
                        CellState::Empty
                    },
                    hidden: true,
                },
            );
        }

        let n_hidden = cells.len();

        let mut field = Field {
            cells,
            size,
            n_hidden,
            n_bombs: bombs,
            mouse: (0.0, 0.0),
            textures,
        };

        for y in 0..size.1 {
            for x in 0..size.0 {
                if field.cell_at(x, y).state == CellState::Bomb {
                    continue;
                }

                let n_bombs = field
                    .adjacent_cells(x, y)
                    .into_iter()
                    .filter(|c| c.state == CellState::Bomb)
                    .count();

                if n_bombs > 0 {
                    field.cell_mut_at(x, y).state = CellState::Value(n_bombs as u8);
                }
            }
        }

        field
    }

    pub fn cell_at(&self, x: u32, y: u32) -> &Cell {
        &self.cells[&(x, y)]
    }

    pub fn cell_mut_at(&mut self, x: u32, y: u32) -> &mut Cell {
        self.cells.get_mut(&(x, y)).unwrap()
    }

    fn adjacent_coords(&self, x: u32, y: u32) -> Vec<(u32, u32)> {
        let (x, y) = (i64::from(x), i64::from(y));
        let mut v = Vec::with_capacity(8);

        for dy in (y - 1).max(0)..(y + 2).min(self.size.1.into()) {
            for dx in (x - 1).max(0)..(x + 2).min(self.size.0.into()) {
                if (dx, dy) != (x, y) {
                    v.push((dx as u32, dy as u32));
                }
            }
        }
        v
    }

    fn adjacent_cells(&self, x: u32, y: u32) -> Vec<&Cell> {
        self.adjacent_coords(x, y)
            .into_iter()
            .map(|(x, y)| self.cell_at(x, y))
            .collect::<Vec<_>>()
    }

    pub fn render(&self, c: pw::Context, g: &mut pw::G2d) {
        use pw::Transformed;

        for (&(x, y), cell) in self.cells.iter() {
            let tex = if cell.hidden {
                &self.textures[0]
            } else {
                match cell.state {
                    CellState::DeathBomb => &self.textures[1],
                    CellState::Bomb => &self.textures[2],
                    CellState::Empty => &self.textures[3],
                    CellState::Value(n) => &self.textures[4 + (usize::from(n) - 1)],
                }
            };

            pw::image(
                tex,
                c.transform
                    .trans(f64::from(x * CELL_SIZE.0), f64::from(y * CELL_SIZE.1))
                    .scale(
                        f64::from(CELL_SIZE.0) / f64::from(TILE_SIZE.0),
                        f64::from(CELL_SIZE.1) / f64::from(TILE_SIZE.1),
                    ),
                g,
            );
        }
    }

    pub fn mouse_move(&mut self, [x, y]: &[f64; 2]) {
        self.mouse.0 = *x;
        self.mouse.1 = *y;
    }

    pub fn mouse_click(&mut self, b: &pw::Button) {
        use pw::{Button, Key, MouseButton};

        match b {
            Button::Mouse(MouseButton::Left) => {
                let (x, y) = (
                    (self.mouse.0 as u32) / CELL_SIZE.0,
                    (self.mouse.1 as u32) / CELL_SIZE.1,
                );

                self.reveal(x, y);

                if self.cell_at(x, y).state == CellState::Bomb {
                    self.cell_mut_at(x, y).state = CellState::DeathBomb;
                    self.lose();
                } else if self.n_hidden == self.n_bombs {
                    self.win();
                }
            }
            Button::Keyboard(Key::R) => self.reset(),
            _ => (),
        }
    }

    fn reveal(&mut self, x: u32, y: u32) {
        let c = self.cell_mut_at(x, y);

        if c.hidden {
            c.hidden = false;

            if c.state == CellState::Empty {
                for (x, y) in self.adjacent_coords(x, y).into_iter() {
                    self.reveal(x, y);
                }
            }

            self.n_hidden -= 1;
        }
    }

    fn lose(&mut self) {
        for (_, c) in self.cells.iter_mut() {
            c.hidden = false;
        }
    }

    fn win(&mut self) {
        for (_, c) in self.cells.iter_mut() {
            c.hidden = false;
        }
    }

    fn reset(&mut self) {
        let last_mouse_pos = self.mouse;
        *self = Field::new(
            &mut rand::thread_rng(),
            self.size,
            self.n_bombs,
            self.textures.clone(),
        );
        self.mouse = last_mouse_pos;
    }
}
