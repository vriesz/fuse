use crate::comms::CommunicationHub;
use crate::payload::PayloadManager;
use crate::flight_control::FlightController;
use crate::sensor_fusion::{SensorData, SensorFusion, Situation, ThreatLevel};
use std::time::{Instant, Duration};

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
            (Decision::SwitchPayloadMode, ThreatLevel::Medium) | 
            (Decision::SwitchPayloadMode, ThreatLevel::Low) => true,
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
    pub decision_cache: Option<Decision>, // Make this field public
}

impl OodaLoop {
    pub fn new() -> Self {
        Self {
            last_cycle_time: Duration::ZERO,
            sensor_fusion: SensorFusion::default(),
            decision_cache: None,
        }
    }

    // Full OODA cycle execution
    pub fn execute_cycle(
        &mut self,
        comms: &mut CommunicationHub,
        payload: &mut PayloadManager,
        flight_controller: &mut FlightController
    ) -> Duration {
        let start_time = Instant::now();
        
        // OBSERVE
        let sensor_data = self.observe(comms, payload);
        
        // ORIENT
        let situation = self.orient(&sensor_data);
        
        // DECIDE
        let decision = self.decide(&situation, payload);
        
        // ACT
        self.act(decision, flight_controller, payload);
        
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
            operator_messages: comms.operators.iter()
                .filter(|o| match o.last_heartbeat {
                    Some(time) => time.elapsed() < Duration::from_secs(5),
                    None => false
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

    fn act(&self, decision: Decision, fc: &mut FlightController, payload: &mut PayloadManager) {
        match decision {
            Decision::ChangeAltitude(delta) => {
                fc.adjust_altitude(delta);
            },
            Decision::SwitchPayloadMode => {
                payload.toggle_operational();
            },
            // ... other actions
        }
    }
}