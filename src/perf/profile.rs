use from_hashmap::FromHashmap;
use std::collections::HashMap;
use std::error::Error;
use std::vec::IntoIter;
pub trait FromHashmap<T>: Default {
    fn from_hashmap(hm: HashMap<String, u128>) -> T;
}
#[derive(Debug, Default, FromHashmap)]
pub struct Measurement {
    l1_dcache_loads: u128,
    l1_dcache_load_misses: u128,
    l1_icache_load_misses: u128,
    llc_load_misses: u128,
    llc_loads: u128,
    cycles: u128,
    instructions: u128,
}

#[derive(Debug)]
pub struct PerfProfile {
    pub measurements: Vec<Measurement>,
}

type Record = (String, u128, Option<String>, String);

impl PerfProfile {
    pub fn from_stream(s: String) -> Result<Self, Box<dyn Error>> {
        println!("{}", s);
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(s.as_bytes());
        let res: Vec<Record> = rdr.deserialize().filter_map(Result::ok).collect();

        Ok(Self {
            measurements: build_measurements(transpose(res)),
        })
    }
}

fn build_measurements(mut m: HashMap<String, Vec<u128>>) -> Vec<Measurement> {
    let keys: Vec<String> = m.keys().cloned().collect();
    let first_vec = m
        .keys()
        .next()
        .expect("perf should have returned some results");
    let mut res = vec![];
    for _ in 0..m[first_vec].len() {
        let mut measurement = HashMap::new();
        for k in &keys {
            measurement
                .entry(k.to_lowercase().replace("-", "_"))
                .or_insert_with(|| m.get_mut(&k[..]).unwrap().pop().unwrap());
        }
        res.push(Measurement::from_hashmap(measurement));
    }
    res
}

fn transpose(records: Vec<Record>) -> HashMap<String, Vec<u128>> {
    let mut res = HashMap::new();
    for r in records {
        let counter = res.entry(r.3).or_insert_with(|| vec![]);
        counter.push(r.1)
    }
    let l = res.values().next().unwrap().len();
    for (key, val) in &res {
        assert_eq!(l, val.len(), "{} different length", key);
    }
    res
}

// consume the Words structure
impl IntoIterator for PerfProfile {
    type Item = Measurement;
    type IntoIter = IntoIter<Self::Item>;

    // note that into_iter() is consuming self
    fn into_iter(self) -> Self::IntoIter {
        self.measurements.into_iter()
    }
}