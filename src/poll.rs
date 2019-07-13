use rand_core::RngCore;
use std::{
    thread,
    time::{Duration, Instant},
};

pub fn at_interval(
    memory: &mut [u8],
    page_size: usize,
    interval: u64,
    rng: &mut impl RngCore,
) -> ! {
    loop {
        access_every_page(memory, page_size, next_usize(rng) % page_size);
        if interval > 0 {
            thread::sleep(Duration::from_millis(interval));
        }
    }
}

// Read and write one byte in each memory page in the given slice of memory.
fn access_every_page(memory: &mut [u8], page_size: usize, page_offset: usize) {
    assert!(page_offset < page_size);
    let start = Instant::now();
    for page in memory.chunks_exact_mut(page_size) {
        // Just an operation that forces a read and write into this memory page.
        page[page_offset % page.len()] ^= page_offset as u8;
    }
    eprintln!(
        "Wrote a random byte to each page in {} ms",
        start.elapsed().as_millis()
    );
}

/// A random `usize`. This function exists so I don't have to generate a 64 bit number and throw
/// half away on a 32 bit system. So mostly premature optimization I guess ¯\_(ツ)_/¯.
/// But would be nice if `RngCore::next_usize` existed.
fn next_usize(rng: &mut impl RngCore) -> usize {
    if cfg!(target_pointer_width = "32") {
        rng.next_u32() as usize
    } else {
        rng.next_u64() as usize
    }
}
