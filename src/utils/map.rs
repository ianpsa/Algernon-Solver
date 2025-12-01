#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CellState {
    Unknown,
    Wall,
    Target,
    Free,
    Robot,
}

pub struct Map {
    grid: Vec<Vec<CellState>>,
    rows: usize,
    cols: usize,
    robot_pos: Option<(usize, usize)>,
    target_pos: Option<(usize, usize)>,
}

impl Map {
    fn is_it_inside(&self, row: i32, col: i32) -> bool {
        row >= 0
            && col >= 0
            && (row as usize) < self.rows
            && (col as usize) < self.cols
    }

    pub fn get_neighbors(&self, pos: (usize, usize)) -> Vec<(usize, usize)> {
        let (x, y) = pos;
        let mut neighbors = Vec::new();

        if x + 1 < self.rows {
            neighbors.push((x + 1, y));
        }
        if y + 1 < self.cols {
            neighbors.push((x, y + 1));
        }
        if x > 0 {
            neighbors.push((x - 1, y));
        }
        if y > 0 {
            neighbors.push((x, y - 1));
        }

        neighbors
    }

    pub fn char_to_cellstate(c: char) -> CellState {
        match c {
            'b' => CellState::Wall,
            't' => CellState::Target,
            'f' => CellState::Free,
            'r' => CellState::Robot,
            _ => CellState::Unknown,
        }
    }

    pub fn from_grid(grid_chars: Vec<Vec<char>>) -> Self {
        let rows = grid_chars.len();
        let cols = grid_chars.first().map(|row| row.len()).unwrap_or(0);
        let mut grid = vec![vec![CellState::Unknown; cols]; rows];
        let mut robot_pos = None;
        let mut target_pos = None;

        for row in 0..rows {
            for col in 0..cols {
                let state = grid_chars
                    .get(row)
                    .and_then(|r| r.get(col))
                    .copied()
                    .map(Self::char_to_cellstate)
                    .unwrap_or(CellState::Unknown);

                match state {
                    CellState::Robot => robot_pos = Some((row, col)),
                    CellState::Target => target_pos = Some((row, col)),
                    _ => {}
                }

                grid[row][col] = state;
            }
        }

        Self {
            grid,
            rows,
            cols,
            robot_pos,
            target_pos,
        }
    }

    pub fn get_cell(&self, pos: (usize, usize)) -> CellState {
        self.grid[pos.0][pos.1]
    }

    pub fn set_cell(&mut self, pos: (usize, usize), state: CellState) {
        self.grid[pos.0][pos.1] = state;

        match state {
            CellState::Robot => self.robot_pos = Some(pos),
            CellState::Target => self.target_pos = Some(pos),
            _ => {
                if self.robot_pos == Some(pos) {
                    self.robot_pos = None;
                }
                if self.target_pos == Some(pos) {
                    self.target_pos = None;
                }
            }
        }
    }

    pub fn robot_position(&self) -> Option<(usize, usize)> {
        self.robot_pos
    }

    pub fn target_position(&self) -> Option<(usize, usize)> {
        self.target_pos
    }

    pub fn new() -> Self {
        Self {
            grid: Vec::new(),
            rows: 0,
            cols: 0,
            robot_pos: None,
            target_pos: None,
        }
    }
}
