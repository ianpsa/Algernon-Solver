use futures::StreamExt;
use r2r::cg_interfaces::{msg, srv};
use r2r::{Client, Node};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;
use tokio::task::JoinHandle;

#[derive(Debug, Clone)]
pub struct SensorData {
    pub up: char,
    pub down: char,
    pub left: char,
    pub right: char,
    pub up_left: char,
    pub up_right: char,
    pub down_left: char,
    pub down_right: char,
}

#[derive(Debug, Clone)]
pub struct SensorReading {
    pub sensors: SensorData,
}

pub struct RosInterface {
    spin_flag: Arc<AtomicBool>,
    spin_thread: Option<thread::JoinHandle<()>>,
    sensor_history: Arc<Mutex<Vec<SensorReading>>>,
    current_position: Arc<Mutex<(usize, usize)>>,
    sensor_task: Option<JoinHandle<()>>,
    move_client: Client<srv::MoveCmd::Service>,
    get_map_client: Client<srv::GetMap::Service>,
    reset_client: Client<srv::Reset::Service>,
}

fn string_to_char(s: &str) -> char {
    s.chars().next().unwrap_or('?')
}

impl RosInterface {
    pub fn new(context: &r2r::Context) -> Result<Self, Box<dyn std::error::Error>> {
        let mut node = r2r::Node::create(context.clone(), "algernon_solver", "")?;

        let move_client = node.create_client::<srv::MoveCmd::Service>(
            "/move_command",
            r2r::QosProfile::services_default(),
        )?;
        let get_map_client = node.create_client::<srv::GetMap::Service>(
            "/get_map",
            r2r::QosProfile::services_default(),
        )?;
        let reset_client = node.create_client::<srv::Reset::Service>(
            "/reset",
            r2r::QosProfile::services_default(),
        )?;

        let sensor_history = Arc::new(Mutex::new(Vec::new()));
        let current_position = Arc::new(Mutex::new((0, 0)));

        let subscriber = node.subscribe::<msg::RobotSensors>(
            "/culling_games/robot_sensors",
            r2r::QosProfile::sensor_data(),
        )?;

        let sensor_history_clone = sensor_history.clone();
        let sensor_task = tokio::spawn(async move {
            let mut stream = subscriber;
            while let Some(msg) = stream.next().await {
                let sensors = SensorData {
                    up: string_to_char(&msg.up),
                    down: string_to_char(&msg.down),
                    left: string_to_char(&msg.left),
                    right: string_to_char(&msg.right),
                    up_left: string_to_char(&msg.up_left),
                    up_right: string_to_char(&msg.up_right),
                    down_left: string_to_char(&msg.down_left),
                    down_right: string_to_char(&msg.down_right),
                };



                let reading = SensorReading { sensors };
                let mut history = sensor_history_clone.lock().unwrap();
                history.push(reading);
            }
        });

        let node_arc = Arc::new(Mutex::new(node));
        let spin_flag = Arc::new(AtomicBool::new(true));
        let spin_node = node_arc.clone();
        let spin_flag_clone = spin_flag.clone();
        let spin_thread = thread::spawn(move || {
            while spin_flag_clone.load(Ordering::Relaxed) {
                let Ok(mut node) = spin_node.lock() else {
                    break;
                };
                node.spin_once(Duration::from_millis(50));
            }
        });

        Ok(Self {
            spin_flag,
            spin_thread: Some(spin_thread),
            sensor_history,
            current_position,
            sensor_task: Some(sensor_task),
            move_client,
            get_map_client,
            reset_client,
        })
    }

    pub async fn move_robot(&self, direction: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let request = srv::MoveCmd::Request {
            direction: direction.to_string(),
        };

        let response = self.move_client.request(&request)?.await?;

        {
            let mut pos = self.current_position.lock().unwrap();
            *pos = (
                response.robot_pos[0] as usize,
                response.robot_pos[1] as usize,
            );
        }
        Ok(response.success)
    }

    pub async fn get_full_map(&self) -> Result<Vec<Vec<char>>, Box<dyn std::error::Error>> {
        let request = srv::GetMap::Request {};

        let response = self.get_map_client.request(&request)?.await?;

        let rows = response.occupancy_grid_shape[0] as usize;
        let cols = response.occupancy_grid_shape[1] as usize;

        let grid = response
            .occupancy_grid_flattened
            .chunks(cols)
            .take(rows)
            .map(|row_slice| {
                row_slice
                    .iter()
                    .map(|s| string_to_char(s))
                    .collect::<Vec<char>>()
            })
            .collect();

        Ok(grid)
    }

    pub async fn reset_game(&self, is_random: bool) -> Result<String, Box<dyn std::error::Error>> {
        let request = srv::Reset::Request {
            is_random,
            map_name: String::new(),
        };

        let response = self.reset_client.request(&request)?.await?;

        Ok(response.loaded_map_name)
    }

    pub fn get_sensor_history(&self) -> Vec<SensorReading> {
        let history = self.sensor_history.lock().unwrap();
        history.clone()
    }

    pub fn get_current_position(&self) -> (usize, usize) {
        let pos = self.current_position.lock().unwrap();
        *pos
    }

    pub async fn wait_for_services(&self) -> Result<(), Box<dyn std::error::Error>> {
        Node::is_available(&self.move_client)?.await?;
        Node::is_available(&self.get_map_client)?.await?;
        Node::is_available(&self.reset_client)?.await?;
        Ok(())
    }
}

impl Drop for RosInterface {
    fn drop(&mut self) {
        if let Some(task) = self.sensor_task.take() {
            task.abort();
        }
        self.spin_flag.store(false, Ordering::SeqCst);
        if let Some(handle) = self.spin_thread.take() {
            let _ = handle.join();
        }
    }
}
