use serde::{Serialize, Deserialize};
use crate::flight_control::FlightController;
use crate::models::constraints::MissionType;
use crate::payload::PayloadType;
use crate::comms::LinkType;
use crate::comms::CommunicationHub;
use crate::payload::PayloadManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UavSystems {
    pub payload: PayloadManager,
    pub comms: CommunicationHub,
    pub flight_controller: FlightController,
    // ... other systems
}

impl UavSystems {
    pub fn new(mission_type: MissionType) -> Self {
        let payload = match mission_type {
            MissionType::Surveillance => PayloadManager::new(
                Some(PayloadType::SurveillanceCamera {
                    resolution_mpx: 20.0,
                    zoom_level: 10,
                    thermal_capable: true,
                })
            ),
            MissionType::Strike => PayloadManager::new(None), // Weapons payload
            MissionType::Logistics => PayloadManager::new(
                Some(PayloadType::CargoContainer {
                    max_weight_kg: 50.0,
                    secure_locking: true,
                })
            ),
        };

        Self {
            payload,
            comms: CommunicationHub::new(LinkType::MAVLink { 
                version: 2, 
                heartbeat_interval_ms: 500 
            }, true),
            flight_controller: FlightController::new(),
            // ... other initializations
        }
    }
}