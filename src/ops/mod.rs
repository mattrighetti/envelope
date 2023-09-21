mod add;
mod print;
mod delete;

pub use add::*;
pub use print::*;
pub use delete::*;

use std::path::Path;
use std::io::{self, BufRead};
use std::fs::File;

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
