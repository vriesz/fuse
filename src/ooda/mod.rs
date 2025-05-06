// src/ooda/mod.rs

use crate::comms::CommunicationHub;
use crate::flight_control::FlightController;
use crate::payload::PayloadManager;
use crate::physical::{ComponentId, PhysicalTopology};
use crate::sensor_fusion::{SensorData, SensorFusion, Situation, ThreatLevel};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq)]
pub enum Decision {
    ChangeAltitude(f32),
    SwitchPayloadMode,
    // Add other decisions as needed
}

impl Decision {
    pub fn matches(&self, situation: &Situation) -> bool {
        // Simple implementation
        match (self, &situation.threat_level) {
            (Decision::ChangeAltitude(_), ThreatLevel::High) => true,
            (Decision::SwitchPayloadMode, ThreatLevel::Medium)
            | (Decision::SwitchPayloadMode, ThreatLevel::Low) => true,
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
}

impl OodaLoop {
    pub fn new() -> Self {
        Self {
            last_cycle_time: Duration::ZERO,
            sensor_fusion: SensorFusion::default(),
            decision_cache: None,
            physical_layout: None,
        }
    }

    pub fn with_physical_layout(layout: PhysicalTopology) -> Self {
        Self {
            last_cycle_time: Duration::ZERO,
            sensor_fusion: SensorFusion::default(),
            decision_cache: None,
            physical_layout: Some(layout),
        }
    }

    // Full OODA cycle execution
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

        // ORIENT
        let situation = self.orient(&sensor_data);

        // DECIDE
        let decision = self.decide(&situation, payload);

        // ACT - now with physical concerns
        let action_latency = self.act(decision, flight_controller, payload);

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

    fn decide(&mut self, situation: &Situation, payload: &PayloadManager) -> Decision {
        // Cache decisions for similar situations
        if let Some(cached) = &self.decision_cache {
            if cached.matches(situation) {
                return cached.clone();
            }
        }

        let new_decision = DecisionMaker::make(situation, payload);
        self.decision_cache = Some(new_decision.clone());
        new_decision
    }

    // Modified to return actuation latency
    fn act(
        &self,
        decision: Decision,
        fc: &mut FlightController,
        payload: &mut PayloadManager,
    ) -> Duration {
        match decision {
            Decision::ChangeAltitude(delta) => {
                fc.adjust_altitude(delta);

                // Calculate physical actuation latency
                if let Some(layout) = &self.physical_layout {
                    // Simulate time for command to reach motors
                    let mut max_latency: f32 = 0.0; // Fixed: specify f32 type explicitly

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
        }
    }
}
