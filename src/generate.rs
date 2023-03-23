use std::{time::{SystemTime, UNIX_EPOCH}, fs::File, io::Write};

use clap::Args;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaChaRng;
use rand_distr::{Uniform, Normal, Distribution};

use crate::instance::KnapsackInstance;

#[derive(Debug, Args)]
pub struct KnapsackGenerator {
    /// An optional seed to kickstart the instance generation
    #[clap(short='s', long)]
    seed: Option<u128>,
    /// The number of items that must be produced
    #[clap(short='n', long, default_value="10")]
    nb_items: usize,
    /// The number of clusters of similar items
    #[clap(short='c', long, default_value="3")]
    nb_clusters: usize,
    /// The capacity of the knapsack
    #[clap(long, default_value="5000")]
    capacity: isize,
    /// The minimum weight
    #[clap(long, default_value="1000")]
    min_weight: usize,
    /// The maximum weight
    #[clap(long, default_value="10000")]
    max_weight: usize,
    /// The std deviation of the weight among a cluster
    #[clap(long, default_value="100")]
    weight_std_dev: usize,
    /// The minimum profit
    #[clap(long, default_value="1000")]
    min_profit: usize,
    /// The maximum profit
    #[clap(long, default_value="10000")]
    max_profit: usize,
    /// The std deviation of the profit among a cluster
    #[clap(long, default_value="100")]
    profit_std_dev: usize,
    /// Name of the file where to generate the knapsack instance
    #[clap(short, long)]
    output: Option<String>,
}

impl KnapsackGenerator {

    pub fn generate(&mut self) {
        if self.min_weight < self.weight_std_dev {
            self.max_weight += self.weight_std_dev - self.min_weight;
            self.min_weight = self.weight_std_dev;
        }

        let mut rng = self.rng();

        let mut nb_items_per_cluster = vec![self.nb_items / self.nb_clusters; self.nb_clusters];
        for i in 0..(self.nb_items % self.nb_clusters) {
            nb_items_per_cluster[i] += 1;
        }
        
        let weight = Self::generate_vec(&mut rng, self.nb_clusters, &nb_items_per_cluster, self.min_weight, self.max_weight, self.weight_std_dev);
        let profit = Self::generate_vec(&mut rng, self.nb_clusters, &nb_items_per_cluster, self.min_profit, self.max_profit, self.profit_std_dev);

        let instance = KnapsackInstance {
            nb_items: self.nb_items,
            capacity: self.capacity,
            weight,
            profit,
        };

        let instance = serde_json::to_string_pretty(&instance).unwrap();

        if let Some(output) = self.output.as_ref() {
            File::create(output).unwrap().write_all(instance.as_bytes()).unwrap();
        } else {
            println!("{instance}");
        }
    }

    fn generate_vec(rng: &mut impl Rng, nb_clusters: usize, nb_items_per_cluster: &Vec<usize>, min_val: usize, max_val: usize, std_dev: usize) -> Vec<isize> {
        let mut data = vec![];

        let rand_centroid = Uniform::new_inclusive(min_val, max_val);
        for i in 0..nb_clusters {
            let centroid = rand_centroid.sample(rng);
            let rand = Normal::new(centroid as f64, std_dev as f64).expect("cannot create normal dist");

            for _ in 0..nb_items_per_cluster[i] {
                data.push(rand.sample(rng).round() as isize);
            }
        }

        data
    }
    
    fn rng(&self) -> impl Rng {
        let init = self.seed.unwrap_or_else(|| SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis());
        let mut seed = [0_u8; 32];
        seed.iter_mut().zip(init.to_be_bytes().into_iter()).for_each(|(s, i)| *s = i);
        seed.iter_mut().rev().zip(init.to_le_bytes().into_iter()).for_each(|(s, i)| *s = i);
        ChaChaRng::from_seed(seed)
    }

}