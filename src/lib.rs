// src/lib.rs
pub mod comms;
pub mod flight_control;
pub mod models;
pub mod payload;
pub mod sensor_fusion;
pub mod ooda;
pub mod operations;
pub mod engine;

pub use models::constraints::UavConstraints;
pub use engine::generate_architecture;

#[cfg(test)]
mod tests {
    mod comms_tests;
    mod flight_control_tests;
    mod payload_tests;
    mod sensor_fusion_tests;
    mod ooda_tests;
    mod engine_tests;
    mod integration_tests;
    mod hitl_tests;
}