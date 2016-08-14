extern crate regex;
extern crate glob;

use std::error::Error;
use std::io::prelude::*;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::env;

use regex::Regex;
use glob::glob;

fn process_chunk(path: std::path::PathBuf, mut potfile: &File, re: &Regex) -> u32 {
    let re_chunknum = Regex::new(r"chunk(\d+)/clust.gout").unwrap();
    let cap = re_chunknum.captures(&path.to_str().unwrap()).expect("Malformed path.");
    println!("Processing chunk {}", cap.at(1).unwrap());

    let clust = match File::open(&path) {
         Err(why) => panic!("couldn't open current chunks clust.gout: {}", why.description()),
         Ok(file) => file,
    };

    let mut line_count: u32 = 0;

    //Use a buffer because these files are upwards of 20GB sometimes.
    let reader = BufReader::new(clust);
    for line in reader.lines() {
        let test = line.unwrap();
        for cap in re.captures_iter(&test) {
            let potval: f64 = cap.at(1).unwrap_or("").parse::<f64>().unwrap()*239.2311f64;
            let potout = format!("{:.6}\n", potval);
            line_count = line_count + 1u32;
            match potfile.write_all(potout.as_bytes()) {
                Err(why) => panic!("couldn't write to output: {}", why.description()),
                Ok(_) => {},
            }
        }
    }
    line_count
}

fn get_expected_total() -> u32 {
    //Here we're looking for how many lines we're expecting in the output. This should match our input file.
    let input = match File::open("chunk01/input.gin") {
         Err(why) => panic!("Cannot open chunk01/input.gin: {}", why.description()),
         Ok(file) => file,
    };
    let mut line_count: u32 = 0;
    let reader = BufReader::new(input);
    for line in reader.lines() {
        let test = line.unwrap();
        let found = regex::is_match(r"cart", &test).expect("Malformed regex."); //the GULP command 'cart' is stated once per input.
        if found {
            line_count = line_count + 1u32;
        }
    }
    line_count
}

fn main() {

    //By default we generate values for node 1, although we can use a CLA to build other nodes (ultimately we need 1 - 30).
    let mut node = 1;
    if let Some(arg1) = env::args().nth(1) {
        node = arg1.parse().unwrap();
    }

    println!("Consolidating potential file for node {}...", node);

    //Check for completed files.
    let job_count = glob("chunk*/clust.gout").unwrap().count();
    let dir_count = glob("chunk*").unwrap().count();
    if dir_count != job_count {
        panic!("Found {} chunk directories and {} cluster files. Check all jobs are complete.", dir_count, job_count);
    }

    //setup output file
    let potname = format!("potential_{}.dat", node);
    let potpath = Path::new(&potname);
    let potfile = match File::create(&potpath) {
         Err(why) => panic!("couldn't create {}: {}", potpath.display(), why.description()),
         Ok(file) => file,
    };

    let re_final = Regex::new(r"Final energy =\s+(-?\d+\.?\d+)\s+eV").unwrap(); //this is called in a loop. You don't want to have it compile multiple times: https://github.com/rust-lang-nursery/regex
    let mut line_counts: Vec<u32> = Vec::new();
    //Run our process on all chunk files (in order).
    for chunk in glob("chunk*/clust.gout").expect("Failed to read gout files.") {
        match chunk {
            Err(why) => panic!("Could not process chunk: {:?}", why),
            Ok(path) => line_counts.push(process_chunk(path, &potfile, &re_final)),
        }
    }

    //Make sure our potential file has the correct amount of lines in it.
    println!("Verifying Output...");
    let input_count = get_expected_total();
    let mut idx = 1;
    let mut complete = true;
    for count in line_counts {
        if count != input_count {
            println!("WARNING: Chunk {:02} has only processed {} positions of an expected {}.\n         Make sure job has finished correcly.", idx, count, input_count);
            complete = false;
        }
        idx = idx + 1;
    }

    if complete {
        println!("potential_{}.dat constructed seccesfully.", node);
    } else {
        println!("There were errors in the construction process. Please check your input.");
    }
}
