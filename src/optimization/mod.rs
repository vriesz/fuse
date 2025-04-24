use crate::models::components::UavArchitecture;
use crate::models::components::Processor;

pub fn optimize_cost(architectures: Vec<UavArchitecture>) -> UavArchitecture {
    architectures.into_iter()
        .min_by_key(|arch| match arch.processor {
            Processor::XilinxZynqUltraScale => 1200,
            Processor::NvidiaJetsonAGXOrin => 800,
            Processor::QualcommRB5 => 300,
            Processor::IntelCorei7 => 600,
            Processor::RaspberryPiCM4 => 100,

        })
        .expect("At least one architecture should be provided")
}
