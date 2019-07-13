use std::{fmt, num::ParseIntError, str::FromStr};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "balloon",
    about = "Allocates a chunk of memory and tries to prevent it from being swapped.",
    rename_all = "kebab-case"
)]
pub struct Opt {
    /// How many bytes of memory to allocate.
    #[structopt(parse(try_from_str = "parse_size"))]
    pub size: usize,

    #[structopt(long, parse(try_from_str = "parse_page_size"))]
    /// The page size to use. Controls the alignment of the allocated memory, as well as the offset
    /// between each read+write in the poll loop activated with the --poll-interval option.
    /// Is by default read from the OS if possible, otherwise set to 4096.
    /// Must be non-zero and a power of two.
    pub page_size: Option<usize>,

    #[structopt(long)]
    /// Skips locking the memory region to main memory via the `mlock` syscall.
    pub no_mlock: bool,

    #[structopt(long)]
    /// Skips filling the allocated memory with random data after allocating it. Note that if
    /// this flag is given together with --no-mlock, or your system does not support mlock, then
    /// it is likely the ballooning will have no effect. The kernel might not dedicate any memory
    /// to this process.
    pub no_fill: bool,

    #[structopt(long)]
    /// If given, this activates a memory poll loop. This will iterate through all allocated pages
    /// and read+write one random byte to each, and then sleep for the given number of ms before
    /// starting over again. This polling is disabled by default.
    pub poll_interval: Option<u64>,
}

fn parse_size(arg: &str) -> Result<usize, ParseSizeError> {
    match arg.find(|c: char| !c.is_digit(10)) {
        Some(split_i) => {
            let (size_str, prefix) = arg.split_at(split_i);
            let multiplier = match prefix {
                "k" => 1024usize,
                "M" => 1024usize.pow(2),
                "G" => 1024usize.pow(3),
                "T" => 1024usize.pow(4),
                x => return Err(ParseSizeError::InvalidPrefix(x.to_owned())),
            };
            let size = usize::from_str(size_str).map_err(ParseSizeError::ParseIntError)?;
            Ok(size * multiplier)
        }
        None => usize::from_str(arg).map_err(ParseSizeError::ParseIntError),
    }
}

#[derive(Debug)]
enum ParseSizeError {
    ParseIntError(ParseIntError),
    InvalidPrefix(String),
}

impl fmt::Display for ParseSizeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseSizeError::ParseIntError(inner) => inner.fmt(f),
            ParseSizeError::InvalidPrefix(prefix) => write!(f, "Invalid size prefix '{}'", prefix),
        }
    }
}

impl std::error::Error for ParseSizeError {}

fn parse_page_size(arg: &str) -> Result<usize, ParsePageSizeError> {
    let size = usize::from_str(arg).map_err(ParsePageSizeError::ParseIntError)?;
    if !size.is_power_of_two() {
        Err(ParsePageSizeError::NotPowerOfTwo)
    } else {
        Ok(size)
    }
}

#[derive(Debug)]
enum ParsePageSizeError {
    ParseIntError(ParseIntError),
    NotPowerOfTwo,
}

impl fmt::Display for ParsePageSizeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ParsePageSizeError::*;
        match self {
            ParseIntError(inner) => inner.fmt(f),
            NotPowerOfTwo => write!(f, "Page size must be a power of two"),
        }
    }
}

impl std::error::Error for ParsePageSizeError {}
