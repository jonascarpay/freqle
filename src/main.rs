mod myclap;

use chrono::{DateTime, Utc};
use myclap::{BumpArgs, Command};
use serde::{Deserialize, Serialize};
use std::{
    collections::hash_map::HashMap,
    fs::{File, OpenOptions},
    io,
    path::PathBuf,
};

fn main() -> Result<()> {
    let cmd = myclap::parse_args();
    match &cmd {
        Command::Bump(args) => bump(args),
        Command::View(_) => todo!(),
        Command::Delete(_) => todo!(),
    }
}

fn bump(args: &BumpArgs) -> Result<()> {
    let mut tbl = Table::load(&args.path, args.strict)?;
    tbl.decay();
    match &args.key {
        Some(key) => tbl.bump(key),
        None => (),
    }
    tbl.expire(args.threshold);
    tbl.write(&args.path)?;
    Ok(())
}

#[derive(Debug)]
enum FreqleError {
    // TODO: take the path as an argument
    StrictFileMissing,
    IOError(io::Error),
    BinError(bincode::Error),
}

type Result<T> = std::result::Result<T, FreqleError>;

#[derive(Serialize, Deserialize, Debug)]
struct Energies {
    hourly: f64,
    daily: f64,
    monthly: f64,
}

impl Energies {
    fn bump(&mut self) {
        self.hourly += 1.0;
        self.daily += 1.0;
        self.monthly += 1.0;
    }
    pub fn new() -> Energies {
        Energies {
            hourly: 1.0,
            daily: 1.0,
            monthly: 1.0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Table {
    last_update: DateTime<Utc>,
    energies: HashMap<String, Energies>,
}

impl Table {
    pub fn new() -> Table {
        Table {
            last_update: Utc::now(),
            energies: HashMap::new(),
        }
    }

    pub fn expire(&mut self, threshold: f64) {
        self.energies.retain(|_, erg| erg.monthly > threshold);
    }

    pub fn decay(&mut self) {
        let now = Utc::now();
        let delta = (now - self.last_update).num_seconds() as f64;
        let alpha_hour = 0.5_f64.powf(delta / 3600.0);
        let alpha_day = 0.5_f64.powf(delta / 24.0 * 3600.0);
        let alpha_month = 0.5_f64.powf(delta / 30.0 * 24.0 * 3600.0);
        for (_, elt) in self.energies.iter_mut() {
            elt.hourly *= alpha_hour;
            elt.daily *= alpha_day;
            elt.monthly *= alpha_month;
        }
        self.last_update = now;
    }

    pub fn bump(&mut self, key: &String) {
        self.energies
            .entry(key.clone()) // TODO don't clone
            .and_modify(|erg| erg.bump())
            .or_insert(Energies::new());
    }

    fn load(p: &PathBuf, strict: bool) -> Result<Table> {
        if p.exists() {
            let file: File = File::open(p).map_err(FreqleError::IOError)?;
            let tbl: Table = bincode::deserialize_from(&file).map_err(FreqleError::BinError)?;
            Ok(tbl)
        } else if strict {
            Err(FreqleError::StrictFileMissing)
        } else {
            Ok(Table::new())
        }
    }

    fn write(&self, p: &PathBuf) -> Result<()> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(p)
            .map_err(FreqleError::IOError)?;
        bincode::serialize_into(file, self).map_err(FreqleError::BinError)
    }
}
