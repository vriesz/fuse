use crate::models::architecture::UavSystems;

// Add missing methods that benchmarks need
impl UavSystems {
    pub fn reset_to_position(&mut self, _x: f64, _y: f64, _z: f64) {
        // Mock implementation
    }
    
    pub fn get_power_consumption(&self) -> f64 {
        // Mock implementation
        20.0
    }
    
    pub fn mission_successful(&self) -> bool {
        // Mock implementation
        true
    }
}

// Add process_message to CommunicationHub
impl crate::comms::CommunicationHub {
    pub fn process_message(&mut self, _data: Vec<u8>) {
        // Mock implementation
    }
}