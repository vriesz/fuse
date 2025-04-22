use crate::{payload::PayloadManager, comms::CommunicationHub};

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
            comms: CommunicationHub::new(),
            flight_controller: FlightController::new(),
            // ... other initializations
        }
    }
}