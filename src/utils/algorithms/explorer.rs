use crate::utils::map::{Map, CellState};
use crate::utils::algorithms::pathfinding::bfs_pathfind_explorer;
use crate::utils::ros::ros_interface::SensorData;


pub struct Explorer {
    map: Map,
}

impl Explorer {
    pub fn new() -> Self {
        Self {
            map: Map::from_grid(vec![vec!['?'; 29]; 29]),
        }
    }
    
    pub fn update_map_from_sensors(&mut self, sensors: &SensorData, current_pos: (usize, usize)) {
        let (row, col) = current_pos;
        
        // Atualizar célula atual
        self.map.set_cell(current_pos, CellState::Robot);
        
        // up (row - 1)
        if row > 0 {
            self.map.set_cell((row - 1, col), Map::char_to_cellstate(sensors.up));
        }
        
        // down (row + 1)
        if row < 28 {
            self.map.set_cell((row + 1, col), Map::char_to_cellstate(sensors.down));
        }
        
        // left (col - 1)
        if col > 0 {
            self.map.set_cell((row, col - 1), Map::char_to_cellstate(sensors.left));
        }
        
        // right (col + 1)
        if col < 28 {
            self.map.set_cell((row, col + 1), Map::char_to_cellstate(sensors.right));
        }
        
        // up_left (row - 1, col - 1)
        if row > 0 && col > 0 {
            self.map.set_cell((row - 1, col - 1), Map::char_to_cellstate(sensors.up_left));
        }
        
        // up_right (row - 1, col + 1)
        if row > 0 && col < 28 {
            self.map.set_cell((row - 1, col + 1), Map::char_to_cellstate(sensors.up_right));
        }
        
        // down_left (row + 1, col - 1)
        if row < 28 && col > 0 {
            self.map.set_cell((row + 1, col - 1), Map::char_to_cellstate(sensors.down_left));
        }
        
        // down_right (row + 1, col + 1)
        if row < 28 && col < 28 {
            self.map.set_cell((row + 1, col + 1), Map::char_to_cellstate(sensors.down_right));
        }

    }
    
    pub fn is_fully_mapped(&self, _current_pos: (usize, usize)) -> bool {
        for row in 0..29 {
            for col in 0..29 {
                if self.map.get_cell((row, col)) == CellState::Unknown {
                    return false;
                }
            }
        }
        true
    }
    
    pub fn is_area_mapped(&self, pos: (usize, usize)) -> bool {
        // Verifica se todas as células ao redor estão mapeadas (não Unknown)
        for neighbor in self.map.get_neighbors(pos) {
            if self.map.get_cell(neighbor) == CellState::Unknown {
                return false;
            }
        }
        true
    }
    
    pub fn get_map(&self) -> Map {
        // Clonar o mapa interno para uso externo
        Map::from_grid(self.export_map_as_grid())
    }
    
    fn export_map_as_grid(&self) -> Vec<Vec<char>> {
        let mut grid = vec![vec!['?'; 29]; 29];
        for row in 0..29 {
            for col in 0..29 {
                let cell = self.map.get_cell((row, col));
                grid[row][col] = match cell {
                    CellState::Unknown => '?',
                    CellState::Wall => 'b',
                    CellState::Target => 't',
                    CellState::Free => 'f',
                    CellState::Robot => 'r',
                };
            }
        }
        grid
    }
    
    pub fn next_move(&mut self, current_pos: (usize, usize), sensors: &SensorData) -> Option<String> {
        self.update_map_from_sensors(sensors, current_pos);
        
        if let Some(frontier) = self.find_nearest_frontier(current_pos) {
            if let Some(path) = bfs_pathfind_explorer(&self.map, current_pos, frontier) {
                if path.len() > 1 {
                    let next_pos = path[1];
                    return Some(Self::direction_between(current_pos, next_pos));
                }
            }
        }
        
        if sensors.right == 'f' {
            return Some("right".to_string());
        }
        if sensors.up == 'f' {
            return Some("up".to_string());
        }
        if sensors.left == 'f' {
            return Some("left".to_string());
        }
        if sensors.down == 'f' {
            return Some("down".to_string());
        }
        
        None
    }
    
    fn find_nearest_frontier(&self, current_pos: (usize, usize)) -> Option<(usize, usize)> {
        use std::collections::{VecDeque, HashSet};
        
        let mut queue: VecDeque<(usize, usize)> = VecDeque::new();
        let mut visited = HashSet::new();
        
        queue.push_back(current_pos);
        visited.insert(current_pos);
        
        while let Some(pos) = queue.pop_front() {
            // é uma fronteira ?
            if self.is_frontier(pos) {
                return Some(pos);
            }
            
            // explorar vizinhos
            for neighbor in self.map.get_neighbors(pos) {
                if visited.contains(&neighbor) {
                    continue;
                }
                
                if self.map.get_cell(neighbor) == CellState::Wall {
                    continue;
                }
                
                let cell = self.map.get_cell(neighbor);
                // Durante exploração, bloqueia Target (trata como parede)
                if cell != CellState::Free && cell != CellState::Robot {
                    continue;
                }
                
                visited.insert(neighbor);
                queue.push_back(neighbor);
            }
        }
        
        None
    }
    
    // Função auxiliar para verificar se uma célula é fronteira
    fn is_frontier(&self, pos: (usize, usize)) -> bool {
        // A célula deve ser Free ou Robot (NOT Target - ignoramos target durante exploração)
        let cell = self.map.get_cell(pos);
        if cell != CellState::Free && cell != CellState::Robot {
            return false;
        }
        
        // Verificar se tem pelo menos um vizinho Unknown
        for neighbor in self.map.get_neighbors(pos) {
            if self.map.get_cell(neighbor) == CellState::Unknown {
                return true;
            }
        }
        
        false
    }

    fn direction_between(from: (usize, usize), to: (usize, usize)) -> String {
        let (from_row, from_col) = from;
        let (to_row, to_col) = to;
        
        if to_row < from_row {
            "up".to_string()
        } else if to_row > from_row {
            "down".to_string()
        } else if to_col < from_col {
            "left".to_string()
        } else {
            "right".to_string()
        }
    }
    
}
