# Balloon - memory filling tool

Allocates a chunk of memory of a specified size and tries to make sure the
OS keeps it in main memory, and not the swap.

Useful for testing and debugging OOM/low memory situations as well as performing certain benchmarks.

The name is inspired from the [memory ballooning] technique commonly used with virtual machines.

[memory ballooning]: https://www.techopedia.com/definition/30466/memory-ballooning

## How it operates

The tool is started with an argument telling it how much memory it should fill up, and optionally
some flags controlling certain aspects of functionality that will be described here. When the
program starts it will do the following:

1. Allocate the amount of memory specified, in one continuous chunk. The chunk will be
   aligned to the memory page size the operating system uses, or optionally to the value set with
   `--page-size`.
2. If the `--no-lock` flag is absent, and the program runs on an OS where it has this feature
   implemented, it will lock the allocated memory region to the main memory, prohibiting the OS
   from moving it to the swap. This is implemented with the `mlock` call on unix based platforms
   and `VirtualLock` on Windows.
3. If the `--no-fill` flag is absent, the allocated memory will be filled with data generated from
   a fast PRNG. The PRNG is not cryptographically secure, but that is not important. We just want
   to access all memory pages and insert data that the OS can't easily compress. This exists so
   the OS can't just do some invisible optimization where it's not actually dedicating any memory
   to the process.
4. If the `--poll-interval <sleep in ms>` argument is passed the program will now enter an
   infinite loop where each iteration consists of reading+writing one random byte in each allocated
   memory page and then sleeping for `<sleep in ms>` before in starts over again.
   If `--poll-interval` is not present, the program will just sleep forever from here.

## Notes on memory locking

On Linux there are certain restrictions on who can lock memory with the `mlock` call and how much
they can lock. If running on Linux 2.6.8 or earlier, only root can lock memory. In newer kernels
unpriviledges users can lock memory, but the amount is limited by the RLIMIT_MEMLOCK resource limit.
This limit can be adjusted with the `ulimit -l <amount in kb>` command, but only root can do that,
and the limit is applied per session. So the easiest would be to run balloon as root, otherwise
try something like:

```bash
faern@machine:~ $ sudo -i
root@machine:~ $ ulimit -l 10485760 # 10 GiB in KiB
root@machine:~ $ sudo -i faern
faern@machine:~ $ balloon 10G
```

## Example scenario

Say you want to benchmark your filesystem and disks. So you run some benchmarking utility that
writes and then reads many/large files to and from disk. Chances are your OS cache all these
files in memory, since it had a lot of free memory laying around, and these files seem frequently
accessed. You end up not really hitting the disks or the filesystem that much at all, and all you
have benchmarked is essentially how well your OS does caching and how fast memory you have.

Solution: Fill up unused memory with this tool prior to starting the benchmarking utility.
Let's say you have 16 GB memory in your computer. You check your current memory usage and you are
currently using 2.2 GB of it (not including OS caches etc). There has to be some error margin, so
you run:
```bash
# In one terminal:
$ balloon 12G

# While the above runs, you do your benchmarking in another one:
$ some_io_bench_utility --dir /storage/bench/
```

Now your results are likely much lower, and more realistic. And the drive LEDs probably blink a lot
more.

Sidenote: You probably want to close as many other programs as possible before your benchmarks.
They might allocate and free memory while the benchmarks run, leaving memory space for your OS
to start doing caching in, or using too much forcing the os to start swap pages and degrading
performance. Or the other programs might be using the resource you are trying to benchmark,
therefore taking away precious bandwidth from that resource.

## Warning

This tool can easily crash your computer. Given that it tries hard to make sure the memory it
controls is neither compressible nor moved to swap it is very easy to make it exhaust all your
memory resources.

If you run `balloon 14G` on a machine with 16 GB memory and only 10 of those are free, it is very
possibly your computer will just shut down.

## Disclaimer

I, the author, don't have detailed knowledge on the memory management internals of most/any
operating systems. I would greatly appreciate feedback and suggestions on how to improve this tool.
