use libc;
use rax::Rax;
use std;

const DEFAULT_CHUNK_SIZE: u64 = 1024 * 1024 * 8;
const DEFAULT_CHUNKS_PER_FILE: u64 = 8;

pub struct Config {
    /// Size of chunk
    pub chunk_size: u64,
    /// Number of chunks per physical file-system file
    pub chunks_per_file: u64,
    /// Size of I/O buffer to read
    pub page_size: u32,
    /// Max amount of memory to keep in the head list
    pub head_memory_limit: u64,
}

pub enum Jobs {
    Dedupe
}

/// Active job structure
pub struct Job {
    nack: usize,

    // Support upto 48 byte dedupe keys
    dup_len: u8,
    dup: [u8; 48],
}

// Mimic C version
#[repr(C)]
pub struct StreamID {
    pub ms: libc::uint64_t,
    pub seq: libc::uint64_t,
}

pub enum ChunkState {
    Idle,
    Loading,
    Error,
}

pub struct File {
    pub first: StreamID,
    pub last: StreamID,
    //
    pub list: Vec<Chunk>,
}

pub struct Chunk {
    pub first: StreamID,
    pub last: StreamID,
    pub state: ChunkState,

    ///
    pub list: Box<Rax<Job>>,
}

pub enum Task {
    Load(String)
}

pub struct JobStream {
    flags: u32,
    /// Chunk of jobs to be consumed
    head: Box<Chunk>,
    /// Pending Chunks
    queue: std::collections::VecDeque<Box<Chunk>>,
    /// Chunk
    tail: Box<Chunk>,
    /// Radix tree of all chunks.
    chunks: Box<Rax<Chunk>>,
    /// Current configuration to control the behavior and memory consumption.
    config: Config,
    /// Dequeue of tasks.
    tasks: std::collections::VecDeque<Task>,
}

impl JobStream {
    fn tick(&mut self) {
        // Perform cleanup and timeouts
    }
}