#![feature(integer_atomics)]
#![feature(exclusive_range_pattern)]

extern crate human_size;
extern crate ignore;
extern crate num_cpus;
#[macro_use]
extern crate clap;

use clap::{App, Arg};
use human_size::{Multiple, Size};
use ignore::WalkBuilder;
use ignore::WalkState::*;
use std::cmp;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

// save into hashmap only files which is greater than this size 
// as uses mostly doesnt care about small files in this utility
const MIN_FILE_SIZE:u64 = 1048576;

fn main() {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .author(crate_authors!())
        .arg(
            Arg::with_name("FOLDER")
                .help("Folder where to scan")
                .index(1),
        )
        .arg(
            Arg::with_name("number")
                .short("n")
                .long("number")
                .value_name("NUMBER")
                .help("Number of top results to return. Default: 10"),
        )
        .get_matches();

    let dir = matches.value_of("FOLDER").unwrap_or(".");
    let results_amount = matches.value_of("number").unwrap_or("10");
    let results_amount: usize = results_amount
        .to_string()
        .parse()
        .expect("number value expected");

    let acc = Arc::new(AtomicU64::new(0));
    let files = Arc::new(Mutex::new(HashMap::new()));

    println!("Scanning folder: {}", dir);

    let _ = WalkBuilder::new(dir)
        .ignore(false)
        .threads(num_cpus::get())
        .git_global(false)
        .git_ignore(false)
        .git_exclude(false)
        .parents(false)
        .hidden(false)
        .build_parallel()
        .run(|| {
            let acc_in = Arc::clone(&acc);
            let files_in = Arc::clone(&files);
            Box::new(move |result| {
                let pp = match result {
                    Ok(result) => result,
                    Err(_) => return Continue,
                };

                let p = pp.path();
                let d = p.to_path_buf().into_os_string().into_string().unwrap();
                if let Some(ref result_type) = pp.file_type() {
                    if result_type.is_file() && !result_type.is_symlink() {
                        match pp.metadata() {
                            Ok(m) => {
                                // add into verbose hashmap only if size more than 1Mb
                                if m.len() > MIN_FILE_SIZE {
                                    let mut data = files_in.lock().unwrap();
                                    data.insert(d, m.len());
                                }
                                acc_in.fetch_add(m.len(), Ordering::SeqCst);
                            }
                            Err(_) => return Continue,
                        };
                    };
                } else {
                    return Continue;
                }

                Continue
            })
        });

    let res = Arc::try_unwrap(acc).unwrap().into_inner();
    let all = Arc::try_unwrap(files).unwrap().into_inner().unwrap();

    let mut count_vec: Vec<_> = all.iter().collect();
    count_vec.sort_by(|a, b| b.1.cmp(a.1));
    for x in 0..cmp::min(results_amount, count_vec.len()) {
        println!(
            "{} {}",
            get_human_readable_name(*count_vec[x].1),
            count_vec[x].0
        );
    }
    if count_vec.len() > 0 {
        println!("-------------");
    }
    println!("Total size of folder:\n {}", get_human_readable_name(res));
}

fn get_human_readable_name(size_in_bytes: u64) -> Size {
    const KILO: u64 = 1024u64;
    const MEGA: u64 = 1048576u64;
    const GIGA: u64 = 1073741824u64;
    const MAX: u64 = <u64>::max_value();

    match size_in_bytes {
        0..KILO => Size::new(_round(size_in_bytes as f64), Multiple::Byte).unwrap(),
        KILO..MEGA => {
            Size::new(_divide_and_round(size_in_bytes, KILO), Multiple::Kilobyte).unwrap()
        }
        MEGA..GIGA => {
            Size::new(_divide_and_round(size_in_bytes, MEGA), Multiple::Megabyte).unwrap()
        }
        GIGA..MAX => Size::new(_divide_and_round(size_in_bytes, GIGA), Multiple::Gigabyte).unwrap(),
        _ => Size::new(_round(size_in_bytes as f64), Multiple::Byte).unwrap(),
    }
}

fn _divide_and_round(size: u64, divisor: u64) -> f64 {
    _round(size as f64 / divisor as f64)
}
fn _round(what: f64) -> f64 {
    (what * 100.0).round() / 100.0
}
