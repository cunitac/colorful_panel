use {
    rand::prelude::*,
    tetra::{
        graphics::{
            self,
            mesh::{Mesh, ShapeStyle},
            Color, DrawParams, Rectangle,
        },
        input::{self, Key, MouseButton},
        math::Vec2,
        Context, ContextBuilder,
    },
};

const N: usize = 30;
const WINDOW_SIZE: f32 = 600.0;
const WINDOW_MARGIN: f32 = 10.0;
const CELL_SIZE: f32 = (WINDOW_SIZE - WINDOW_MARGIN * 2.0) / N as f32;

fn main() -> tetra::Result {
    ContextBuilder::new("Colorful Panel", WINDOW_SIZE as i32, WINDOW_SIZE as i32)
        .quit_on_escape(true)
        .build()?
        .run(State::new)
}

struct State {
    cells: Vec<Vec<Cell>>,
    action: Option<Action>,
    num_changed: usize,
}

impl State {
    fn new(_ctx: &mut Context) -> tetra::Result<State> {
        let mut rng = thread_rng();
        let cells = Rectangle::row(WINDOW_MARGIN, WINDOW_MARGIN, CELL_SIZE, CELL_SIZE)
            .take(N)
            .map(|origin| {
                Rectangle::column(origin.x, origin.y, origin.width, origin.height)
                    .take(N)
                    .map(|rectangle| Cell {
                        rectangle,
                        color: rng.gen_range(0..9),
                    })
                    .collect()
            })
            .collect();

        Ok(State {
            cells,
            action: None,
            num_changed: 0,
        })
    }
}

impl tetra::State for State {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        let mouse_position = input::get_mouse_position(ctx);

        const BOARD: Rectangle = Rectangle::new(
            WINDOW_MARGIN,
            WINDOW_MARGIN,
            WINDOW_SIZE - WINDOW_MARGIN,
            WINDOW_SIZE - WINDOW_MARGIN,
        );
        if BOARD.contains_point(mouse_position) {
            let position = mouse_position - Vec2::new(WINDOW_MARGIN, WINDOW_MARGIN);
            let row = ((position.x / CELL_SIZE).floor() as usize).min(N - 1);
            let col = ((position.y / CELL_SIZE).floor() as usize).min(N - 1);

            if let Some(mut action) = self.action.take() {
                self.action = Some(if (action.row, action.col) != (row, col) {
                    if action.color != self.cells[row][col].color {
                        Action::new(&self.cells, row, col, self.cells[row][col].color)
                    } else {
                        action.row = row;
                        action.col = col;
                        action
                    }
                } else if input::is_mouse_button_pressed(ctx, MouseButton::Left) {
                    action.apply(self);
                    self.num_changed += 1;
                    eprintln!("{}", self.num_changed);
                    Action::new(&self.cells, row, col, self.cells[row][col].color)
                } else if input::is_key_pressed(ctx, Key::Left) {
                    action.color = (action.color + 9 - 1) % 9;
                    action
                } else if input::is_key_pressed(ctx, Key::Right) {
                    action.color = (action.color + 1) % 9;
                    action
                } else {
                    action
                });
            } else {
                self.action = Some(Action::new(
                    &self.cells,
                    row,
                    col,
                    self.cells[row][col].color,
                ));
            }
        }

        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::clear(ctx, Color::BLACK);

        const COLOR: [Color; 9] = [
            Color::rgb(1.0, 0.0, 0.0),
            Color::rgb(0.0, 1.0, 0.0),
            Color::rgb(1.0, 1.0, 0.0),
            Color::rgb(0.0, 0.0, 1.0),
            Color::rgb(1.0, 0.0, 1.0),
            Color::rgb(0.0, 1.0, 1.0),
            Color::rgb(1.0, 0.5, 0.0),
            Color::rgb(0.0, 1.0, 0.5),
            Color::rgb(0.5, 0.0, 1.0),
        ];

        for Cell { color, rectangle } in self.cells.iter().flatten() {
            Mesh::rectangle(ctx, ShapeStyle::Fill, *rectangle)?
                .draw(ctx, DrawParams::new().color(COLOR[*color]));
        }

        if let Some(action) = self.action.as_ref() {
            for &(row, col) in &action.effected {
                Mesh::rectangle(ctx, ShapeStyle::Stroke(4.0), self.cells[row][col].rectangle)?
                    .draw(ctx, DrawParams::new().color(Color::BLACK));
                Mesh::rectangle(ctx, ShapeStyle::Stroke(2.0), self.cells[row][col].rectangle)?
                    .draw(ctx, DrawParams::new().color(Color::WHITE));
            }
            for &(row, col) in &action.effected {
                Mesh::rectangle(ctx, ShapeStyle::Fill, self.cells[row][col].rectangle)?
                    .draw(ctx, DrawParams::new().color(COLOR[action.color]));
            }
            let selected = self.cells[action.row][action.col].rectangle;
            Mesh::rectangle(ctx, ShapeStyle::Stroke(4.0), selected)?
                .draw(ctx, DrawParams::new().color(Color::BLACK));
            Mesh::rectangle(ctx, ShapeStyle::Stroke(2.0), selected)?
                .draw(ctx, DrawParams::new().color(Color::WHITE));
        }

        Ok(())
    }
}

struct Cell {
    color: usize,
    rectangle: Rectangle,
}

struct Action {
    row: usize,
    col: usize,
    color: usize,
    effected: Vec<(usize, usize)>,
}

impl Action {
    fn new(cells: &[Vec<Cell>], row: usize, col: usize, color: usize) -> Action {
        const D: [(usize, usize); 4] = [
            (0, 1),
            (1, 0),
            (0, 1usize.wrapping_neg()),
            (1usize.wrapping_neg(), 0),
        ];
        let mut change = vec![vec![false; N]; N];
        let mut stack = vec![(row, col)];
        while let Some((row, col)) = stack.pop() {
            change[row][col] = true;
            let adjacent4 = D
                .iter()
                .map(|&(dr, dc)| (row.wrapping_add(dr), col.wrapping_add(dc)))
                .filter(|&(r, c)| r < N && c < N)
                .filter(|&(r, c)| cells[r][c].color == cells[row][col].color)
                .filter(|&(r, c)| !change[r][c]);
            for (row, col) in adjacent4 {
                stack.push((row, col));
            }
        }
        let effected = change
            .iter()
            .enumerate()
            .flat_map(|(row, change)| {
                change
                    .iter()
                    .enumerate()
                    .filter(|(_, change)| **change)
                    .map(move |(col, _)| (row, col))
            })
            .collect();
        Self {
            row,
            col,
            color,
            effected,
        }
    }
    /// なにか変わったら true
    fn apply(&self, state: &mut State) -> bool {
        if self.color == state.cells[self.row][self.col].color {
            return false;
        }
        for &(row, col) in &self.effected {
            state.cells[row][col].color = self.color;
        }
        true
    }
}
