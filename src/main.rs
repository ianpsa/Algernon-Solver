mod utils;

use std::error::Error;
use r2r::Timer;
use tokio::time::{Duration, sleep, timeout};
use log::{info, error, trace};
use utils::algorithms::pathfinding::bfs_pathfind;
use utils::map::Map;
use utils::ros::ros_interface::RosInterface;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let ctx = r2r::Context::create()?;
    let ros = RosInterface::new(&ctx)?;
    ros.wait_for_services().await?;
    
    let map_name = ros.reset_game(true).await?;
    sleep(Duration::from_secs(1)).await;
    
    info!("{map_name}");
    let full_grid = ros.get_full_map().await?;
    let mut map = Map::from_grid(full_grid);

    
    loop {
        let Some(path) = (bfs_pathfind(&map)) else {
            error!("Path não existe ou target não existe.");
            break;
        };



        trace!("Caminho encontrado com {} passos.", path.len() - 1);

        let mut sem_falhas = true;

        for window in path.windows(2) {
            let from = window[0];
            let to = window[1];
            let dir = go_to_dir(from, to).expect("passo não ortogonal");

            trace!("Movendo de {:?} para {:?} ({dir})", from, to);

            let moved = ros.move_robot(dir).await?;
            if !moved {
                eprintln!("Movimento {dir} falhou; recalculando caminho com mapa atualizado.");
                let full_grid = ros.get_full_map().await?;
                map = Map::from_grid(full_grid);
                sem_falhas = false;
                break;
            }


            let full_grid = ros.get_full_map().await?;
            map = Map::from_grid(full_grid);

            if let (Some(robot), Some(target)) = (map.robot_position(), map.target_position()) {
                if robot == target {

                    info!("Algernon ao alvo!");
                    return Ok(());
                }
            }
        }

        if sem_falhas {
            // Loop volta para confirmar se ainda existe caminho (caso o alvo tenha mudado)
            // ou se já estamos sobre o alvo.
            continue;
        }
    }

    Ok(())
}

/// Converte dois pontos adjacentes do caminho em uma direção ROS.
fn go_to_dir(
    from: (usize, usize),
    to: (usize, usize),
) -> Option<&'static str> {
    let row_delta = to.0 as isize - from.0 as isize;
    let col_delta = to.1 as isize - from.1 as isize;

    match (row_delta, col_delta) {
        (0, 1) => Some("right"),
        (0, -1) => Some("left"),
        (1, 0) => Some("down"),
        (-1, 0) => Some("up"),
        _ => None,
    }
}
