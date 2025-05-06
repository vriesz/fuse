// src/physical/topology.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Represents a physical location of a component in 3D space relative to the UAV center
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// Distance in cm from center on X axis (forward/backward)
    pub x: f32,
    /// Distance in cm from center on Y axis (left/right)
    pub y: f32,
    /// Distance in cm from center on Z axis (up/down)
    pub z: f32,
}

impl Position {
    pub fn distance_to(&self, other: &Position) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;

        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

/// Types of physical connections between components
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionType {
    /// Standard copper wire connection
    Copper {
        /// AWG wire gauge (lower = thicker)
        gauge: u8,
        /// Number of wires in the cable
        wires: u8,
        /// Whether the cable is shielded
        shielded: bool,
    },
    /// Fiber optic connection
    FiberOptic {
        /// Single mode or multi-mode
        single_mode: bool,
        /// Bandwidth capacity in Gbps
        bandwidth_gbps: f32,
    },
    /// Direct PCB trace
    PcbTrace {
        /// Width of the trace in mils
        width_mils: u16,
        /// Layers in the PCB the trace crosses
        layers: u8,
    },
    /// Wireless connection
    Wireless {
        /// Radio frequency in MHz
        frequency_mhz: u32,
        /// Transmit power in mW
        power_mw: f32,
    },
}

/// Physical properties of a connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    /// Type of physical connection
    pub connection_type: ConnectionType,
    /// Physical length of the connection in cm
    pub length_cm: f32,
    /// Maximum data rate in Mbps
    pub max_data_rate_mbps: f32,
    /// Latency per cm in nanoseconds
    pub latency_ns_per_cm: f32,
    /// Error rate (bit errors per 10^9 bits)
    pub error_rate: f32,
    /// Power consumption in mW
    pub power_mw: f32,
}

impl Connection {
    pub fn calculate_latency(&self) -> f32 {
        self.length_cm * self.latency_ns_per_cm
    }

    pub fn calculate_reliability(&self) -> f32 {
        // Higher number = more errors
        let bits_per_message = 1024.0 * 8.0; // Example: 1KB message
        let error_probability =
            1.0 - (1.0 - self.error_rate / 1_000_000_000.0).powi(bits_per_message as i32);

        // Return as percentage reliability (100% = perfect)
        (1.0 - error_probability) * 100.0
    }
}

/// Component identity for nodes in the physical mesh
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ComponentId {
    FlightController,
    MainProcessor,
    PowerDistribution,
    Gps,
    Imu,
    Camera,
    Lidar,
    Radar,
    RadioLink,
    MotorController(u8), // Multiple motor controllers with index
    Battery,
    SensorHub,
    CommunicationHub,
}

impl fmt::Display for ComponentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComponentId::FlightController => write!(f, "FlightController"),
            ComponentId::MainProcessor => write!(f, "MainProcessor"),
            ComponentId::PowerDistribution => write!(f, "PowerDistribution"),
            ComponentId::Gps => write!(f, "GPS"),
            ComponentId::Imu => write!(f, "IMU"),
            ComponentId::Camera => write!(f, "Camera"),
            ComponentId::Lidar => write!(f, "Lidar"),
            ComponentId::Radar => write!(f, "Radar"),
            ComponentId::RadioLink => write!(f, "RadioLink"),
            ComponentId::MotorController(idx) => write!(f, "MotorController-{}", idx),
            ComponentId::Battery => write!(f, "Battery"),
            ComponentId::SensorHub => write!(f, "SensorHub"),
            ComponentId::CommunicationHub => write!(f, "CommunicationHub"),
        }
    }
}

/// A physical component in the UAV topology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub id: ComponentId,
    pub position: Position,
    pub weight_g: f32,
    pub power_consumption_mw: f32,
    pub heat_generation_c: f32,
}

/// Represents the complete physical topology of the UAV
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalTopology {
    pub components: HashMap<ComponentId, Component>,
    pub connections: HashMap<(ComponentId, ComponentId), Connection>,
    pub total_weight_g: f32,
    pub total_power_mw: f32,
    pub dimensions_cm: (f32, f32, f32), // Width, length, height
}

impl PhysicalTopology {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            connections: HashMap::new(),
            total_weight_g: 0.0,
            total_power_mw: 0.0,
            dimensions_cm: (0.0, 0.0, 0.0),
        }
    }

    pub fn add_component(&mut self, component: Component) {
        self.total_weight_g += component.weight_g;
        self.total_power_mw += component.power_consumption_mw;

        // Update the UAV dimensions based on component positions
        let pos = &component.position;
        let (width, length, height) = self.dimensions_cm;
        self.dimensions_cm = (
            width.max(pos.y.abs() * 2.0),
            length.max(pos.x.abs() * 2.0),
            height.max(pos.z.abs() * 2.0),
        );

        self.components.insert(component.id.clone(), component);
    }

    pub fn connect(
        &mut self,
        from: &ComponentId,
        to: &ComponentId,
        connection_type: ConnectionType,
    ) -> Result<(), String> {
        // Ensure both components exist
        if !self.components.contains_key(from) {
            return Err(format!("Component {} does not exist", from));
        }
        if !self.components.contains_key(to) {
            return Err(format!("Component {} does not exist", to));
        }

        // Calculate the physical distance between components
        let from_pos = &self.components.get(from).unwrap().position;
        let to_pos = &self.components.get(to).unwrap().position;
        let distance = from_pos.distance_to(to_pos);

        // Create the appropriate connection based on type and distance
        let connection = match connection_type {
            ConnectionType::Copper {
                gauge,
                wires,
                shielded,
            } => {
                // Calculate properties based on copper characteristics
                let resistance_per_cm = match gauge {
                    22 => 0.0005, // Smaller gauge = less resistance
                    24 => 0.0008,
                    26 => 0.0013,
                    _ => 0.001, // Default value
                };

                let error_multiplier = if shielded { 0.2 } else { 1.0 };

                Connection {
                    connection_type: connection_type.clone(),
                    length_cm: distance,
                    max_data_rate_mbps: match gauge {
                        22 => 100.0,
                        24 => 50.0,
                        26 => 25.0,
                        _ => 10.0,
                    } * (wires as f32 / 4.0), // More wires = more bandwidth
                    latency_ns_per_cm: 0.5, // Electrical signal propagation
                    error_rate: 0.1 * distance * resistance_per_cm * error_multiplier,
                    power_mw: distance * resistance_per_cm * 10.0, // Power loss in the wire
                }
            }
            ConnectionType::FiberOptic {
                single_mode,
                bandwidth_gbps,
            } => {
                Connection {
                    connection_type: connection_type.clone(),
                    length_cm: distance,
                    max_data_rate_mbps: bandwidth_gbps * 1000.0,
                    latency_ns_per_cm: 0.33, // Light propagation in fiber
                    error_rate: if single_mode { 0.001 } else { 0.01 } * distance / 100.0,
                    power_mw: 50.0, // Fixed power for transceivers
                }
            }
            ConnectionType::PcbTrace { width_mils, layers } => {
                let cross_layer_penalty = (layers - 1) as f32 * 0.1;

                Connection {
                    connection_type: connection_type.clone(),
                    length_cm: distance,
                    max_data_rate_mbps: (width_mils as f32 / 10.0) * 1000.0,
                    latency_ns_per_cm: 0.3 + cross_layer_penalty,
                    error_rate: 0.001 * distance / 10.0 * cross_layer_penalty,
                    power_mw: distance * 0.1 * (10.0 / width_mils as f32),
                }
            }
            ConnectionType::Wireless {
                frequency_mhz,
                power_mw,
            } => {
                // Higher frequencies attenuate more with distance
                let freq_factor = frequency_mhz as f32 / 1000.0;

                Connection {
                    connection_type: connection_type.clone(),
                    length_cm: distance,
                    max_data_rate_mbps: match frequency_mhz {
                        f if f < 1000 => 10.0,
                        f if f < 2500 => 54.0,
                        f if f < 6000 => 1200.0,
                        _ => 3000.0,
                    },
                    latency_ns_per_cm: 0.33, // Speed of light propagation
                    error_rate: (distance * freq_factor) / power_mw * 10.0,
                    power_mw,
                }
            }
        };

        // Add to total power consumption
        self.total_power_mw += connection.power_mw;

        // Store the connection
        self.connections
            .insert((from.clone(), to.clone()), connection);

        Ok(())
    }

    pub fn get_path_latency(&self, path: &[ComponentId]) -> Result<f32, String> {
        if path.len() < 2 {
            return Err("Path must contain at least two components".to_string());
        }

        let mut total_latency = 0.0;

        for i in 0..path.len() - 1 {
            let from = &path[i];
            let to = &path[i + 1];

            if let Some(connection) = self.connections.get(&(from.clone(), to.clone())) {
                total_latency += connection.calculate_latency();
            } else if let Some(connection) = self.connections.get(&(to.clone(), from.clone())) {
                // Check for reverse connection
                total_latency += connection.calculate_latency();
            } else {
                return Err(format!("No connection between {} and {}", from, to));
            }
        }

        Ok(total_latency)
    }

    pub fn get_path_reliability(&self, path: &[ComponentId]) -> Result<f32, String> {
        if path.len() < 2 {
            return Err("Path must contain at least two components".to_string());
        }

        let mut total_reliability = 100.0; // Start at 100%

        for i in 0..path.len() - 1 {
            let from = &path[i];
            let to = &path[i + 1];

            if let Some(connection) = self.connections.get(&(from.clone(), to.clone())) {
                // Multiply reliability (convert percentage to fraction first)
                total_reliability = total_reliability * connection.calculate_reliability() / 100.0;
            } else if let Some(connection) = self.connections.get(&(to.clone(), from.clone())) {
                // Check for reverse connection
                total_reliability = total_reliability * connection.calculate_reliability() / 100.0;
            } else {
                return Err(format!("No connection between {} and {}", from, to));
            }
        }

        Ok(total_reliability)
    }

    pub fn find_shortest_path(
        &self,
        from: &ComponentId,
        to: &ComponentId,
    ) -> Option<Vec<ComponentId>> {
        // Simple Dijkstra's algorithm implementation
        use std::cmp::Ordering;
        use std::collections::{BinaryHeap, HashSet};

        // Define a custom struct for priority queue
        #[derive(Eq, PartialEq)]
        struct Node {
            id: ComponentId,
            cost: i32,
        }

        impl Ord for Node {
            fn cmp(&self, other: &Self) -> Ordering {
                // Reverse the order for min-heap
                other.cost.cmp(&self.cost)
            }
        }

        impl PartialOrd for Node {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        // Check if source and destination exist
        if !self.components.contains_key(from) || !self.components.contains_key(to) {
            return None;
        }

        // Prepare data structures
        let mut dist: HashMap<ComponentId, f32> = HashMap::new();
        let mut prev: HashMap<ComponentId, ComponentId> = HashMap::new();
        let mut queue = BinaryHeap::new();
        let mut visited = HashSet::new();

        // Initialize distances
        for id in self.components.keys() {
            dist.insert(id.clone(), f32::INFINITY);
        }

        // Start from source
        *dist.get_mut(from).unwrap() = 0.0;
        queue.push(Node {
            id: from.clone(),
            cost: 0,
        });

        // Main loop
        while let Some(Node { id, cost }) = queue.pop() {
            if id == *to {
                // Found destination, reconstruct path
                let mut path = vec![to.clone()];
                let mut current = to;

                while let Some(previous) = prev.get(current) {
                    path.push(previous.clone());
                    current = previous;
                }

                path.reverse();
                return Some(path);
            }

            if visited.contains(&id) {
                continue;
            }

            visited.insert(id.clone());

            // Get all neighbors
            for ((src, dst), connection) in &self.connections {
                let neighbor = if *src == id {
                    dst
                } else if *dst == id {
                    src
                } else {
                    continue;
                };

                // let new_cost = cost + connection.calculate_latency();
                let new_cost_int = (cost + (connection.calculate_latency() as i32)) as i32;

                if new_cost_int < *dist.get(neighbor).unwrap() as i32 {
                    *dist.get_mut(neighbor).unwrap() = new_cost_int as f32;
                    prev.insert(neighbor.clone(), id.clone());
                    let _cost_as_int = (new_cost_int as f32 * 1000.0) as i32;
                    queue.push(Node {
                        id: neighbor.clone(),
                        cost: _cost_as_int,
                    });
                }
            }
        }

        None // No path found
    }
}
