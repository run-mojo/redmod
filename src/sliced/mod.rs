use rax::*;
use std;
use sds::SDS;
//use std::mem::size_of;

use stream::*;

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
    pub node_max_bytes: u32,
    pub node_max_entries: u16,
}

/// Active job structure
#[repr(packed)]
pub struct Job {
    nack: usize,

    // Support upto 48 byte dedupe keys
    dup_len: u8,
}

impl Drop for Job {
    fn drop(&mut self) {
        if self.dup_len > 0 {}
    }
}

pub enum ChunkState {
    Idle,
    Loading,
    Error,
}

pub struct JobsFile {
    pub first: StreamID,
    pub last: StreamID,
    //
    pub list: Vec<JobsChunk>,
}

///
#[repr(packed)]
pub struct JobsChunk {
    pub first: StreamID,
    pub last: StreamID,
    ///
    pub state: ChunkState,

    ///
    pub list: Box<Rax<StreamID, Job>>,
}

pub enum Task {
    Load(String)
}

pub struct ConsumerGroup;

pub struct Consumer;

/// This is the core Redis Data Type.
pub struct JobStream {
    flags: u32,
    stream: *mut stream,
    /// Chunk of jobs to be consumed
    head: Box<JobsChunk>,
    /// Pending Chunks
    queue: std::collections::VecDeque<Box<JobsChunk>>,
    /// Chunk
    tail: Box<JobsChunk>,
    /// Radix tree of all chunks.
    chunks: Box<Rax<StreamID, JobsChunk>>,
    /// De-duplication RAX
    dup: Option<Box<Rax<SDS, Job>>>,
    /// Current configuration to control the behavior and memory consumption.
    config: Config,
    /// Dequeue of tasks.
    tasks: std::collections::VecDeque<Task>,
}

impl JobStream {
    fn tick(&mut self) {
        // Perform cleanup and timeouts
        // Iterate all consumer groups
        // Find min ID - First entry on group PEL
        // Fax max ID -
    }

    fn add(&mut self) {

    }

    fn add_to_dedupe(&mut self) {

    }

    fn remove_job(&mut self) {}
}

#[cfg(test)]
mod tests {
    use std;
    use sliced::*;

    #[test]
    fn test_packing() {
        println!("sizeof<Job> =         {}", std::mem::size_of::<Job>());
//        println!("sizeof<StreamID> =    {}", std::mem::size_of::<StreamID>());
    }
}