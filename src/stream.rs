#![allow(dead_code)]

use libc;
use listpack::*;
use rax::*;
use sds::*;
use std;
use std::default::Default;
use std::fmt;
use std::mem::size_of;
use std::ptr;

pub struct Stream {
    pub s: *mut stream,
}

const STREAM_ID: StreamID = StreamID { ms: 0, seq: 0 };
const STREAM_ID_REF: *const StreamID = &STREAM_ID as *const StreamID;

impl Stream {
    pub fn new() -> Stream {
        return Stream { s: unsafe { streamNew() } };
    }

    fn lookup_consumer_group(&self, groupname: Sds) -> *mut streamCG {
        unsafe { streamLookupCG(self.s, groupname) }
    }

    pub fn append() {}

    pub fn append_vector(&self, fields: *mut Sds, len: usize) -> StreamID {
        unsafe {
            let added_id: StreamID = std::mem::uninitialized();

            streamAppendItemSDSMap(
                self.s,
                fields,
//                &fields as *mut *mut _ as *mut *mut libc::c_void,
                len as i64,
                &added_id,
                ptr::null_mut(),
            );

            added_id
        }
    }

    //    pub fn append(&self, fields: &mut Vec<Sds>) {
//        unsafe {
//            let mut added_id: *mut StreamID = ptr::null_mut();
////            let mut added_id: *mut StreamID = ptr::null_mut();
//
//            streamAppendItem2(
//                self.s,
//                fields.as_mut_ptr(),
//                fields.len() as i64,
//                added_id,
//                ptr::null_mut(),
//            )
//        }
//    }
    pub fn append_stream() {}
}

//
impl Drop for Stream {
    fn drop(&mut self) {
        unsafe { freeStream(self.s) }
    }
}

#[derive(Copy)]
#[repr(C)]
pub struct StreamID {
    ms: libc::uint64_t,
    seq: libc::uint64_t,
}

impl fmt::Debug for StreamID {
    fn fmt(&self, _f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        Ok(())
    }
}

impl Default for StreamID {
    fn default() -> Self {
        StreamID { ms: 0, seq: 0 }
    }
}

impl Clone for StreamID {
    fn clone(&self) -> Self {
        StreamID { ms: self.ms, seq: self.seq }
    }
}


impl RaxKey for StreamID {
    type Output = StreamID;

    fn encode(self) -> Self::Output {
        StreamID {
            ms: self.ms.to_be(),
            seq: self.seq.to_be(),
        }
    }

    fn to_buf(&self) -> (*const u8, usize) {
        (self as *const _ as *const u8, size_of::<StreamID>())
    }

    fn from_buf(ptr: *const u8, len: usize) -> StreamID {
        if len != size_of::<StreamID>() {
            return StreamID::default();
        }

        unsafe {
            StreamID {
                ms: u64::from_be(*(ptr as *mut [u8; 8] as *mut u64)),
                seq: u64::from_be(*(ptr.offset(8) as *mut [u8; 8] as *mut u64)),
            }
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct EntryPack;

//use std::fs::File;
//use std::io::prelude::*;

impl EntryPack {


    fn read_from_from() {

//        std::fs::File::open()
//
//        let mut f = File::open(filename).expect("file not found");
//
//        let mut contents = String::new();
//        f.read_to_string(&mut contents)
//            .expect("something went wrong reading the file");
    }

    fn deserialize() -> *mut listpack {
        std::ptr::null_mut()
    }

    fn append(lp: *mut listpack, fields: &[Sds]) -> *mut listpack {
        /* Create a new listpack and radix tree node if needed. Note that when
     * a new listpack is created, we populate it with a "master entry". This
     * is just a set of fields that is taken as refernce in order to compress
     * the stream entries that we'll add inside the listpack.
     *
     * Note that while we use the first added entry fields to create
     * the master entry, the first added entry is NOT represented in the master
     * entry, which is a stand alone object. But of course, the first entry
     * will compress well because it's used as reference.
     *
     * The master entry is composed like in the following example:
     *
     * +-------+---------+------------+---------+--/--+---------+---------+-+
     * | count | deleted | num-fields | field_1 | field_2 | ... | field_N |0|
     * +-------+---------+------------+---------+--/--+---------+---------+-+
     *
     * count and deleted just represent respectively the total number of
     * entries inside the listpack that are valid, and marked as deleted
     * (delted flag in the entry flags set). So the total number of items
     * actually inside the listpack (both deleted and not) is count+deleted.
     *
     * The real entries will be encoded with an ID that is just the
     * millisecond and sequence difference compared to the key stored at
     * the radix tree node containing the listpack (delta encoding), and
     * if the fields of the entry are the same as the master enty fields, the
     * entry flags will specify this fact and the entry fields and number
     * of fields will be omitted (see later in the code of this function).
     *
     * The "0" entry at the end is the same as the 'lp-count' entry in the
     * regular stream entries (see below), and marks the fact that there are
     * no more entries, when we scan the stream from right to left. */
        unsafe {
            if lp.is_null() {
                let numfields = fields.len() / 2;


                let mut lp = lpNew();
                lp = lpAppendInteger(lp, 1);
                lp = lpAppendInteger(lp, 0);
                lp = lpAppendInteger(lp, numfields as i64);
                for i in 0..numfields {
                    let field = fields[i * 2];
                    lp = lpAppend(lp, field as *mut u8, get_len(field) as u32);
                }
                lp = lpAppendInteger(lp, 0); /* Master entry zero terminator. */
            }
        }

        lp
    }

//    fn append(lp: *mut listpack, map: &[Sds]) -> (*mut listpack, bool) {
//        (lp, true)
//    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct stream {
    rax: *mut ::rax::rax,
    length: libc::uint64_t,
    last_id: StreamID,
    cgroups: *mut u8,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct streamIterator;
//    stream: *mut stream,
//    master_id: StreamID,
//    master_fields_count: libc::uint64_t,
//    master_fields_start
//}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct streamCG {
    last_id: StreamID,
    pel: *mut rax,
    consumers: *mut rax,
}

//#[derive(Clone, Copy)]
#[repr(C)]
pub struct streamConsumer {
    seen_time: libc::c_longlong,
    name: Sds,
    pel: *mut rax,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct streamNACK {
    delivery_time: libc::c_longlong,
    delivery_count: libc::uint64_t,
    consumer: *mut streamConsumer,
}

#[allow(improper_ctypes)]
#[allow(non_snake_case)]
#[link(name = "redismodule", kind = "static")]
extern "C" {
//    fn createObject()

    fn streamNew() -> *mut stream;

    fn freeStream(s: *mut stream);

    fn streamAppendItemSDSMap(
        s: *mut stream,
        argv: *mut Sds,
        numfields: libc::int64_t,
        added_id: *const StreamID,
        use_id: *mut StreamID,
    );

    fn streamIteratorStart(
        si: *mut streamIterator,
        s: *mut stream,
        start: StreamID,
        end: StreamID,
        rev: libc::c_int,
    );

    fn streamIteratorGetID(
        si: *mut streamIterator,
        id: *mut StreamID,
        numfields: *mut libc::int64_t,
    ) -> libc::c_int;

    fn streamIteratorGetField(
        si: *mut streamIterator,
        fieldptr: *mut *mut u8,
        valueptr: *mut *mut u8,
        fieldlen: *mut libc::int64_t,
        valuelen: *mut libc::int64_t,
    );

    fn streamIteratorRemoveEntry(
        si: *mut streamIterator,
        id: *mut StreamID,
    ) -> libc::c_int;

    fn streamIteratorStop(
        si: *mut streamIterator,
    ) -> libc::c_int;

    fn streamDeleteItem(
        s: *mut stream,
        id: *mut StreamID,
    ) -> libc::c_int;

    fn string2ull(
        s: *const libc::c_char,
        value: *mut libc::uint64_t,
    ) -> libc::c_int;

    fn streamCreateNACK(
        consumer: *mut streamConsumer
    ) -> *mut streamNACK;

    fn streamFreeNACK(
        na: *mut streamNACK
    );

    fn streamFreeConsumer(
        sc: *mut streamConsumer
    );

    fn streamCreateCG(
        s: *mut stream,
        name: *mut libc::c_char,
        namelen: libc::size_t, id: *mut StreamID,
    ) -> *mut streamCG;

    fn streamFreeCG(cg: *mut streamCG);

    fn streamLookupCG(
        s: *mut stream,
        groupname: Sds,
    ) -> *mut streamCG;

    fn streamLookupConsumer(
        cg: *mut streamCG,
        name: Sds,
        create: libc::c_int,
    ) -> *mut streamConsumer;

    fn streamDelConsumer(
        cg: *mut streamCG,
        name: Sds,
    ) -> libc::uint64_t;
}

#[cfg(test)]
mod tests {
    use rax::*;
    use sds;
    //    use std;
    use stream::Stream;

    #[test]
    fn it_works() {
        let s = Stream::new();

//        let mut array = ArrayVec::from([
//            sds::sds_new("id"),
//            sds::sds_from_long_long(1),
//            sds::sds_new("auth-key"),
//            sds::sds_new_len("some_really_long_auth_ley"),
//            sds::sds_new("data"),
//            sds::sds_new_len("{\"id\": \"JSON_ID\"}")
//        ]);

        let mut x = [
            sds::new("128"),
            sds::new("123"),
            sds::new("1234"),
            sds::new("12345"),
//            sds::sds_from_long_long(1),
//            sds::sds_new("auth-key"),
//            sds::sds_new_len("some_really_long_auth_ley"),
//            sds::sds_new("data"),
//            sds::sds_new_len("{\"id\": \"JSON_ID\"}")
        ];

        let ss = sds::new("hi");
//        sds::sds_len(ss);
        println!("{}", sds::get_len(ss));


//        sds::sds_dup(x[0]);
//        sds::sds_dup(x[1]);


        for _ in 0..1000 {
            let mut id = s.append_vector(x.as_mut_ptr(), x.len() / 2);
//            println!("{}-{}", id.ms, id.seq);
        }
        unsafe {
            raxShow((*s.s).rax);
        }
//        let mut id = s.append_vector((x).as_mut_ptr(), x.len() / 2);
//        println!("{}-{}", id.ms, id.seq);
//        id = s.append_vector((x).as_mut_ptr(), x.len() / 2);
//        println!("{}-{}", id.ms, id.seq);
    }
}