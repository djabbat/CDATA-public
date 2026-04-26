//! CDATA CLI (MCOA Counter #1) — simple trajectory output matching
//! the interface of telomere-sim, mito_ros-sim, etc.

use std::env;
use cell_dt_cli::{compute_damage, Tissue};

fn parse_args() -> (Tissue, f64, f64) {
    let mut tissue = Tissue::HSC;
    let mut days: f64 = 3650.0;
    let mut rate: f64 = 0.01;
    let args: Vec<String> = env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--tissue" => {
                tissue = match args[i+1].as_str() {
                    "HSC" => Tissue::HSC,
                    "Fibroblast" => Tissue::Fibroblast,
                    "Neuron" => Tissue::Neuron,
                    "Cardiomyocyte" => Tissue::Cardiomyocyte,
                    "Hepatocyte" => Tissue::Hepatocyte,
                    "IntestinalCrypt" => Tissue::IntestinalCrypt,
                    other => {
                        eprintln!("Unknown tissue: {}", other);
                        std::process::exit(2);
                    }
                };
                i += 2;
            }
            "--days" => { days = args[i+1].parse().expect("--days f64"); i += 2; }
            "--rate" => { rate = args[i+1].parse().expect("--rate f64"); i += 2; }
            flag => {
                eprintln!("Unknown flag: {}", flag);
                std::process::exit(2);
            }
        }
    }
    (tissue, days, rate)
}

fn main() {
    let (tissue, days, rate) = parse_args();
    let params = tissue.params();
    println!("t_days,n,d,tissue,counter");
    let mut n: f64 = 0.0;
    for day in 0..=days as u64 {
        let t = day as f64;
        n += rate;
        let d = compute_damage(&params, n, t, 0.0);
        println!("{},{},{:.8},{:?},1", t, n, d, tissue);
    }
}
