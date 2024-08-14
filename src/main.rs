use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use num_bigint::BigUint;
use num_traits::{One, ToPrimitive};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Eq, Hash, Clone)]
struct Key {
    m: usize,
    n: usize,
}

impl Key {
    fn new(m: usize, n: usize) -> Self {
        Key { m, n }
    }
}

type GBCoef = Vec<BigUint>;

struct GaussianBinomial {
    is_cached_table: HashMap<Key, bool>,
    cached_lookup_table: HashMap<Key, GBCoef>,
    data_path: PathBuf,
}

impl GaussianBinomial {
    fn from(data_path: &Path) -> Self {
        if !data_path.join("cache_table.bin").exists() {
            std::fs::create_dir_all(data_path).unwrap();
            return Self {
                is_cached_table: HashMap::<Key, bool>::new(),
                cached_lookup_table: HashMap::<Key, GBCoef>::new(),
                data_path: PathBuf::from(data_path),
            };
        }

        let is_cached_table: HashMap<Key, bool> = deserialize(data_path.join("cache_table.bin"));
        Self {
            is_cached_table,
            cached_lookup_table: HashMap::<Key, GBCoef>::new(),
            data_path: PathBuf::from(data_path),
        }
    }

    fn is_cached(&self, m: usize, n: usize) -> bool {
        self.is_cached_table.contains_key(&Key { m, n })
    }

    // from t-1 to compute t
    fn compute_helper(&mut self, t: usize) {
        // m , n, l
        // =>  (m-1 , n, l-n) + (m, n-1, l)
        if t == 0 {
            self.is_cached_table.insert(Key::new(t, t), true);
            self.cached_lookup_table
                .insert(Key::new(t, t), vec![BigUint::one()]);
            return;
        }

        let is_cached = (1..=(t - 1)).all(|a| self.is_cached(a, t - 1) && self.is_cached(t - 1, a));
        if !is_cached {
            panic!("required result is not cached");
        }

        let lookup_table = &mut self.cached_lookup_table;
        for m in 1..=(t - 1) {
            lookup_table.insert(Key::new(m, t), coef(m, t, lookup_table));
            lookup_table.insert(Key::new(t, m), coef(t, m, lookup_table));
            lookup_table.remove(&Key::new(m, t - 1));
            lookup_table.remove(&Key::new(t - 1, m));
            self.is_cached_table.insert(Key::new(m, t), true);
            self.is_cached_table.insert(Key::new(t, m), true);
        }

        lookup_table.insert(Key::new(t, t), coef(t, t, lookup_table));
        self.is_cached_table.insert(Key::new(t, t), true);

        // let r = lookup_table[&Key::new(t, t)].clone();

        // serialize(self.data_path.join(format!("GB_{t}.bin")), lookup_table);
        // serialize(
        //     self.data_path.join("cache_table.bin"),
        //     self.is_cached_table.clone(),
        // );
    }

    // compute(from, from+1) == compute_helper(from+1)
    fn compute(&mut self, from: usize, to: usize) -> GBCoef {
        // m , n, l
        // =>  (m-1 , n, l-n) + (m, n-1, l)

        if from == 0 {
            self.compute_helper(from);
        } else {
            // load data for from
            let is_cached = (1..=from).all(|a| self.is_cached(a, from) && self.is_cached(from, a));
            if !is_cached {
                panic!("required result is not cached");
            }
            self.cached_lookup_table = deserialize(self.data_path.join(format!("GB_{}.bin", from)));
        }

        for m in (from + 1)..=to {
            self.compute_helper(m);
        }

        let r = self.cached_lookup_table[&Key::new(to, to)].clone();

        serialize(
            self.data_path.join(format!("GB_{to}.bin")),
            self.cached_lookup_table.clone(),
        );
        serialize(
            self.data_path.join("cache_table.bin"),
            self.is_cached_table.clone(),
        );

        r
    }
}

// m x n, l
// =>  (m-1 x n, l-n) + (m, n-1, l)
fn coef(m: usize, n: usize, map: &HashMap<Key, Vec<BigUint>>) -> Vec<BigUint> {
    let mut res = Vec::<BigUint>::with_capacity(m * n + 1);
    if m == 1usize || n == 1usize {
        for _ in 0usize..=(m * n) {
            res.push(BigUint::one());
        }
        return res;
    }

    // println!("{m}, {n}");
    // println!("{:#?}", map.keys());
    let first = map.get(&Key::new(m - 1usize, n)).unwrap();
    let second = map.get(&Key::new(m, n - 1usize)).unwrap();

    for l in 0usize..n {
        // first term is 0
        res.push(second[l].clone());
    }

    for l in n..=(m * n - m) {
        // first term is nonzero
        res.push(first[l - n].clone() + second[l].clone());
    }

    for l in (m * n - m + 1usize)..=(m * n) {
        // first term is nonzero
        res.push(first[l - n].clone());
    }

    res
}

fn serialize<T, F>(filename: F, res: HashMap<Key, T>)
where
    T: Serialize,
    F: AsRef<Path>,
{
    let mut s = flexbuffers::FlexbufferSerializer::new();
    res.into_iter()
        .map(|(k, v)| (k.m, k.n, v))
        .collect::<Vec<(usize, usize, T)>>()
        .serialize(&mut s)
        .unwrap();

    std::fs::write(filename, s.view()).expect("Unable to write file");
}

fn deserialize<T, F>(filename: F) -> HashMap<Key, T>
where
    T: DeserializeOwned,
    F: AsRef<Path>,
{
    let data = std::fs::read(filename).expect("Unable to read file");
    let r = flexbuffers::Reader::get_root(data.as_ref()).unwrap();
    let res_vec = Vec::<(usize, usize, T)>::deserialize(r).unwrap();

    res_vec
        .into_iter()
        .map(|(m, n, v)| (Key::new(m, n), v))
        .collect::<HashMap<Key, T>>()
}

fn main() {
    let data_path = std::path::PathBuf::from("data");
    let mut gb = GaussianBinomial::from(&data_path);

    let start = 0usize;
    let end = 512usize;
    let step = 16usize;
    for k in (start / step)..(end / step) {
        let r = gb.compute(k * step, (k + 1) * step);
        let n = (k + 1) * step;
        // if n & (n - 1) == 0 {
        if n % 32 == 0 {
            // a power of 2
            let s = r.iter().sum::<BigUint>().to_f64().unwrap().ln();

            let mut file = File::create(data_path.join(format!("{n}.dat"))).unwrap();

            for (l, x) in r.iter().enumerate() {
                let ln_x = x.to_f64().unwrap().ln();
                if let Err(e) = writeln!(file, "{} {} {} {} {}", n, n, l, ln_x, ln_x - s) {
                    eprintln!("Couldn't write to file: {}", e);
                }
            }
        }
    }
}
