// Copyright 2020 Xavier Gillard
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal in
// the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software is furnished to do so,
// subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
// FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
// COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
// IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
// CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

//! This example show how to implement a solver for the knapsack problem
//! using ddo. It is a fairly simple example but it features most of the aspects you will
//! want to copy when implementing your own solver.
//! 
use ddo::*;
use ordered_float::OrderedFloat;

use crate::instance::KnapsackInstance;

/// The state of the DP model
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KnapsackState {
    pub depth: usize,
    pub capacity: isize,
}

/// This structure describes a Knapsack instance
#[derive(Debug, Clone)]
pub struct Knapsack {
    pub instance: KnapsackInstance,
    order: Vec<usize>,
}

impl Knapsack {
    pub fn new(instance: KnapsackInstance) -> Self {
        let mut order = (0..instance.nb_items).collect::<Vec<usize>>();
        order.sort_unstable_by_key(|i| OrderedFloat(- instance.profit[*i] as f64 / instance.weight[*i] as f64));

        Knapsack { instance, order }
    }
}

impl Problem for Knapsack {
    type State = KnapsackState;

    fn nb_variables(&self) -> usize {
        self.instance.nb_items
    }

    fn initial_state(&self) -> Self::State {
        KnapsackState {
            depth: 0,
            capacity: self.instance.capacity,
        }
    }

    fn initial_value(&self) -> isize {
        0
    }

    fn transition(&self, state: &Self::State, decision: ddo::Decision) -> Self::State {
        KnapsackState {
            depth: state.depth + 1,
            capacity: state.capacity - decision.value * self.instance.weight[decision.variable.id()],
        }
    }

    fn transition_cost(&self, _: &Self::State, decision: ddo::Decision) -> isize {
        decision.value * self.instance.profit[decision.variable.id()]
    }

    fn next_variable(&self, depth: usize, _: &mut dyn Iterator<Item = &Self::State>)
        -> Option<ddo::Variable> {
        if depth < self.instance.nb_items {
            Some(Variable(self.order[depth]))
        } else {
            None
        }
    }

    fn for_each_in_domain(&self, variable: ddo::Variable, state: &Self::State, f: &mut dyn ddo::DecisionCallback) {
        f.apply(Decision {variable, value: 0});

        if state.capacity >= self.instance.weight[variable.id()] {
            f.apply(Decision {variable, value: 1});
        }
    }
}

/// This structure implements the Knapsack relaxation
pub struct KnapsackRelax {
    pb: Knapsack
}

impl KnapsackRelax {
    pub fn new(pb: Knapsack) -> Self {
        KnapsackRelax { pb }
    }
}

impl Relaxation for KnapsackRelax {
    type State = KnapsackState;

    fn merge(&self, states: &mut dyn Iterator<Item = &Self::State>) -> Self::State {
        KnapsackState {
            depth: states.next().map(|s| s.depth).unwrap_or(0),
            capacity: states.map(|s| s.capacity).max().unwrap_or(self.pb.instance.capacity),
        }
    }

    fn relax(&self, _: &Self::State, _: &Self::State, _:  &Self::State, _: Decision, cost: isize) -> isize {
        cost
    }

    fn fast_upper_bound(&self, state: &Self::State) -> isize {
        let mut depth = state.depth;
        let mut max_profit = 0;
        let mut capacity = state.capacity;

        while capacity > 0 && depth < self.pb.instance.nb_items {
            let item = self.pb.order[depth];

            if capacity >= self.pb.instance.weight[item] {
                max_profit += self.pb.instance.profit[item];
                capacity -= self.pb.instance.weight[item];
            } else {
                let item_ratio = capacity as f64 / self.pb.instance.weight[item] as f64;
                let item_profit = item_ratio * self.pb.instance.profit[item] as f64;
                max_profit += item_profit.floor() as isize;
                capacity = 0;
            }

            depth += 1;
        }

        max_profit
    }
}


/// The last bit of information which we need to provide when implementing a ddo-based
/// solver is a `StateRanking`. This is an heuristic which is used to select the most
/// and least promising nodes as a means to only delete/merge the *least* promising nodes
/// when compiling restricted and relaxed DDs.
pub struct KnapsackRanking;
impl StateRanking for KnapsackRanking {
    type State = KnapsackState;

    fn compare(&self, a: &Self::State, b: &Self::State) -> std::cmp::Ordering {
        a.capacity.cmp(&b.capacity)
    }
}
