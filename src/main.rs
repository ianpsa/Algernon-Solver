mod utils;

use std::error::Error;
use clap::{Parser, ValueEnum};
use tokio::time::{Duration, sleep};
use log::{info, error, trace};
use utils::algorithms::pathfinding::bfs_pathfind;
use utils::algorithms::explorer::Explorer;
use utils::map::Map;
use utils::ros::ros_interface::RosInterface;

#[derive(Debug, Clone, ValueEnum)]
enum Mode {
    /// Usa mapa completo para encontrar o caminho mais curto (BFS)
    Pathfinding,
    /// Explora o mapa usando apenas sensores e monta o mapa (wall following + BFS)
    Explorer,
}

#[derive(Parser, Debug)]
#[command(name = "algernon-solver")]
struct Args {
    // modo
    #[arg(value_enum)]
    mode: Mode,

}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let args = Args::parse();
    
    info!("Iniciando em modo: {:?}", args.mode);

    let ctx = r2r::Context::create()?;
    let ros = RosInterface::new(&ctx)?;
    ros.wait_for_services().await?;
    
    ros.reset_game(true).await?;
    sleep(Duration::from_secs(1)).await;

    match args.mode {
        Mode::Pathfinding => run_pathfinding_mode(ros).await,
        Mode::Explorer => run_explorer_mode(ros).await,
    }
}

async fn run_pathfinding_mode(ros: RosInterface) -> Result<(), Box<dyn Error>> {
    let full_grid = ros.get_full_map().await?;
    let mut map = Map::from_grid(full_grid);
    
    loop {
        if map.robot_position().is_none() || map.target_position().is_none() {
            error!("Algernon ou target não encontrado no mapa.");
            break;
        }

        let Some(path) = bfs_pathfind(&map) else {
            error!("Path não existe.");
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
                    info!("Algernon chegou ao alvo!");
                    return Ok(());
                }
            }
        }

        if sem_falhas {
            continue;
        }
    }

    Ok(())
}

async fn run_explorer_mode(ros: RosInterface) -> Result<(), Box<dyn Error>> {
    let mut explorer = Explorer::new();
    
    // Esperar sensores inicializarem
    sleep(Duration::from_millis(500)).await;
    
    loop {
        sleep(Duration::from_millis(50)).await;
        
        // posição atual
        let current_pos = ros.get_current_position();
        
        let sensor_readings = ros.get_sensor_history();
        let latest_sensor = match sensor_readings.last() {
            Some(reading) => &reading.sensors,
            None => {
                error!("Nenhum dado de sensor disponível, aguardando...");
                sleep(Duration::from_millis(200)).await;
                continue;
            }
        };

        // próximo movimento
        let next_move = match explorer.next_move(current_pos, latest_sensor) {
            Some(dir) => dir,
            None => {
                error!("Algernon tá preso durante exploração!");
                break;
            }
        };

        info!("Movimento: {} (posição atual: {:?})", next_move, current_pos);

        // mover
        let _moved = ros.move_robot(&next_move).await?;

        // Verificar se mapa está completamente explorado
        if explorer.is_fully_mapped(current_pos) {
            info!("Mapa explorado!");
            break;
        }
        
        if !explorer.is_area_mapped(current_pos) {
            sleep(Duration::from_millis(100)).await;
        }
    }    
    
    sleep(Duration::from_secs(1)).await;
        
    let mut map;
    
    loop {
        // Atualizar posição do Algernon no mapa
        let full_grid = ros.get_full_map().await?;
        map = Map::from_grid(full_grid);
        
        if map.robot_position().is_none() || map.target_position().is_none() {
            error!("Algernon ou target não encontrado no mapa.");
            break;
        }

        let Some(path) = bfs_pathfind(&map) else {
            error!("Path não existe.");
            break;
        };

        let mut sem_falhas = true;

        for window in path.windows(2) {
            let from = window[0];
            let to = window[1];
            let dir = go_to_dir(from, to).expect("passo não ortogonal");
            
            info!("Movendo de {:?} para {:?} ({dir})", from, to);

            let moved = ros.move_robot(dir).await?;
            if !moved {
                error!("Movimento {dir} falhou; recalculando caminho.");
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
