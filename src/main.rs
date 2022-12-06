//used to get command line arguments
use std::env;
use std::num::IntErrorKind;
//used to exit gracefully
use std::process;
//used for all the filesystem stuff
use std::fs;
use std::path::Path;
use std::thread::current;

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
                compress_file(file);
            }
            //everything that isn't an argument
            _ => graceful_exit("Error: Unknown argument, see 'lz77 help' for details."),
        }
    } else {
        graceful_exit("Error: Please specify an argument, see 'lz77 help' for details.");
    }
    //make sure a file was specified, and that that file exists
    if args.len() == 1 {
        graceful_exit("Error: No file specified, please specify a file to compress");
    }
}

//clean up and stop the program without panicking
fn graceful_exit(err: &str) {
    eprintln!("{}", err);
    process::exit(0);
}

fn compress_file(file: &Path) {
    //get the contents of the file as a vector of bytes
    let file_bytes: Vec<u8> = fs::read(file).unwrap();
    //print out file_bytes as a debug(?) statement
    println!("{:?}", file_bytes);
    //the resulting compressed vector
    //first item in tuple is the offset
    //second item in the tuple is the length of the match
    //third item is the byte that's directly after the match
    let mut resulting_file_vec: Vec<(usize, usize, u8)> = vec![];
    //keep track of the current position
    let mut current_pos: usize = 0;

    //run until the end of the file
    while current_pos < file_bytes.len() - 1{
        //(0, 0, *) is the same as "no match found, here's the next byte"
        let mut current_calculated_tuple: (usize, usize, u8) = (0, 0, file_bytes[current_pos]);

        //special handling for the first byte because there's nothing before it to compare against
        if current_pos == 0 {
            println!("Pushing first tuple: {:?}", current_calculated_tuple);
            resulting_file_vec.push(current_calculated_tuple);
            current_pos += 1;
        }
        //keep track of the current index byte being compared
        let mut index_of_byte_to_compare: usize = current_pos - 1;
        // compare bytes before the current pos to the current pos
        'sub_matches: while index_of_byte_to_compare > 0 {
            //see if we found a matching character
            if file_bytes[index_of_byte_to_compare] == file_bytes[current_pos] {
                
                println!("Found matches at {} and {}", index_of_byte_to_compare, current_pos);
                //set the offset of the found match
                current_calculated_tuple.0 = current_pos - index_of_byte_to_compare;
                println!("offset value of {} equals current pos value of {}", current_pos - current_calculated_tuple.0, current_pos);
                //calculate and set the length of the match
                let mut match_length: usize = 1;
                // if the bytes following all equal each other and we don't accidentally run off of the edge of the vector, continue
                //if current_pos + match_length < file_bytes.len() - 1  {
                    while current_pos + match_length < file_bytes.len() - 1 && file_bytes[current_pos + match_length] == file_bytes[index_of_byte_to_compare + match_length] {
                        println!("Found preceding matches at {} and {}", index_of_byte_to_compare + match_length, current_pos + match_length);
                         match_length += 1;
                    
                    }
                //}
                current_pos += match_length;
                current_calculated_tuple.1 = match_length;
                current_calculated_tuple.2 = file_bytes[current_pos];
                println!("Found preceding matches at index: {}, pushing tuple: {:?}", current_pos, current_calculated_tuple);
                resulting_file_vec.push(current_calculated_tuple);
                break 'sub_matches;
                
            } else {
            //prevent it from wrapping off of the edge of the cliff
            if index_of_byte_to_compare > 0 {

            index_of_byte_to_compare -= 1;
            }
            current_calculated_tuple.2 = file_bytes[current_pos];
            println!("No preceding matches found at index: {}, pushing tuple: {:?}", current_pos, current_calculated_tuple);
            resulting_file_vec.push(current_calculated_tuple);
            
            }
            
        }
        current_pos += 1;
        //FUNCTIONAL END
    }
    println!("{:?}", resulting_file_vec);
    //get current byte in file_bytes
    //compare it to every byte before it in file_bytes
    //if equal, get the difference between the current position of the cursor and the position of the match
    //compare the byte after the cursor to the byte after the match until a match is not found or the index of the byte being checked and the current position are the same
}
