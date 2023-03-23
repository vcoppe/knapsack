//! This module defines an abstract representation of a knapsack instance.

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnapsackInstance {
    pub nb_items: usize,
    pub capacity: isize,
    pub weight: Vec<isize>,
    pub profit: Vec<isize>,
}
