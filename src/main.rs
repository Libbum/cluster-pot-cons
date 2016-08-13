extern crate regex;
extern crate glob;

use std::error::Error;
use std::io::prelude::*;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use regex::Regex;
use glob::glob;

fn main() {
    println!("Consolidating Potential File");

    //get node of current job
    let mut node: u32 = 0;

    let re_node = Regex::new(r"pot(\d+)\.job").unwrap();
    let jobs = glob("*.job").unwrap().count();
    if jobs > 1 {
        panic!("Too many job files in directory.")
    } else if jobs == 0 {
        panic!("No job files found in directory.")
    }

    for entry in glob("*.job").unwrap().filter_map(Result::ok) {
        let cap = re_node.captures(entry.to_str().unwrap()).expect("Malformed job file.");
        node = cap.at(1).unwrap().parse::<u32>().unwrap();
    }

    println!("Node = {}", node);

    //setup output file
    let potpath = format!("potential_{}.dat", node);
    let path = Path::new(&potpath);
    let potdisplay = path.display();

    let mut potfile = match File::create(&path) {
         Err(why) => panic!("couldn't create {}: {}", potdisplay, why.description()),
         Ok(file) => file,
    };


    //load buffer and poulate output
    let clust = match File::open("clust.gout") {
         Err(why) => panic!("couldn't open clust.gout: {}", why.description()),
         Ok(file) => file,
    };

    let re = Regex::new(r"Final energy =\s+(-?\d+\.?\d+)\s+eV").unwrap();

    let reader = BufReader::new(clust);
    for line in reader.lines() {
        let test = line.unwrap();
        for cap in re.captures_iter(&test) {
            let potval: f64 = cap.at(1).unwrap_or("").parse::<f64>().unwrap()*239.2311_f64;
            let potout = format!("{:.6}\n", potval);
            match potfile.write_all(potout.as_bytes()) {
                Err(why) => panic!("couldn't write to {}: {}", potdisplay, why.description()),
                Ok(_) => {},
            }
        }
    }
}
