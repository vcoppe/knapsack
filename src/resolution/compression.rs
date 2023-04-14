use std::{collections::{HashMap, BTreeSet, HashSet}, mem::swap, ops::Bound::*};

use clustering::{kmeans, Elem};
use ddo::{Compression, Problem, Decision};

use crate::instance::KnapsackInstance;

use super::model::{Knapsack, KnapsackState};

struct Item<'a> {
    id: usize,
    pb: &'a Knapsack,
}

impl<'a> Elem for Item<'a> {
    fn dimensions(&self) -> usize {
        1
    }

    fn at(&self, _: usize) -> f64 {
        self.pb.instance.weight[self.id] as f64
    }
}

pub struct KnapsackCompression<'a> {
    pub problem: &'a Knapsack,
    pub meta_problem: Knapsack,
    pub mapping: HashMap<isize, isize>,
    states: Vec<BTreeSet<isize>>,
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

        let states = Self::compute_meta_states(&meta_problem);

        KnapsackCompression {
            problem,
            meta_problem,
            mapping,
            states,
        }
    }

    fn compute_meta_weight(pb: &Knapsack, membership: &Vec<usize>, n_meta_items: usize) -> Vec<isize> {
        let mut meta_weight = vec![isize::MAX; n_meta_items];

        for (i, j) in membership.iter().copied().enumerate() {
            meta_weight[j] = meta_weight[j].min(pb.instance.weight[i]);
        }
        
        (0..pb.instance.nb_items).map(|i| meta_weight[membership[i]]).collect()
    }

    fn compute_meta_states(meta_pb: &Knapsack) -> Vec<BTreeSet<isize>> {
        let mut map = vec![BTreeSet::new(); meta_pb.instance.nb_items];

        let mut depth = 0;
        let mut current = HashSet::new();
        let mut next = HashSet::new();

        current.insert(meta_pb.initial_state());

        while let Some(var) = meta_pb.next_variable(depth, &mut current.iter()) {
            for state in current.drain() {
                map[state.depth].insert(state.capacity);
                meta_pb.for_each_in_domain(var, &state, &mut |d| { next.insert(meta_pb.transition(&state, d)); });
            }

            swap(&mut current, &mut next);
            depth += 1;
        }

        map
    }
}

impl<'a> Compression for KnapsackCompression<'a> {
    type State = KnapsackState;

    fn get_compressed_problem(&self) -> &dyn Problem<State = Self::State> {
        &self.meta_problem
    }

    fn compress(&self, state: &KnapsackState) -> KnapsackState {
        match self.states[state.depth].range((Included(state.capacity), Unbounded)).next() {
            Some(capacity) => KnapsackState { depth: state.depth, capacity: *capacity },
            None => state.clone(),
        }
    }

    fn decompress(&self, solution: &Vec<Decision>) -> Vec<Decision> {
        solution.clone()
    }
}