// src/sensor_fusion/mod.rs
pub struct SensorData {
    pub imu: IMUReading,
    pub gps: Option<GPSPosition>,
    pub lidar: Option<f32>,  // Distance in meters
}

pub fn fuse_data(data: SensorData) -> PositionEstimate {
    // Kalman filter implementation would go here
    PositionEstimate {
        x: 0.0, y: 0.0, z: 0.0,
        certainty: 0.95
    }
}

use nalgebra::Vector3;

pub struct KalmanFilter {
    state: Vector3<f32>,  // x, y, z
    covariance: f32,
}

impl KalmanFilter {
    pub fn new() -> Self {
        Self {
            state: Vector3::zeros(),
            covariance: 1.0,
        }
    }

    pub fn update(&mut self, imu: &ImuData, gps: Option<GpsData>, dt: f32) {
        // Prediction step (IMU only)
        self.state += Vector3::new(imu.accel.x, imu.accel.y, imu.accel.z) * dt;
        self.covariance += 0.1;  // Process noise

        // Update step (GPS correction)
        if let Some(gps) = gps {
            let k = self.covariance / (self.covariance + gps.variance);
            self.state = self.state + k * (Vector3::new(gps.x, gps.y, gps.z) - self.state);
            self.covariance *= 1.0 - k;
        }
    }
}

// In models/components.rs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataFusion {
    KalmanFilter(KalmanConfig),
    DnnFusion(NeuralNetworkConfig),
}

pub struct KalmanConfig {
    pub process_noise: f32,
    pub sensor_weights: (f32, f32),  // IMU vs GPS trust
}