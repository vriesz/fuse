// src/physical/interference.rs

use super::topology::{ComponentId, ConnectionType, PhysicalTopology};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EmissionType {
    Magnetic,
    Electrical,
    RadioFrequency(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmissionSource {
    pub component: ComponentId,
    pub emission_type: EmissionType,
    pub strength: f32,     // Relative strength from 0-10
    pub falloff_rate: f32, // How quickly it diminishes with distance
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmcProperties {
    pub sources: Vec<EmissionSource>,
    pub susceptibility: HashMap<ComponentId, HashMap<EmissionType, f32>>,
}

impl EmcProperties {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            susceptibility: HashMap::new(),
        }
    }

    pub fn add_emission_source(&mut self, source: EmissionSource) {
        self.sources.push(source);
    }

    pub fn set_susceptibility(
        &mut self,
        component: ComponentId,
        emission_type: EmissionType,
        factor: f32,
    ) {
        self.susceptibility
            .entry(component)
            .or_insert_with(HashMap::new)
            .insert(emission_type, factor);
    }

    pub fn calculate_interference_impact(
        &self,
        topology: &PhysicalTopology,
    ) -> HashMap<(ComponentId, ComponentId), f32> {
        let mut impact = HashMap::new();

        // For each connection in the topology
        for ((from, to), connection) in &topology.connections {
            let mut interference_factor = 0.0;

            // For each emission source
            for source in &self.sources {
                if source.component == *from || source.component == *to {
                    // Skip if source is one of the endpoints (self-interference handled elsewhere)
                    continue;
                }

                // Get position of the source
                if let Some(source_component) = topology.components.get(&source.component) {
                    let source_pos = &source_component.position;

                    // Check both endpoints of the connection
                    if let (Some(from_component), Some(to_component)) =
                        (topology.components.get(from), topology.components.get(to))
                    {
                        let from_pos = &from_component.position;
                        let to_pos = &to_component.position;

                        // Calculate distances
                        let dist_to_from = source_pos.distance_to(from_pos);
                        let dist_to_to = source_pos.distance_to(to_pos);

                        // Calculate emission intensity at both endpoints using inverse square law
                        let intensity_at_from =
                            source.strength / (dist_to_from.powf(source.falloff_rate));
                        let intensity_at_to =
                            source.strength / (dist_to_to.powf(source.falloff_rate));

                        // Get susceptibility factors for each endpoint
                        let from_susceptibility = self
                            .susceptibility
                            .get(from)
                            .and_then(|map| map.get(&source.emission_type))
                            .cloned()
                            .unwrap_or(0.0);

                        let to_susceptibility = self
                            .susceptibility
                            .get(to)
                            .and_then(|map| map.get(&source.emission_type))
                            .cloned()
                            .unwrap_or(0.0);

                        // Calculate interference at each endpoint
                        let from_interference = intensity_at_from * from_susceptibility;
                        let to_interference = intensity_at_to * to_susceptibility;

                        // Use the maximum as the interference factor for this connection
                        interference_factor += from_interference.max(to_interference);
                    }
                }
            }

            // Apply interference based on connection type
            let connection_factor = match &connection.connection_type {
                ConnectionType::Copper { shielded, .. } => {
                    if *shielded {
                        0.2
                    } else {
                        1.0
                    }
                }
                ConnectionType::FiberOptic { .. } => 0.01, // Highly resistant
                ConnectionType::PcbTrace { .. } => 0.5,
                ConnectionType::Wireless { .. } => 1.5, // More susceptible
            };

            // Store the final interference factor
            impact.insert(
                (from.clone(), to.clone()),
                interference_factor * connection_factor,
            );
        }

        impact
    }
}
