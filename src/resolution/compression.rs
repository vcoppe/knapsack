use std::collections::HashMap;

use clustering::{kmeans, Elem};
use ddo::{Compression, Problem, Decision, Dominance};

use crate::instance::KnapsackInstance;

use super::model::{Knapsack, KnapsackState};

struct Item<'a> {
    id: usize,
    pb: &'a Knapsack,
}

impl<'a> Elem for Item<'a> {
    fn dimensions(&self) -> usize {
        2
    }

    fn at(&self, i: usize) -> f64 {
        if i == 0 {
            self.pb.instance.weight[self.id] as f64
        } else {
            self.pb.instance.profit[self.id] as f64
        }
    }
}

pub struct KnapsackCompression<'a> {
    pub problem: &'a Knapsack,
    pub meta_problem: Knapsack,
    pub mapping: HashMap<isize, isize>,
}

impl<'a> KnapsackCompression<'a> {
    pub fn new(problem: &'a Knapsack, n_meta_items: usize) -> Self {
        let mut elems = vec![];
        for i in 0..problem.instance.nb_items {
            elems.push(Item {
                id: i,
                pb: problem,
            });
        }
        let clustering = kmeans(n_meta_items, Some(0), &elems, 1000);

        let weight = Self::compute_meta_weight(problem, &clustering.membership, n_meta_items);

        let meta_instance = KnapsackInstance {
            nb_items: problem.instance.nb_items,
            capacity: problem.instance.capacity,
            weight,
            profit: problem.instance.profit.clone(),
        };
        let meta_problem = Knapsack {
            instance: meta_instance,
            order: problem.order.clone(),
        };

        let mut mapping = HashMap::new();
        mapping.insert(0, 0);
        mapping.insert(1, 1);

        KnapsackCompression {
            problem,
            meta_problem,
            mapping,
        }
    }

    fn compute_meta_weight(pb: &Knapsack, membership: &Vec<usize>, n_meta_items: usize) -> Vec<isize> {
        let mut meta_weight = vec![isize::MAX; n_meta_items];

        for (i, j) in membership.iter().copied().enumerate() {
            meta_weight[j] = meta_weight[j].min(pb.instance.weight[i]);
        }
        
        (0..pb.instance.nb_items).map(|i| meta_weight[membership[i]]).collect()
    }
}

impl<'a> Compression for KnapsackCompression<'a> {
    type State = KnapsackState;

    fn get_compressed_problem(&self) -> &dyn Problem<State = Self::State> {
        &self.meta_problem
    }

    fn compress(&self, state: &KnapsackState) -> KnapsackState {
        state.clone()
    }

    fn decompress(&self, solution: &Vec<Decision>) -> Vec<Decision> {
        solution.clone()
    }
}

#[derive(PartialEq, Eq, Hash)]
pub struct KnapsackKey {
    pub depth: usize,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct KnapsackValue {
    pub capacity: isize,
}

pub struct KnapsackDominance;
impl Dominance for KnapsackDominance {
    type State = KnapsackState;
    type Key = KnapsackKey;
    type Value = KnapsackValue;

    fn get_key(&self, state: &Self::State) -> Self::Key {
        KnapsackKey {
            depth: state.depth,
        }
    }

    fn get_value(&self, state: &Self::State) -> Self::Value {
        KnapsackValue {
            capacity: state.capacity,
        }
    }

    fn is_dominated_by(&self, a: &Self::Value, b: &Self::Value) -> bool {
        a.capacity <= b.capacity
    }
}