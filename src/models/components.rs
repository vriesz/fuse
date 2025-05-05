// src/models/components.rs

use serde::{Serialize, Deserialize};
use crate::flight_control::FlightController;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KalmanConfig {
    pub process_noise: f32,
    pub sensor_weights: (f32, f32),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NeuralNetworkConfig {
    pub model_path: String,
    pub inference_rate: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PIDParams {
    pub roll: (f32, f32, f32),
    pub pitch: (f32, f32, f32),
    pub yaw: (f32, f32, f32),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataFusion {
    KalmanFilter(KalmanConfig),
    DnnFusion(NeuralNetworkConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FlightControllerType {
    PX4(PIDParams),
    ArduPilot(PIDParams),
    Betaflight,
    Custom(PIDParams),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CommsSystem {
    MAVLink { version: u8 },
    LoRa { frequency: u32 },
    WiFiDirect { bandwidth: u32 },
    MilitaryEncrypted { key_rotation: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SensorSuite {
    BasicImu,
    GpsEnhanced,
    FullNavigation,
    Autonomous,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Processor {
    XilinxZynqUltraScale,
    NvidiaJetsonAGXOrin,
    QualcommRB5,
    IntelCorei7,
    RaspberryPiCM4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UAVArchitecture {
    pub processor: Processor,
    pub data_fusion: DataFusion,
    pub flight_control: FlightControllerType,
    pub sensors: SensorSuite,
    pub comms: CommsSystem,
}

impl From<FlightControllerType> for FlightController {
    fn from(fc_type: FlightControllerType) -> Self {
        match fc_type {
            FlightControllerType::PX4(params) => Self::from_params(params.roll),
            FlightControllerType::ArduPilot(params) => Self::from_params(params.pitch),
            FlightControllerType::Custom(params) => Self::from_params(params.yaw),
            FlightControllerType::Betaflight => Self::new(), // Uses default PID values
        }
    }
}