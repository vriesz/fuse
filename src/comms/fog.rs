// src/comms/fog.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeTask {
    pub id: String,
    pub cpu_load: f32,
    pub memory_mb: f32,
    pub deadline_ms: u32,
    pub priority: TaskPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeNode {
    pub id: String,
    pub available_cpu: f32,
    pub available_memory_mb: f32,
    pub network_latency_ms: f32,
    pub assigned_tasks: Vec<ComputeTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FogComputingManager {
    pub local_cpu_threshold: f32,
    pub edge_nodes: HashMap<String, EdgeNode>,
    pub task_queue: Vec<ComputeTask>,
}

impl FogComputingManager {
    pub fn new(threshold: f32) -> Self {
        Self {
            local_cpu_threshold: threshold,
            edge_nodes: HashMap::new(),
            task_queue: Vec::new(),
        }
    }

    pub fn add_edge_node(&mut self, id: String, cpu: f32, memory: f32, latency: f32) {
        self.edge_nodes.insert(
            id.clone(),
            EdgeNode {
                id,
                available_cpu: cpu,
                available_memory_mb: memory,
                network_latency_ms: latency,
                assigned_tasks: Vec::new(),
            },
        );
    }

    pub fn queue_task(&mut self, task: ComputeTask) {
        self.task_queue.push(task);
    }

    pub fn distribute_tasks(&mut self) -> usize {
        let mut assigned_count = 0;

        self.task_queue
            .sort_by(|a, b| (b.priority.clone() as u8).cmp(&(a.priority.clone() as u8)));
        let tasks = std::mem::take(&mut self.task_queue);
        for task in tasks {
            if self.assign_task(&task) {
                assigned_count += 1;
            } else {
                self.task_queue.push(task);
            }
        }

        assigned_count
    }

    // Change the assign_task signature to take a reference
    fn assign_task(&mut self, task: &ComputeTask) -> bool {
        // Find best node for task
        let mut best_node: Option<&mut EdgeNode> = None;
        let mut best_score = f32::MAX;

        for node in self.edge_nodes.values_mut() {
            if node.available_cpu >= task.cpu_load && node.available_memory_mb >= task.memory_mb {
                let score = node.network_latency_ms * (1.0 - node.available_cpu);
                if score < best_score {
                    best_score = score;
                    best_node = Some(node);
                }
            }
        }

        if let Some(node) = best_node {
            node.available_cpu -= task.cpu_load;
            node.available_memory_mb -= task.memory_mb;
            // Clone the task when adding it to the node's assigned_tasks
            node.assigned_tasks.push(task.clone());
            return true;
        }

        false
    }
}
