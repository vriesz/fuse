// src/sensor_fusion/mod.rs

use nalgebra::Vector3;
use serde::{Serialize, Deserialize};
use crate::models::components::NeuralNetworkConfig;
use crate::comms::RadarContact;

// Vector3 serialization helpers
mod vector3_serde {
    use nalgebra::Vector3;
    use serde::{Serialize, Serializer, Deserialize, Deserializer};

    pub fn serialize<S>(vector: &Vector3<f32>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (vector.x, vector.y, vector.z).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vector3<f32>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (x, y, z) = <(f32, f32, f32)>::deserialize(deserializer)?;
        Ok(Vector3::new(x, y, z))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IMUReading {
    #[serde(with = "vector3_serde")]
    pub accel: Vector3<f32>,
    #[serde(with = "vector3_serde")]
    pub gyro: Vector3<f32>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPSPosition {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f32,
    pub accuracy: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpsData {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub variance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorData {
    pub imu: IMUReading,
    pub gps: Option<GPSPosition>,
    pub lidar: Option<f32>,
    pub radar_contacts: Vec<RadarContact>,
    pub operator_messages: usize,
    pub payload_status: (f32, bool),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KalmanConfig {
    pub process_noise: f32,
    pub sensor_weights: (f32, f32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KalmanFilter {
    #[serde(with = "vector3_serde")]
    state: Vector3<f32>,
    covariance: f32,
    config: KalmanConfig,
}

impl KalmanFilter {
    pub fn new(config: KalmanConfig) -> Self {
        Self {
            state: Vector3::zeros(),
            covariance: 1.0,
            config,
        }
    }

    pub fn update(&mut self, imu: &IMUReading, gps: Option<GpsData>, dt: f32) {
        // Prediction step (IMU only)
        self.state += imu.accel * dt;
        self.covariance += self.config.process_noise;

        // Update step (GPS correction)
        if let Some(gps) = gps {
            let k = self.covariance / (self.covariance + gps.variance);
            let gps_vec = Vector3::new(gps.x, gps.y, gps.z);
            self.state = self.state + (gps_vec - self.state) * k;
            self.covariance *= 1.0 - k;
        }
    }

    pub fn current_estimate(&self) -> (Vector3<f32>, f32) {
        (self.state, 1.0 - self.covariance)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataFusion {
    KalmanFilter(KalmanConfig),
    DnnFusion(NeuralNetworkConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionEstimate {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub certainty: f32,
}

pub fn fuse_data(data: SensorData, fusion_method: &DataFusion) -> PositionEstimate {
    match fusion_method {
        DataFusion::KalmanFilter(config) => {
            let mut kf = KalmanFilter::new(config.clone());
            
            // Convert GPSPosition to GpsData if available
            let gps_data = data.gps.as_ref().map(|gps| GpsData {
                x: gps.latitude as f32,
                y: gps.longitude as f32,
                z: gps.altitude,
                variance: gps.accuracy,
            });
            
            kf.update(&data.imu, gps_data, 0.1); // Using fixed dt for example
            
            let (position, certainty) = kf.current_estimate();
            PositionEstimate {
                x: position.x,
                y: position.y,
                z: position.z,
                certainty: certainty.clamp(0.0, 1.0),
            }
        }
        DataFusion::DnnFusion(_) => {
            // Placeholder for neural network implementation
            PositionEstimate {
                x: 0.0, y: 0.0, z: 0.0,
                certainty: 0.95
            }
        }
    }
}

// OODA loop related structures for sensor fusion
#[derive(Debug, Clone, PartialEq)]
pub enum ThreatLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone)]
pub struct Situation {
    pub threat_level: ThreatLevel,
    // Add other fields as needed
}

#[derive(Debug, Clone)]
pub struct SensorFusion {
    // Add necessary fields
}

impl Default for SensorFusion {
    fn default() -> Self {
        Self {
            // Initialize fields
        }
    }
}

impl SensorFusion {
    pub fn analyze(&mut self, data: &SensorData) -> Situation {
        // Simple implementation
        Situation {
            threat_level: if data.radar_contacts.len() > 2 {
                ThreatLevel::High
            } else if data.operator_messages > 0 {
                ThreatLevel::Medium
            } else {
                ThreatLevel::Low
            },
        }
    }
}