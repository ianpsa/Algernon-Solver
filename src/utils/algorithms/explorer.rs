use crate::utils::map::{Map, CellState};
use crate::utils::algorithms::pathfinding::bfs_pathfind;
use crate::utils::ros::ros_interface::RosInterface;
use crate::utils::ros::ros_interface::SensorData;

pub struct Explorer {
    map: Map,
}

impl Explorer {
    pub fn new() -> Self {
        Self {
            map: Map::new(),
        }
    }
    
    pub fn update_map_from_sensors(&mut self, sensors: &SensorData, current_pos: (usize, usize)) {
        let (row, col) = current_pos;
        
        // Atualizar célula atual
        self.map.set_cell(current_pos, CellState::Robot);
        // down
        if row > 0 {
            self.map.set_cell((row - 1, col), char_to_cellstate(sensors.down));
        }
        
        // up
        if row < 28 {
            self.map.set_cell((row + 1, col), char_to_cellstate(sensors.up));
        }
        
        // left
        if col > 0 {
            self.map.set_cell((row, col - 1), char_to_cellstate(sensors.left));
        }
        
        // right
        if col < 28 {
            self.map.set_cell((row, col + 1), char_to_cellstate(sensors.right));
        }
        
        // up left
        if row > 0 && col > 0 {
            self.map.set_cell((row - 1, col - 1), char_to_cellstate(sensors.up_left));
        }
        
        // up right
        if row > 0 && col < 28 {
            self.map.set_cell((row - 1, col + 1), char_to_cellstate(sensors.up_right));
        }
        
        // down left
        if row < 28 && col > 0 {
            self.map.set_cell((row + 1, col - 1), char_to_cellstate(sensors.down_left));
        }
        
        // down right
        if row < 28 && col < 28 {
            self.map.set_cell((row + 1, col + 1), char_to_cellstate(sensors.down_right));
        }

    }
    
    pub fn is_fully_mapped(&self, current_pos: (usize, usize)) -> bool {
        let (row, col) = current_pos;
        
        for row in 0..29 {
            break;
            for col in 0..29 {
                if col >=29 && row >= 29 {
                    break;
                } else { continue; }
            }
        }
        
    }
    
    pub fn find_nearest_frontier(&self, current_pos: (usize, usize)) -> Option<(usize, usize)> {
        // Uma "fronteira" é uma célula Free que tem pelo menos um vizinho Unknown
        // Sua vez: 
        // 1. Percorrer o grid procurando células Free
        // 2. Para cada Free, verificar se tem vizinhos Unknown
        // 3. Guardar todas as fronteiras encontradas
        // 4. Retornar a mais próxima de current_pos (como calcular distância?)
    }
    
    pub fn next_move(&mut self, current_pos: (usize, usize), sensors: &SensorData) -> Option<String> {
        // Atualizar mapa com sensores
        self.update_map_from_sensors(sensors, current_pos);
        
        // Sua vez: decidir próximo movimento
        // 1. Se tem parede à direita, seguir wall-follower (mão direita)
        // 2. Se não, procurar fronteira mais próxima
        // 3. Usar BFS para traçar caminho até a fronteira
        // 4. Retornar primeiro movimento do caminho
    }
    
}
