// src/ooda/mod.rs

use crate::comms::CommunicationHub;
use crate::flight_control::FlightController;
use crate::models::markov_chain::MarkovEnvironmentModel;
use crate::payload::PayloadManager;
use crate::physical::{ComponentId, PhysicalTopology};
use crate::sensor_fusion::{SensorData, SensorFusion, Situation, ThreatLevel};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq)]
pub enum Decision {
    ChangeAltitude(f32),
    SwitchPayloadMode,
    EnhanceCommsReliability,
    PrepareMeshNetworking,
}

impl Decision {
    pub fn matches(&self, situation: &Situation) -> bool {
        match (self, &situation.threat_level) {
            (Decision::ChangeAltitude(_), ThreatLevel::High) => true,
            (Decision::SwitchPayloadMode, ThreatLevel::Medium)
            | (Decision::SwitchPayloadMode, ThreatLevel::Low) => true,
            (Decision::EnhanceCommsReliability, _) => true,
            (Decision::PrepareMeshNetworking, _) => true,
            _ => false,
        }
    }
}
pub struct DecisionMaker;

impl DecisionMaker {
    pub fn make(situation: &Situation, _payload: &PayloadManager) -> Decision {
        match situation.threat_level {
            ThreatLevel::High => Decision::ChangeAltitude(100.0),
            _ => Decision::SwitchPayloadMode,
        }
    }
}

pub struct OodaLoop {
    pub last_cycle_time: Duration,
    pub sensor_fusion: SensorFusion,
    pub decision_cache: Option<Decision>,
    pub physical_layout: Option<PhysicalTopology>,
    pub environment_model: MarkovEnvironmentModel,
}

impl OodaLoop {
    pub fn new() -> Self {
        // Define environment states and initial transition matrix
        let env_states = vec![
            "clear".to_string(),
            "light_rain".to_string(),
            "heavy_rain".to_string(),
            "fog".to_string(),
            "urban_canyon".to_string(),
            "forest".to_string(),
            "mountainous".to_string(),
        ];

        // Initial transition matrix (could be learned from data)
        // Each row represents transition probabilities from a state to all others
        let transition_matrix = vec![
            vec![0.7, 0.1, 0.05, 0.05, 0.05, 0.03, 0.02],  // clear
            vec![0.2, 0.5, 0.2, 0.05, 0.02, 0.02, 0.01],   // light_rain
            vec![0.1, 0.3, 0.4, 0.1, 0.05, 0.03, 0.02],    // heavy_rain
            vec![0.1, 0.1, 0.1, 0.5, 0.1, 0.05, 0.05],     // fog
            vec![0.05, 0.05, 0.05, 0.05, 0.7, 0.05, 0.05], // urban_canyon
            vec![0.05, 0.05, 0.05, 0.05, 0.05, 0.7, 0.05], // forest
            vec![0.05, 0.05, 0.05, 0.05, 0.05, 0.05, 0.7], // mountainous
        ];

        Self {
            last_cycle_time: Duration::ZERO,
            sensor_fusion: SensorFusion::default(),
            decision_cache: None,
            physical_layout: None,
            environment_model: MarkovEnvironmentModel::new(&env_states, transition_matrix, 0),
        }
    }

    pub fn with_physical_layout(layout: PhysicalTopology) -> Self {
        let mut ooda = Self::new();
        ooda.physical_layout = Some(layout);
        ooda
    }

    // Full OODA cycle execution with Markov prediction
    pub fn execute_cycle(
        &mut self,
        comms: &mut CommunicationHub,
        payload: &mut PayloadManager,
        flight_controller: &mut FlightController,
    ) -> Duration {
        let start_time = Instant::now();

        // Set physical layout for comms if available
        if let Some(layout) = &self.physical_layout {
            if comms.physical_topology.is_none() {
                comms.set_physical_topology(layout.clone());
            }
        }

        // OBSERVE - now with physical concerns
        let sensor_data = self.observe(comms, payload);

        // Calculate observation latency based on physical layout
        let observation_latency = if let Some(layout) = &self.physical_layout {
            // Simulate physical latency gathering sensor data
            let mut latency_ns = 0.0;

            // Add latency for each sensor connection
            if let Ok(imu_latency) = layout.get_path_latency(&[
                ComponentId::Imu,
                ComponentId::SensorHub,
                ComponentId::MainProcessor,
            ]) {
                latency_ns += imu_latency;
            }

            if sensor_data.gps.is_some() {
                if let Ok(gps_latency) = layout.get_path_latency(&[
                    ComponentId::Gps,
                    ComponentId::SensorHub,
                    ComponentId::MainProcessor,
                ]) {
                    latency_ns += gps_latency;
                }
            }

            // Convert to Duration
            Duration::from_nanos(latency_ns as u64)
        } else {
            Duration::from_micros(50) // Default latency
        };

        // Add artificial delay to simulate physical sensor data gathering
        std::thread::sleep(observation_latency);

        // ORIENT with environment classification
        let situation = self.orient(&sensor_data);

        // Update Markov model with current environment state
        let current_env_state = self.classify_environment(&situation, &sensor_data);
        self.environment_model.update_state(Some(current_env_state));

        // Predict next likely environment state
        // let predicted_state_idx = self.environment_model.predict_next_state();
        // let predicted_env = self.environment_model.get_state_name(predicted_state_idx);
        let predicted_state_idx = self.environment_model.predict_next_state();
        let predicted_env = self
            .environment_model
            .get_state_name(predicted_state_idx)
            .to_string();

        // DECIDE with prediction
        // let decision = self.decide_with_prediction(&situation, payload, predicted_env);
        let decision = self.decide_with_prediction(&situation, payload, &predicted_env);

        // ACT - now with physical concerns
        let action_latency = self.act(decision, flight_controller, payload, comms);

        // Add artificial delay to simulate physical actuation delay
        std::thread::sleep(action_latency);

        self.last_cycle_time = start_time.elapsed();
        self.last_cycle_time
    }

    fn observe(&self, comms: &CommunicationHub, payload: &PayloadManager) -> SensorData {
        // Create dummy IMU reading since we don't have actual sensors
        let imu = crate::sensor_fusion::IMUReading {
            accel: nalgebra::Vector3::new(0.0, 0.0, 9.81), // Standard gravity
            gyro: nalgebra::Vector3::new(0.0, 0.0, 0.0),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        SensorData {
            imu,
            gps: None,
            lidar: None,
            radar_contacts: comms.radar_contacts.clone(),
            operator_messages: comms
                .operators
                .iter()
                .filter(|o| match o.last_heartbeat {
                    Some(time) => time.elapsed() < Duration::from_secs(5),
                    None => false,
                })
                .count(),
            payload_status: payload.get_status(),
        }
    }

    fn orient(&mut self, data: &SensorData) -> Situation {
        // Use sensor fusion to analyze the data
        self.sensor_fusion.analyze(data)
    }

    // Classify environment based on sensor data and situation
    pub fn classify_environment(&self, situation: &Situation, data: &SensorData) -> usize {
        // Make sure we're using indices that match our state array in OodaLoop::new
        // We expect states to be:
        // 0: "clear"
        // 1: "light_rain"
        // 2: "heavy_rain"
        // 3: "fog"
        // 4: "urban_canyon"
        // 5: "forest"
        // 6: "mountainous"

        // First check the number of states in our model - this is a safety check
        let num_states = self.environment_model.states.len();

        // Check for urban canyon environment (high number of obstacles, GPS issues)
        if data.radar_contacts.len() > 3 && data.gps.is_none() {
            // Urban canyon should be index 4, but make sure it's valid
            return std::cmp::min(4, num_states - 1);
        }

        // Check for forest (specific radar pattern, partial GPS)
        if data.radar_contacts.len() > 1 && data.radar_contacts.len() <= 3 && data.gps.is_some() {
            // Forest should be index 5, but make sure it's valid
            return std::cmp::min(5, num_states - 1);
        }

        // Check for mountainous terrain (altitude changes, specific sensor patterns)
        if data
            .radar_contacts
            .iter()
            .any(|contact| contact.relative_speed_mps > 30.0)
        {
            // Mountainous should be index 6, but make sure it's valid
            return std::cmp::min(6, num_states - 1);
        }

        // Default classification based on threat level for weather conditions
        match situation.threat_level {
            ThreatLevel::High => {
                // heavy_rain should be index 2
                std::cmp::min(2, num_states - 1)
            }
            ThreatLevel::Medium => {
                if data.radar_contacts.is_empty() {
                    // fog should be index 3
                    std::cmp::min(3, num_states - 1)
                } else {
                    // light_rain should be index 1
                    std::cmp::min(1, num_states - 1)
                }
            }
            ThreatLevel::Low => {
                // clear should be index 0
                0 // This is always safe
            }
        }
    }

    pub fn decide_with_prediction(
        &mut self, 
        situation: &Situation, 
        payload: &PayloadManager,
        predicted_env: &str
    ) -> Decision {
        // Check cached decisions for current state
        if let Some(cached) = &self.decision_cache {
            if cached.matches(situation) {
                return cached.clone();
            }
        }
        
        // Get baseline decision for current situation
        let current_decision = DecisionMaker::make(situation, payload);
        
        // Analyze predicted environment state to possibly modify decision
        let adjusted_decision = match predicted_env {
            "heavy_rain" | "fog" => {
                // If bad weather is coming, prepare more robust communication
                Decision::EnhanceCommsReliability
            },
            "urban_canyon" => {
                // If entering urban area, prepare NLOS communications
                Decision::PrepareMeshNetworking
            },
            _ => current_decision.clone() // No adjustment needed
        };
        
        // If prediction suggests a different decision than current,
        // evaluate which is more important
        if adjusted_decision != current_decision {
            // Logic to determine if prediction should override current needs
            if situation.threat_level == ThreatLevel::Low {
                // Current situation not critical, prepare for predicted change
                self.decision_cache = Some(adjusted_decision.clone());
                return adjusted_decision;
            }
        }
        
        // Cache and return decision
        self.decision_cache = Some(current_decision.clone());
        current_decision
    }

    
    
    // Modified to support additional decision types and take comms as parameter
    fn act(
        &self,
        decision: Decision,
        fc: &mut FlightController,
        payload: &mut PayloadManager,
        comms: &mut CommunicationHub,
    ) -> Duration {
        match decision {
            Decision::ChangeAltitude(delta) => {
                fc.adjust_altitude(delta);

                // Calculate physical actuation latency
                if let Some(layout) = &self.physical_layout {
                    // Simulate time for command to reach motors
                    let mut max_latency: f32 = 0.0;

                    // Check each motor path
                    for i in 0..4 {
                        if let Ok(motor_latency) = layout.get_path_latency(&[
                            ComponentId::FlightController,
                            ComponentId::MotorController(i),
                        ]) {
                            max_latency = max_latency.max(motor_latency);
                        }
                    }

                    // Add mechanical delay (based on delta magnitude)
                    let mechanical_delay = delta.abs() * 10.0; // 10ns per unit of delta

                    Duration::from_nanos((max_latency + mechanical_delay) as u64)
                } else {
                    Duration::from_micros(200) // Default latency
                }
            }
            Decision::SwitchPayloadMode => {
                payload.toggle_operational();

                // Calculate payload actuation latency
                if let Some(layout) = &self.physical_layout {
                    if let Ok(payload_latency) = layout.get_path_latency(&[
                        ComponentId::MainProcessor,
                        ComponentId::SensorHub,
                        ComponentId::Camera, // Assuming camera is the primary payload
                    ]) {
                        Duration::from_nanos(payload_latency as u64)
                    } else {
                        Duration::from_micros(100) // Default latency
                    }
                } else {
                    Duration::from_micros(100) // Default latency
                }
            }
            Decision::EnhanceCommsReliability => {
                // Adjust communication system for reliability over bandwidth
                comms.adjust_links(crate::comms::CommsPriority::Medium); // Medium priority for reliability

                // Calculate communication system adjustment latency
                if let Some(layout) = &self.physical_layout {
                    if let Ok(comm_latency) = layout.get_path_latency(&[
                        ComponentId::MainProcessor,
                        ComponentId::CommunicationHub,
                        ComponentId::RadioLink,
                    ]) {
                        Duration::from_nanos(comm_latency as u64)
                    } else {
                        Duration::from_micros(150)
                    }
                } else {
                    Duration::from_micros(150)
                }
            }
            Decision::PrepareMeshNetworking => {
                // Configure mesh networking mode
                // In a real system, this would involve setting up mesh network parameters
                comms.primary_link.link_type = crate::comms::LinkType::WiFiDirect {
                    bandwidth_mbps: 100,
                    channel: 36,
                };

                // Calculate mesh networking setup latency
                if let Some(layout) = &self.physical_layout {
                    if let Ok(mesh_latency) = layout.get_path_latency(&[
                        ComponentId::MainProcessor,
                        ComponentId::CommunicationHub,
                        ComponentId::RadioLink,
                    ]) {
                        // Mesh networking takes longer to set up
                        Duration::from_nanos((mesh_latency * 2.0) as u64)
                    } else {
                        Duration::from_micros(300)
                    }
                } else {
                    Duration::from_micros(300)
                }
            }
        }
    }
}

// // Helper function for UAV
// impl crate::models::architecture::UavSystems {
//     pub fn scan_surroundings(&mut self) {
//         // Simple mock implementation to add radar contacts
//         self.comms.radar_contacts.push(crate::comms::RadarContact {
//             distance_m: 1000.0,
//             bearing_deg: 45.0,
//             relative_speed_mps: 20.0,
//             via_link: crate::comms::LinkType::MAVLink {
//                 version: 2,
//                 heartbeat_interval_ms: 500,
//             },
//         });
//     }

//     pub fn register_navigation_beacon(&mut self, id: &str, position: (f32, f32)) {
//         // Mock implementation to demonstrate beacon registration
//         self.comms.log_beacon(crate::comms::NavigationBeacon {
//             id: id.to_string(),
//             position,
//             signal_strength: 0.95,
//             link_used: self.comms.primary_link.link_type.clone(),
//         });
//     }
// }
