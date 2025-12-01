use crate::utils::map::{CellState, Map};
use std::collections::{HashMap, HashSet, VecDeque};

/// Pathfinding para modo normal - só usa células conhecidas (Free, Robot, Target)
pub fn bfs_pathfind(map: &Map) -> Option<Vec<(usize, usize)>> {
    let start = map.robot_position()?;
    let target = map.target_position()?;
    bfs_pathfind_internal(map, start, target, false)
}

/// Pathfinding para modo explorer - permite células Unknown
pub fn bfs_pathfind_explorer(map: &Map, start: (usize, usize), target: (usize, usize)) -> Option<Vec<(usize, usize)>> {
    bfs_pathfind_internal(map, start, target, true)
}

fn bfs_pathfind_internal(map: &Map, start: (usize, usize), target: (usize, usize), allow_unknown: bool) -> Option<Vec<(usize, usize)>> {
    let mut queue: VecDeque<(usize, usize)> = VecDeque::new();
    let mut visited: HashSet<(usize, usize)> = HashSet::new();
    let mut parent: HashMap<(usize, usize), (usize, usize)> = HashMap::new();

    queue.push_back(start);
    visited.insert(start);

    while let Some(current) = queue.pop_front() {
        if current == target {
            return Some(reconstruct_path(start, target, &parent));
        }

        // Explorar vizinhos
        for neighbor in map.get_neighbors(current) {
            if visited.contains(&neighbor) {
                continue;
            }

            let cell = map.get_cell(neighbor);
            
            let is_navigable = if allow_unknown {
                cell != CellState::Wall && cell != CellState::Target
            } else {
                cell == CellState::Free || cell == CellState::Robot || cell == CellState::Target
            };

            if !is_navigable {
                continue;
            }

            visited.insert(neighbor);
            parent.insert(neighbor, current);
            queue.push_back(neighbor);
        }
    }

    None
}

fn reconstruct_path(
    start: (usize, usize),
    target: (usize, usize),
    parent: &HashMap<(usize, usize), (usize, usize)>,
) -> Vec<(usize, usize)> {
    let mut path = Vec::new();
    let mut curr = target;

    while curr != start {
        path.push(curr);
        curr = parent[&curr];
    }

    path.push(start);
    path.reverse();
    path
}
