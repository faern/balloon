use number_prefix::{NumberPrefix, Prefixed, Standalone};
use rand_core::{RngCore, SeedableRng};
use rand_xorshift::XorShiftRng;
use std::{
    alloc::{alloc, Layout},
    convert::TryFrom,
    process, thread,
    time::{Duration, Instant},
};
use structopt::StructOpt;

mod cli;
mod mlock;
mod poll;

fn main() {
    let opt = cli::Opt::from_args();

    let page_size = opt.page_size.unwrap_or_else(page_size);
    let num_pages = opt.size / page_size;

    eprintln!(
        "Page size: {} ({} bytes)",
        prefixed_bytes(page_size),
        page_size
    );
    eprintln!(
        "Allocating {} ({} bytes, {} pages)",
        prefixed_bytes(opt.size),
        opt.size,
        num_pages,
    );

    let alloc_start = Instant::now();
    let layout = Layout::from_size_align(opt.size, page_size).expect("Invalid memory layout");
    let data_slice: &mut [u8] = unsafe { std::slice::from_raw_parts_mut(alloc(layout), opt.size) };
    eprintln!(
        "Allocated memory region {:p}..{:p} in {} ms",
        data_slice.as_ptr(),
        (data_slice.as_ptr() as usize + opt.size) as *const u8,
        alloc_start.elapsed().as_millis()
    );

    if !opt.no_mlock {
        let mlock_start = Instant::now();
        if let Err(e) = mlock::mlock(data_slice) {
            eprintln!("Unable to mlock memory: {}", e);
            process::exit(1);
        }
        eprintln!(
            "Locked memory region in main memory in {} ms",
            mlock_start.elapsed().as_millis()
        );
    } else {
        println!("Skipping locking the allocated memory pages to main memory");
    }

    // We could just depend on the larger `rand` crate and get good seeding etc for free.
    // But the randomness here is not very important. The only important part is that we fill
    // the memory with data that is not easily compressible.
    let seed = alloc_start.elapsed().as_nanos() as u64;
    let mut rng = <XorShiftRng as SeedableRng>::seed_from_u64(seed);

    if !opt.no_fill {
        let fill_start = Instant::now();
        rng.fill_bytes(data_slice);
        eprintln!(
            "Filled memory with random data in {} ms",
            fill_start.elapsed().as_millis()
        );
    } else {
        println!("Skipping filling the allocated memory with random data")
    }

    if let Some(poll_interval) = opt.poll_interval {
        // Forever read and write to random bytes in the memory. To prevent the OS from moving it
        // to swap.
        poll::at_interval(data_slice, page_size, poll_interval, &mut rng);
    } else {
        println!("Skipping polling the memory. Will now sleep forever");
        loop {
            thread::sleep(Duration::from_secs(60));
        }
    }
}

/// Format an integer representing bytes into a strix with proper prefix.
fn prefixed_bytes(bytes: usize) -> String {
    match NumberPrefix::binary(bytes as f64) {
        Standalone(bytes) => {
            if (bytes - 1.0).abs() < std::f64::EPSILON {
                "1 byte".to_owned()
            } else {
                format!("{} bytes", bytes)
            }
        }
        Prefixed(prefix, n) => format!("{:.0} {}B", n, prefix),
    }
}

/// Query the system for the memory page size.
fn page_size() -> usize {
    #[cfg(unix)]
    {
        let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
        usize::try_from(page_size).expect("Page size larger than usize")
    }
    #[cfg(not(unix))]
    {
        4096
    }
}
