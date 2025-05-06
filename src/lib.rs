// src/lib.rs
pub mod benchmarks;
pub mod comms;
pub mod engine;
pub mod flight_control;
pub mod models;
pub mod ooda;
pub mod operations;
pub mod optimization;
pub mod payload;
pub mod physical;
pub mod sensor_fusion;

pub use engine::generate_architecture;
pub use models::constraints::UavConstraints;

#[cfg(test)]
mod tests {
    mod comms_tests;
    mod engine_tests;
    mod flight_control_tests;
    mod hitl_tests;
    mod integration_tests;
    mod ooda_tests;
    mod payload_tests;
    mod sensor_fusion_tests;
}
