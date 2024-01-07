mod myclap;

use chrono::{DateTime, Utc};
use myclap::{BumpArgs, Command, DeleteArgs, ViewArgs};
use ordered_float::NotNan;
use serde::{Deserialize, Serialize};
use std::{
    collections::hash_map::HashMap,
    fmt::Debug,
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, BufWriter, Write},
    ops,
    path::PathBuf,
};

fn main() {
    let cmd = myclap::parse_args();
    let res = match &cmd {
        Command::Bump(args) => run_bump(args),
        Command::View(args) => run_view(args),
        Command::Delete(args) => run_delete(args),
    };
    if let Err(err) = res {
        let path = cmd.get_path();
        match err {
            FreqleError::StrictFileMissing => {
                println!("Error: File {path:?} missing in --strict mode")
            }
            FreqleError::IOError(err) => eprintln!("IO error while reading {path:?}: {err}"),
            FreqleError::BinError(err) => eprintln!("Binary format error in {path:?}: {err}"),
            FreqleError::NumError => {
                eprintln!("Error: unexpected NaN! Please open a bug report.")
            }
        }
        std::process::exit(1);
    }
}

fn run_bump(args: &BumpArgs) -> Result<()> {
    let mut tbl = Table::load(&args.path, args.strict)?;
    tbl.decay();
    match &args.key {
        Some(key) => tbl.bump(key),
        None => (),
    }
    tbl.expire(args.threshold);
    tbl.write(&args.path)
}

fn run_view(args: &ViewArgs) -> Result<()> {
    let mut tbl = Table::load(&args.path, args.strict)?;
    tbl.decay();
    if args.augment || args.restrict {
        let stdin_buf = BufReader::new(std::io::stdin());
        let mlines: io::Result<Vec<String>> = stdin_buf.lines().collect();
        let lines = mlines.map_err(FreqleError::IOError)?;
        if args.augment {
            tbl.augment(&lines);
        }
        if args.restrict {
            tbl = tbl.restrict(&lines);
        }
    }
    let w = Weights {
        hourly: args.hourly,
        daily: args.daily,
        monthly: args.monthly,
    };
    let mut writer = BufWriter::new(std::io::stdout().lock());
    tbl.view(w, args.scores, &mut writer)?;
    Ok(())
}

fn run_delete(args: &DeleteArgs) -> Result<()> {
    let mut tbl = Table::load(&args.path, args.strict)?;
    tbl.delete(&args.key);
    tbl.write(&args.path)
}

enum FreqleError {
    StrictFileMissing,
    IOError(io::Error),
    BinError(bincode::Error),
    NumError,
}

type Result<T> = std::result::Result<T, FreqleError>;

type Energies = TVec3;
type Weights = TVec3;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
struct TVec3 {
    hourly: f64,
    daily: f64,
    monthly: f64,
}

impl TVec3 {
    pub fn new(c: f64) -> TVec3 {
        TVec3 {
            hourly: c,
            daily: c,
            monthly: c,
        }
    }
    fn pow(self, other: TVec3) -> TVec3 {
        TVec3 {
            hourly: self.hourly.powf(other.hourly),
            daily: self.daily.powf(other.daily),
            monthly: self.monthly.powf(other.monthly),
        }
    }
    fn dot(self, other: TVec3) -> f64 {
        self.hourly * other.hourly + self.daily * other.daily + self.monthly * other.monthly
    }
}

impl ops::Div for TVec3 {
    type Output = TVec3;
    fn div(self, other: TVec3) -> TVec3 {
        TVec3 {
            hourly: self.hourly / other.hourly,
            daily: self.daily / other.daily,
            monthly: self.monthly / other.monthly,
        }
    }
}

impl ops::AddAssign<f64> for TVec3 {
    fn add_assign(&mut self, rhs: f64) {
        self.hourly += rhs;
        self.daily += rhs;
        self.monthly += rhs;
    }
}

impl ops::MulAssign for TVec3 {
    fn mul_assign(&mut self, rhs: TVec3) {
        self.hourly *= rhs.hourly;
        self.daily *= rhs.daily;
        self.monthly *= rhs.monthly;
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
        let delta_sec = (now - self.last_update).num_seconds() as f64;
        let half_lives_sec = TVec3 {
            hourly: 3600.0,
            daily: 24.0 * 3600.0,
            monthly: 30.0 * 24.0 * 3600.0,
        };
        let alpha = TVec3::new(0.5).pow(TVec3::new(delta_sec) / half_lives_sec);
        for (_, elt) in self.energies.iter_mut() {
            *elt *= alpha;
        }
        self.last_update = now;
    }

    pub fn augment(&mut self, entries: &Vec<String>) {
        for entry in entries {
            // TODO ONLY IF NOT PRESENT
            self.energies.insert(
                entry.clone(), // TODO don't clone
                Energies::new(0.0),
            );
        }
    }

    pub fn restrict(&self, entries: &Vec<String>) -> Table {
        let mut tbl = Table {
            last_update: self.last_update,
            energies: HashMap::new(),
        };
        for entry in entries {
            if let Some(erg) = self.energies.get(entry) {
                tbl.energies.insert(entry.clone(), *erg);
            }
        }
        tbl
    }

    pub fn bump(&mut self, key: &str) {
        self.energies
            .entry(key.to_owned()) // TODO don't clone
            .and_modify(|erg| *erg += 1.0)
            .or_insert(Energies::new(1.0));
    }

    pub fn view<W: Write>(self, weights: Weights, scores: bool, tgt: &mut W) -> Result<()> {
        let mut entries: Vec<(&String, &TVec3, f64)> = self
            .energies
            .iter()
            .map(|(k, v)| (k, v, weights.dot(*v)))
            .collect();
        for (_, _, erg) in &entries {
            if erg.is_nan() {
                return Err(FreqleError::NumError);
            }
        }
        entries.sort_by_key(|(_, _, erg)| -NotNan::new(*erg).unwrap()); // TODO NotNan float
        if scores {
            writeln!(tgt, "weighted score\thourly\t\tdaily\t\tmonthly")
                .map_err(FreqleError::IOError)?;
            for (
                k,
                TVec3 {
                    hourly,
                    daily,
                    monthly,
                },
                score,
            ) in entries
            {
                writeln!(
                    tgt,
                    "{score:12.6}\t{hourly:10.6}\t{daily:10.6}\t{monthly:10.6}\t{k}"
                )
                .map_err(FreqleError::IOError)?;
            }
        } else {
            for (k, _, _) in entries {
                writeln!(tgt, "{}", k).map_err(FreqleError::IOError)?;
            }
        }
        Ok(())
    }

    fn delete(&mut self, key: &str) {
        self.energies.remove(key);
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
