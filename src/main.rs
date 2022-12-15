//stealing features from the unstable lib

//used to get command line arguments
use std::env;

//used to exit gracefully and better error handling
use std::error::Error;
use std::process;
//used for all the filesystem stuff
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::{BufWriter, Cursor, SeekFrom};
use std::path::Path;

//fixing cross compatability isssues and finer FS control
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
//speedy
use rayon::prelude::*;
use std::thread;
use crossbeam_utils::thread as crossbeam_thread;
//used for debug printing and making everything pretty
//not necessary for any actual work
use colored::Colorize;
// this is all for the progress meter which sucks back precious time
// use crossterm::terminal::{Clear, ClearType};
// use crossterm::execute;
// use crossterm::queue;
// use std::io::stdout;
fn main() {
    //list of all the command line arguments
    let args: Vec<String> = env::args().collect();

    //handle the different arguments
    if args.len() > 1 {
        //rust equivalent of a switch statement
        match args[1].as_str() {
            "help" => {
                println!("lz77 syntax:\n    lz77 help\nDisplay this message.\n    lz77 compress [file] [resulting_file]\nCompress [file], and write [resulting_file], defaults to out.compressed\nExample: 'lz77 compress cat.png cat.compressed'\n    lz77 decompress [file] [resulting_file]\nDecompress [file] and write [resulting_file]\nExample: 'lz77 decompress cat.compressed cat.png'");
            }
            "compress" => {
                //make sure they specified a file to compress
                if args.len() == 2 {
                    graceful_exit("Error: Expected 2-3 arguments, got 1. Syntax: lz77 compress [file] [resulting file]")
                }
                //the file to be compressed as a file path
                let file: &Path = Path::new(&args[2]);
                //make sure the file exists
                if !file.exists() {
                    graceful_exit("Error: File not found.");
                }
                let out_name: Option<&Path> = if args.len() > 3 {
                    Some(&Path::new(&args[3]))
                } else {
                    None
                };
                compress_file(file, out_name).expect("Generic Error: Unable to compress file");
            }
            "decompress" => {
                //make sure they specified a file to compress
                if args.len() == 2 {
                    graceful_exit("Error: Expected 2-3 arguments, got 1. Syntax: lz77 decompress [file] [resulting file]")
                }
                //the file to be compressed as a file path
                let file: &Path = Path::new(&args[2]);
                //make sure the file exists
                if !file.exists() {
                    graceful_exit("Error: File not found.");
                }
                let out_name: Option<&Path> = if args.len() > 3 {
                    Some(&Path::new(&args[3]))
                } else {
                    None
                };
                decompress_file(file, out_name).expect("Generic Error: Unable to decompress file");
            }
            //everything that isn't an argument
            _ => graceful_exit("Error: Unknown argument, see 'lz77 help' for details."),
        }
    } else {
        graceful_exit("Error: Please specify an argument, see 'lz77 help' for details.");
    }
}

//clean up and stop the program without panicking
fn graceful_exit(err: &str) {
    eprintln!("{}", err);
    process::exit(0);
}

fn compress_file(
    file: &Path,
    file_out_name: Option<&Path>,
) -> Result<(), Box<dyn Error + 'static>> {
    let file_bytes = read_u16_vec_from_file(file).unwrap();

    //make this true to print out a detailed walkthrough of the compression steps
    //definitely don't run on larger files
    let debug_out: bool = false;
    if debug_out {
        //print out file_bytes as a debug(?) statement
        println!("Original file as bytes: {:?}", file_bytes);
    }

    //the resulting compressed vec of tuples
    //first item in tuple is the offset
    //second item in the tuple is the length of the match
    //third item is the byte that's directly after the match
    let mut resulting_file_vec: Vec<(usize, usize, u16)> = vec![];
    //keep track of the current position
    let mut current_pos: usize = 0;
    //size of sliding window(how far back to check for a match) in bytes
    let sliding_window = 4000;

    println!("Started Compression.");
    //run until the end of the file
    while current_pos < file_bytes.len() - 1 {
        //this looks nice but has an absolutely honking performance impact
        // queue!(stdout(), Clear(ClearType::CurrentLine));
        // print!("\r{}% complete", (current_pos * 100 / file_bytes.len()));
        // stdout().flush()?;
        //(0, 0, *) is the same as "no match found, here's the next byte"
        let mut current_calculated_tuple: (usize, usize, u16) = (0, 0, file_bytes[current_pos]);

        //special handling for the first byte because there's nothing before it to compare against

        //keep track of the current index byte being compared
        //it's probably far more storage efficient to work backwards from the current_pos

        //if the sliding window is less than the beginning of the file then start at the beginning of the file
        let mut index_of_byte_to_compare: usize = if current_pos <= sliding_window {
            0
        } else {
            current_pos - sliding_window
        };

        'sub_matches: while index_of_byte_to_compare < current_pos {
            if debug_out {
                //see if we found a matching character
                println!(
                    "Comparing {} with current position {}",
                    format!("{}", index_of_byte_to_compare).green(),
                    format!("{}", current_pos).green()
                );
            }
            if file_bytes[index_of_byte_to_compare] == file_bytes[current_pos] {
                if debug_out {
                    println!(
                        "Found matches at {} and {}",
                        format!("{}", index_of_byte_to_compare).green(),
                        format!("{}", current_pos).green()
                    );
                }
                //set the offset of the found match
                current_calculated_tuple.0 = current_pos - index_of_byte_to_compare;
                //calculate and set the length of the match
                let mut match_length: usize = 1;
                // if the bytes following all equal each other and we don't accidentally run off of the edge of the vector, continue
                while index_of_byte_to_compare + match_length < current_pos
                    && file_bytes[current_pos + match_length]
                        == file_bytes[index_of_byte_to_compare + match_length]
                {
                    if debug_out {
                        println!(
                            "Found preceding matches at {} and {}, current match length: {}",
                            format!("{}", index_of_byte_to_compare + match_length).green(),
                            format!("{}", current_pos + match_length).green(),
                            format!("{}", match_length).green()
                        );
                    }
                    match_length += 1;
                }

                current_pos += match_length;

                current_calculated_tuple.1 = match_length;
                current_calculated_tuple.2 = file_bytes[current_pos];
                if debug_out {
                    println!(
                        "No more preceding matches found at indexes {} and {} or hit max match len ({})",
                        format!("{}", index_of_byte_to_compare + match_length).green(), format!("{}", current_pos).green(), format!("{}", current_pos)
                    );
                }
                break 'sub_matches;
            } else {
                if debug_out {
                    println!(
                        "The byte at {} does not equal the byte at {}, incrementing",
                        format!("{}", index_of_byte_to_compare,).green(),
                        format!("{}", current_pos)
                    );
                }
                index_of_byte_to_compare += 1;
            }
        }
        if debug_out {
            println!(
                "No more prior comparisons to make, pushing tuple: {}",
                format!("{:?}", current_calculated_tuple).green()
            );
        }
        resulting_file_vec.push(current_calculated_tuple);

        current_pos += 1;
        //FUNCTIONAL \END
    }
    println!("\nFinished compression.");
    if debug_out {
        println!("Resulting encoded tuple: {:?}", resulting_file_vec);
    }
    //file storage is really nasty, you could probably score big boi points here by implementing this better

    //trying to do it efficiently
    //hahahaha this is laughably terrible
    //it spits out files 2-3 times the size of the original file
    let mut hex_values_vec: Vec<u16> = Vec::new();

    for i in resulting_file_vec {
        hex_values_vec.push(i.0.try_into()?);
        hex_values_vec.push(i.1.try_into()?);
        hex_values_vec.push(i.2);
    }

    //abuse deref coersion to convert the vec to a slice
    let resulting_file_buf: &[u16] = &hex_values_vec;
    if debug_out {
        println!("Resulting encoded file: {:#X?}", resulting_file_buf);
    }
    // get resulting file name or set default
    let resulting_file_name = match file_out_name {
        Some(name) => name,
        None => {
            //default filename defined here
            Path::new("out.compressed")
        }
    };
    //if file exists, overwrite
    // fs::try_exists(resulting_file_name);
    if Path::exists(resulting_file_name) {
        println!("Found file with same name, overwriting.");
        fs::remove_file(resulting_file_name)?;
    }
    // write the file, one byte at a time
    let mut resulting_file =
        BufWriter::new(File::create(resulting_file_name).expect("Unable to create file"));
    for &n in resulting_file_buf {
        //u16
        resulting_file.write_u16::<BigEndian>(n)?;
        //u8 tests
        //resulting_file.write_u8(n).unwrap();
    }
    println!("File writing complete, exiting.");
    Ok(())
}

fn decompress_file(file: &Path, resulting_file_name: Option<&Path>) -> Result<(), Box<dyn Error>> {
    //Read the file into a tuple of u16s
    let mut file_bytes: Vec<u16> = read_u16_vec_from_file(file)?;
    //more speed gains to be made here
    let tuple_vec: Vec<Vec<u16>> = par_drain_to_tuple(&mut file_bytes)?;

    //https://docs.rs/rayon/latest/rayon/fn.join.html

   pub fn par_drain_to_tuple(v: &mut Vec<u16>) -> Result<Vec<Vec<u16>>, Box<dyn Error>> {
        let mut return_vec: Vec<Vec<u16>> = Vec::with_capacity(v.len() / 3);
        let v_len = v.len();
        // let (lo, hi)  = v.split_at_mut(v_len / 2);
        //println!("v_len: {}", v_len);
        //println!("v_len / 4 = {}", (v_len - 1) / 4);
        let num_of_threads = largest_factor_under_val(v_len / 3, 100);
        println!("Using {} threads", num_of_threads);
        let mut v_chunks =  v.chunks_exact(((v_len) / num_of_threads)).collect::<Vec<&[u16]>>();
       // println!("last chunk: {:#x?}", v_chunks);
        // if there's an uneven number of chunks, append to decompression
        //println!("v_chunks len: {}", v_chunks.len());
        // for i in v_chunks {
        //     thread::spawn(move || drain_to_tuple(Vec::from(i)));
        // }
        //innermost vec should only be exactly 3, I would prefer a slice but don't want to spend time coercing types
        //middle vec is the parsed chunk

        let mut parsed_chunks: Vec<Vec<Vec<u16>>> = vec![Vec::from(Vec::with_capacity(3)); v_chunks.len()]; 
            parsed_chunks = crossbeam_thread::scope (|s| {
                let mut parsed_chunks = parsed_chunks.clone();
                let mut handles: Vec<(usize, crossbeam_utils::thread::ScopedJoinHandle<std::vec::Vec<std::vec::Vec<u16>>>)> = Vec::new();
                for (chunk_num, chunk) in v_chunks.iter().enumerate() {
                    println!("Spawned thread to deal with chunk {} ", chunk_num);
                    let handle = s.spawn(move |_| {
                        drain_to_tuple(Vec::from(*chunk))
                    });
                    handles.push((chunk_num, handle));
                }
                // for chunk_num in  0..parsed_chunks.len() {
                //   //  handles.set_position(chunk_num.try_into().unwrap());
                //     parsed_chunks[chunk_num] = handles[chunk_num].join().unwrap();
                // }
                for handle in handles {
                    parsed_chunks[handle.0] = handle.1.join().unwrap();

                }
                
                parsed_chunks
            }).unwrap();
            for i in  parsed_chunks{
                return_vec.extend(i);
            }
        
        fn drain_to_tuple(v: Vec<u16>) -> Vec<Vec<u16>>{
            let mut v = v.clone();
            let mut new_tuple: Vec<Vec<u16>> = Vec::with_capacity(v.len()/3);
            while v.len() > 0 {
                println!("{}", v.len());
                new_tuple.push(v.drain(0..3).collect::<Vec<u16>>());
            }
            new_tuple

        }
    
        fn largest_factor_under_val(val: usize, limit: usize) -> usize{
            //oneliner to solve for factors
                let mut vec_of_factors = (1..val + 1).into_iter().filter(|&x| val % x == 0).collect::<Vec<usize>>();
                //remove everything over the thread limit
                vec_of_factors.retain(|item| { 
                let remove_over_limit = {
                    if item > &limit {
                        false
                    } else {
                        true
                    }

                };
                remove_over_limit
            });
            vec_of_factors.sort();
            *vec_of_factors.last().clone().unwrap()

        }
        // let pieces = rayon::join(|| drain_to_tuple(&mut Vec::from(lo)), || drain_to_tuple(&mut Vec::from(hi)));
        // return_vec.extend(pieces.0);
        // return_vec.extend(pieces.1);
        Ok(return_vec)
    }


    //let rayon_iter = file_bytes.par_iter();
    // rayon_iter.drain(0..3).collect::<Vec<u16>>();
    // while file_bytes.len() > 0 {
    //     println!("{}", file_bytes.len());

    //     tuple_vec.push(file_bytes.drain(0..3).collect::<Vec<u16>>());
    // }

    let mut resulting_bytes: Vec<u16> = Vec::new();
    //position in the compressed data vec of tuples
    //let mut current_pos = 0;
    for current_pos in 0..tuple_vec.len() {
        //offset from the the last uncompressed byte is tuple_vec[current_pos].0
        //match length is 1
        //next value is 2
        let index_of_offset: usize =
            resulting_bytes.len() - usize::from(tuple_vec[current_pos][0]);
        resulting_bytes.extend_from_within(
            index_of_offset..(index_of_offset + usize::from(tuple_vec[current_pos][1])),
        );
        resulting_bytes.push(tuple_vec[current_pos][2]);
       // println!("{} of {}", current_pos, tuple_vec.len());
    }
    println!("Finished decompressing, writing to file.");
    let resulting_file = match resulting_file_name {
        Some(name) => name,
        None => Path::new("out.decompressed"),
    };
    //if file exists, overwrite
    if Path::exists(resulting_file) {
        println!("Found file with same name, overwriting.");
        fs::remove_file(resulting_file)?;
    }
    // write the file, one byte at a time
    let mut resulting_file =
        BufWriter::new(File::create(resulting_file).expect("Unable to create file"));
    for n in resulting_bytes {
        //u16
        resulting_file.write_u16::<BigEndian>(n)?;
        //u8 tests
        //resulting_file.write_u8(n).unwrap();
    }
    println!("File writing complete, exiting.");
    Ok(())
}

pub fn read_u16_vec_from_file(file: &Path) -> Result<Vec<u16>, Box<dyn Error>> {
    // by reading as a u16 we trade one byte matches for better speed and use of our file format
    //this whole block is painstakingly crafted to be as efficient as possible
    //I could not make this again if I tried
    let mut file_bytes: Vec<u16> = Vec::new();
    let mut read_handle = Cursor::new(fs::read(file)?);

    while read_handle.position() < read_handle.get_ref().len().try_into()? {
        //big endian is used here because we read from the start of the file, and the advantages of little endian are not applicable here
        match read_handle.read_u16::<BigEndian>() {
            Ok(bytes) => {
                file_bytes.push(bytes);
                //println!("{:#x}", bytes)
            }
            Err(err) => {
                println!("Encountered uneven bytes: {}", err);
                //assume we've reached the end of the file and so we put a 0 bit
                //will probably break something
                //file footer should be added at some point
                //see this wizardry for context
                //https://stackoverflow.com/questions/50243866/how-do-i-convert-two-u8-primitives-into-a-u16-primitive
                let last_byte = read_handle.get_ref().last().unwrap();
                let byte_to_push = ((*last_byte as u16) << 8) | 0 as u16;
                file_bytes.push(byte_to_push);
                read_handle.seek(SeekFrom::Current(1))?;
            }
        }
    }
    Ok(file_bytes)
}
