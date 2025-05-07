// src/models/markov_chain.rs

use rand::Rng;
use std::fmt;
use serde::{Serialize, Deserialize};

/// A Markov model for predicting environment state transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkovEnvironmentModel {
    pub states: Vec<String>,                      // Environment states (clear, rain, fog, etc.)
    pub transition_matrix: Vec<Vec<f64>>,         // Transition probabilities
    pub current_state_idx: usize,                 // Current state index
}

impl MarkovEnvironmentModel {
    /// Creates a new Markov environment model with given states, transition probabilities and initial state
    // pub fn new(states: Vec<String>, transition_matrix: Vec<Vec<f64>>, initial_state: usize) -> Self {
        pub fn new(states: &[String], transition_matrix: Vec<Vec<f64>>, initial_state: usize) -> Self {
            // Validate transition matrix dimensions
            assert_eq!(states.len(), transition_matrix.len(), 
                       "Number of states must match number of rows in transition matrix");
        
        for (i, row) in transition_matrix.iter().enumerate() {
            assert_eq!(states.len(), row.len(), 
                       "Row {} of transition matrix must have same length as number of states", i);
            
            // Validate that each row sums to approximately 1.0
            let sum: f64 = row.iter().sum();
            assert!((sum - 1.0).abs() < 0.0001, 
                    "Row {} of transition matrix must sum to 1.0, got {}", i, sum);
        }
        
        // Validate initial state is within bounds
        assert!(initial_state < states.len(), 
                "Initial state index {} must be less than number of states {}", 
                initial_state, states.len());
        
        Self {
            states: states.to_vec(),
            transition_matrix,
            current_state_idx: initial_state,
        }
    }
    
    /// Predicts the next state based on current state's transition probabilities
    pub fn predict_next_state(&self) -> usize {
        // Use current state's transition probabilities to predict the next state
        let transition_probs = &self.transition_matrix[self.current_state_idx];
        
        // Generate random number between 0 and 1
        let mut rng = rand::thread_rng();
        let rand_val: f64 = rng.gen();
        
        // Find the next state based on transition probabilities
        let mut cumulative_prob = 0.0;
        for (next_state, prob) in transition_probs.iter().enumerate() {
            cumulative_prob += prob;
            if rand_val < cumulative_prob {
                return next_state;
            }
        }
        
        // Default to current state if something goes wrong
        self.current_state_idx
    }
    
    /// Updates the current state with observation or prediction
    pub fn update_state(&mut self, observed_state: Option<usize>) {
        if let Some(state) = observed_state {
            // Validate observed state
            assert!(state < self.states.len(), 
                    "Observed state index {} is out of bounds", state);
            self.current_state_idx = state;
        } else {
            // If no observation, predict next state
            self.current_state_idx = self.predict_next_state();
        }
    }
    
    /// Returns the name of the current state
    pub fn get_current_state(&self) -> &str {
        &self.states[self.current_state_idx]
    }
    
    /// Returns the name of a specific state by index
    pub fn get_state_name(&self, state_idx: usize) -> &str {
        assert!(state_idx < self.states.len(), 
                "State index {} is out of bounds", state_idx);
        &self.states[state_idx]
    }
    
    /// Returns the probability of transitioning from current state to target state
    pub fn get_transition_probability(&self, target_state: usize) -> f64 {
        assert!(target_state < self.states.len(),
                "Target state index {} is out of bounds", target_state);
        
        self.transition_matrix[self.current_state_idx][target_state]
    }
    
    /// Returns the most likely next state
    pub fn get_most_likely_next_state(&self) -> usize {
        let transition_probs = &self.transition_matrix[self.current_state_idx];
        
        transition_probs.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(idx, _)| idx)
            .unwrap_or(self.current_state_idx)
    }
}

impl fmt::Display for MarkovEnvironmentModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Current state: {} ({})", 
                 self.get_current_state(), self.current_state_idx)?;
        
        writeln!(f, "Transition probabilities:")?;
        for (i, state) in self.states.iter().enumerate() {
            write!(f, "  From {}: ", state)?;
            for (j, prob) in self.transition_matrix[i].iter().enumerate() {
                write!(f, "{}->{}: {:.2}, ", state, self.states[j], prob)?;
            }
            writeln!(f)?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_markov_model_creation() {
        let states = vec!["clear".to_string(), "rain".to_string(), "fog".to_string()];
        let transition_matrix = vec![
            vec![0.7, 0.2, 0.1],  // from clear
            vec![0.3, 0.4, 0.3],  // from rain
            vec![0.2, 0.3, 0.5],  // from fog
        ];
        
        let model = MarkovEnvironmentModel::new(&states, transition_matrix, 0);
        
        assert_eq!(model.get_current_state(), "clear");
        assert_eq!(model.get_transition_probability(1), 0.2); // clear -> rain
    }
    
    #[test]
    #[should_panic(expected = "Number of states must match")]
    fn test_invalid_transition_matrix() {
        let states = vec!["clear".to_string(), "rain".to_string()];
        let transition_matrix = vec![
            vec![0.7, 0.3],  // from clear
            vec![0.4, 0.6],  // from rain
            vec![0.2, 0.8],  // extra row!
        ];
        
        MarkovEnvironmentModel::new(&states, transition_matrix, 0);
    }
    
    #[test]
    #[should_panic(expected = "Row 0 of transition matrix must sum to 1.0")]
    fn test_invalid_probabilities() {
        let states = vec!["clear".to_string(), "rain".to_string()];
        let transition_matrix = vec![
            vec![0.7, 0.5],  // sums to 1.2
            vec![0.4, 0.6],  // sums to 1.0
        ];
        
        MarkovEnvironmentModel::new(&states, transition_matrix, 0);
    }
    
    #[test]
    fn test_update_state() {
        let states = vec!["clear".to_string(), "rain".to_string(), "fog".to_string()];
        let transition_matrix = vec![
            vec![0.7, 0.2, 0.1],
            vec![0.3, 0.4, 0.3],
            vec![0.2, 0.3, 0.5],
        ];
        
        let mut model = MarkovEnvironmentModel::new(&states, transition_matrix, 0);
        assert_eq!(model.get_current_state(), "clear");
        
        model.update_state(Some(1));
        assert_eq!(model.get_current_state(), "rain");
    }
    
    #[test]
    fn test_most_likely_next_state() {
        let states = vec!["clear".to_string(), "rain".to_string(), "fog".to_string()];
        let transition_matrix = vec![
            vec![0.7, 0.2, 0.1],  // Most likely: clear
            vec![0.3, 0.4, 0.3],  // Most likely: rain
            vec![0.2, 0.3, 0.5],  // Most likely: fog
        ];
        
        let mut model = MarkovEnvironmentModel::new(&states, transition_matrix, 0);
        assert_eq!(model.get_most_likely_next_state(), 0); // clear -> clear
        
        model.update_state(Some(2));
        assert_eq!(model.get_most_likely_next_state(), 2); // fog -> fog
    }
}