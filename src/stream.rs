//#![allow(dead_code)]
//
//use arrayvec::*;
//use libc;
//use rax::rax;
//use sds;
//use sds::Sds;
//use std::ptr;
//use std;
//
//pub struct Stream {
//    s: *mut stream,
//}
//
//const STREAM_ID: StreamID = StreamID{ ms: 0, seq: 0 };
//const STREAM_ID_REF: *const StreamID = &STREAM_ID as *const StreamID;
//
//impl Stream {
//    pub fn new() -> Stream {
//        return Stream { s: unsafe { streamNew() } };
//    }
//
//    pub fn append() {
//
//    }
//
//    pub fn append_vector(&self, mut fields: *mut Sds, len: usize) -> StreamID {
//        unsafe {
//            let mut added_id: StreamID = std::mem::uninitialized();
//
//            streamAppendItem2(
//                self.s,
//                fields,
////                &fields as *mut *mut _ as *mut *mut libc::c_void,
//                len as i64,
//                &added_id,
//                ptr::null_mut(),
//            );
//
//            added_id
//        }
//    }
//
////    pub fn append(&self, fields: &mut Vec<Sds>) {
////        unsafe {
////            let mut added_id: *mut StreamID = ptr::null_mut();
//////            let mut added_id: *mut StreamID = ptr::null_mut();
////
////            streamAppendItem2(
////                self.s,
////                fields.as_mut_ptr(),
////                fields.len() as i64,
////                added_id,
////                ptr::null_mut(),
////            )
////        }
////    }
//    pub fn append_stream() {}
//}
//
////
//impl Drop for Stream {
//    fn drop(&mut self) {
//        unsafe { freeStream(self.s) }
//    }
//}
//
//#[derive(Clone, Copy)]
//#[repr(C)]
//pub struct StreamID {
//    ms: libc::uint64_t,
//    seq: libc::uint64_t,
//}
//
//#[derive(Clone, Copy)]
//#[repr(C)]
//pub struct stream {
//    rax: *mut ::rax::rax,
//    length: libc::uint64_t,
//    last_id: StreamID,
//    cgroups: *mut u8,
//}
//
//#[derive(Clone, Copy)]
//#[repr(C)]
//pub struct streamIterator;
////    stream: *mut stream,
////    master_id: StreamID,
////    master_fields_count: libc::uint64_t,
////    master_fields_start
////}
//
//#[derive(Clone, Copy)]
//#[repr(C)]
//pub struct streamCG {
//    last_id: StreamID,
//    pel: *mut rax,
//    consumers: *mut rax,
//}
//
////#[derive(Clone, Copy)]
//#[repr(C)]
//pub struct streamConsumer {
//    seen_time: libc::c_longlong,
//    name: Sds,
//    pel: *mut rax,
//}
//
//#[derive(Clone, Copy)]
//#[repr(C)]
//pub struct streamNACK {
//    delivery_time: libc::c_longlong,
//    delivery_count: libc::uint64_t,
//    consumer: *mut streamConsumer,
//}
//
//#[allow(improper_ctypes)]
//#[allow(non_snake_case)]
//#[link(name = "redismodule", kind = "static")]
//extern "C" {
////    fn createObject()
//
//    fn streamNew() -> *mut stream;
//
//    fn freeStream(s: *mut stream);
//
////    fn streamAppendItem(s: *mut stream,
////                        argv: *mut *mut libc::c_void,
////                        numfields: libc::int64_t,
////                        added_id: *mut StreamID,
////                        use_id: *mut StreamID);
//
//    fn streamAppendItem2(s: *mut stream,
//                         argv: *mut Sds,
//                         numfields: libc::int64_t,
//                         added_id: *const StreamID,
//                         use_id: *mut StreamID);
//}
//
//#[cfg(test)]
//mod tests {
//    use arrayvec::ArrayVec;
//    use sds;
//    use std;
//    use stream::Stream;
//
//    #[test]
//    fn it_works() {
//        let mut s = Stream::new();
//
////        let mut array = ArrayVec::from([
////            sds::sds_new("id"),
////            sds::sds_from_long_long(1),
////            sds::sds_new("auth-key"),
////            sds::sds_new_len("some_really_long_auth_ley"),
////            sds::sds_new("data"),
////            sds::sds_new_len("{\"id\": \"JSON_ID\"}")
////        ]);
//
//        let mut x = [
//            sds::new("128"),
//            sds::new("123"),
//            sds::new("1234"),
//            sds::new("12345"),
////            sds::sds_from_long_long(1),
////            sds::sds_new("auth-key"),
////            sds::sds_new_len("some_really_long_auth_ley"),
////            sds::sds_new("data"),
////            sds::sds_new_len("{\"id\": \"JSON_ID\"}")
//        ];
//
//        let ss = sds::new("hi");
////        sds::sds_len(ss);
//        println!("{}", sds::len(ss));
//
//
////        sds::sds_dup(x[0]);
////        sds::sds_dup(x[1]);
//
//
//        let mut id = s.append_vector((x).as_mut_ptr(), x.len() / 2);
//        println!("{}-{}", id.ms, id.seq);
//        id = s.append_vector((x).as_mut_ptr(), x.len() / 2);
//        println!("{}-{}", id.ms, id.seq);
//    }
//}