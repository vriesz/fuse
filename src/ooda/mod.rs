use crate::{comms::*, payload::*, flight_control::*};
use std::time::{Instant, Duration};

pub struct OodaLoop {
    last_cycle_time: Duration,
    sensor_fusion: SensorFusion,
    decision_cache: Option<Decision>,
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
        let sensor_data = self.observe(comms);
        
        // ORIENT
        let situation = self.orient(&sensor_data);
        
        // DECIDE
        let decision = self.decide(&situation, payload);
        
        // ACT
        self.act(decision, flight_controller, payload);
        
        self.last_cycle_time = start_time.elapsed();
        self.last_cycle_time
    }

    fn observe(&self, comms: &CommunicationHub) -> SensorData {
        SensorData {
            radar_contacts: comms.radar_contacts.clone(),
            operator_messages: comms.operators.iter()
                .filter(|o| o.last_heartbeat.elapsed() < Duration::from_secs(5))
                .count(),
            payload_status: payload.get_status(),
            // ... other sensor inputs
        }
    }

    fn orient(&mut self, data: &SensorData) -> Situation {
        // AI/ML threat assessment would go here
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