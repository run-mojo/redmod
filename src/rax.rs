use error::RedError;
use libc;
use sds::SDS;
use std;
use std::ptr;
use std::mem::{transmute, size_of};

/// Redis has a beautiful Radix Tree implementation in ANSI C.
/// This brings it to Rust. Great effort went into trying this zero overhead.
/// If you catch something that could be better go ahead and share it.
pub trait RAX<K: RaxKey, V> {
    fn show(&self);

    fn find(&self, key: K) -> Option<&V>;

    fn insert(&mut self, key: K, data: Box<V>) -> Result<(i32, Option<Box<V>>), RedError>;

    fn remove(&mut self, key: K) -> (bool, Option<Box<V>>);
}

pub trait RAXIter<K: RaxKey, V>: RAX<K, V> {}


#[derive(Clone)]
pub struct Rax<K: RaxKey, V> {
    pub rax: *mut rax,
    /// Having more than one iterator per Rax is pretty uncommon.
    /// We'll go ahead and keep a lot open for a pointer to one that
    /// whose lifespan will equal the associated rax.
    pub it: *mut raxIterator,
    phantom: std::marker::PhantomData<(K, V)>,
}

impl<K: RaxKey, V> Rax<K, V> {
    pub fn new() -> Rax<K, V> {
        unsafe {
            let r = raxNew();
            Rax {
                rax: r,
//                it: std::ptr::null_mut(),
                it: raxIteratorNew(r),
                phantom: std::marker::PhantomData,
            }
        }
    }

    ///
    /// The number of entries in the RAX
    ///
    pub fn size(&self) -> u64 {
        unsafe { raxSize(self.rax) }
    }

    pub fn show(&self) {
        unsafe { raxShow(self.rax) }
    }

    pub fn noop(&mut self) {}

    ///
    /// Insert a new entry into the RAX
    ///
    pub fn insert_null(&mut self, key: K) -> Result<(i32, Option<Box<V>>), RedError> {
        unsafe {
            // Allocate a pointer to catch the old value.
            let old: &mut *mut u8 = &mut ptr::null_mut();

            // Integer values require Big Endian to allow the Rax to fully optimize
            // storing them since it will be able to compress the prefixes especially
            // for 64/128bit numbers.
            let k = key.for_encoding();
            let (ptr, len) = k.encode();

            let r = raxInsert(
                self.rax,
                // Grab a raw pointer to the key. Keys are most likely allocated
                // on the stack. The rax will keep it's own copy of the key so we
                // don't want to keep in in the heap twice and it exists in the
                // rax in it's compressed form.
                ptr,
                len,
                std::ptr::null_mut(),
                old,
            );

            // Was there an existing entry?
            if old.is_null() {
                Ok((r, None))
            } else {
                // Box the previous value since Rax is done with it and it's our
                // responsibility now to drop it. Once this Box goes out of scope
                // the value is dropped and memory reclaimed.
                Ok((r, Some(Box::from_raw(*old as *mut V))))
            }
        }
    }

    ///
    /// Insert a new entry into the RAX
    ///
    pub fn insert(&mut self, key: K, data: Box<V>) -> Result<(i32, Option<Box<V>>), RedError> {
        unsafe {
            // Allocate a pointer to catch the old value.
            let old: &mut *mut u8 = &mut ptr::null_mut();

            // Leak the boxed value as we hand it over to Rax to keep track of.
            // These must be heap allocated unless we want to store sizeof(usize) or
            // less bytes, then the value can be the pointer.
            let value: &mut V = Box::leak(data);

            // Integer values require Big Endian to allow the Rax to fully optimize
            // storing them since it will be able to compress the prefixes especially
            // for 64/128bit numbers.
            let k = key.for_encoding();
            let (ptr, len) = k.encode();

            let r = raxInsert(
                self.rax,
                // Grab a raw pointer to the key. Keys are most likely allocated
                // on the stack. The rax will keep it's own copy of the key so we
                // don't want to keep in in the heap twice and it exists in the
                // rax in it's compressed form.
                ptr,
                len,
                value as *mut V as *mut u8,
                old,
            );

            // Was there an existing entry?
            if old.is_null() {
                Ok((r, None))
            } else {
                // Box the previous value since Rax is done with it and it's our
                // responsibility now to drop it. Once this Box goes out of scope
                // the value is dropped and memory reclaimed.
                Ok((r, Some(Box::from_raw(*old as *mut V))))
            }
        }
    }

    ///
    ///
    ///
    pub fn remove(&mut self, key: K) -> (bool, Option<Box<V>>) {
        unsafe {
            let old: &mut *mut u8 = &mut ptr::null_mut();
            let k = key.for_encoding();
            let (ptr, len) = k.encode();

            let r = raxRemove(
                self.rax,
                ptr,
                len,
                old,
            );

            if old.is_null() {
                (r == 1, None)
            } else {
                (r == 1, Some(Box::from_raw(*old as *mut V)))
            }
        }
    }

    ///
    ///
    ///
    pub fn find_exists(&self, key: K) -> (bool, Option<&V>) {
        unsafe {
            let k = key.for_encoding();
            let (ptr, len) = k.encode();

            let value = raxFind(
                self.rax,
                ptr,
                len,
            );

            if value.is_null() {
                (true, None)
            } else if value == raxNotFound {
                (false, None)
            } else {
                // transmute to the value so we don't drop the actual value accidentally.
                // While the key associated to the value is in the RAX then we cannot
                // drop it.
                (true, Some(transmute(value)))
            }
        }
    }

    ///
    ///
    ///
    pub fn find(&self, key: K) -> Option<&V> {
        unsafe {
            let k = key.for_encoding();
            let (ptr, len) = k.encode();

            let value = raxFind(
                self.rax,
                ptr,
                len,
            );

            if value.is_null() || value == raxNotFound {
                None
            } else {
                // transmute to the value so we don't drop the actual value accidentally.
                // While the key associated to the value is in the RAX then we cannot
                // drop it.
                Some(std::mem::transmute(value))
            }
        }
    }

    ///
    ///
    ///
    pub fn exists(&self, key: K) -> bool {
        unsafe {
            let k = key.for_encoding();
            let (ptr, len) = k.encode();

            let value = raxFind(
                self.rax,
                ptr,
                len,
            );

            if value.is_null() || value == raxNotFound {
                false
            } else {
                true
            }
        }
    }

    ///
    ///
    ///
    pub fn iterator(&self) -> RaxIterator<V> {
        unsafe {
            RaxIterator {
                it: raxIteratorNew(self.rax),
                marker: std::marker::PhantomData,
            }
        }
    }

    ///
    ///
    ///
    pub fn first(&self) -> bool {
        unsafe {
            raxSeek(self.it, RAX_MIN, ptr::null(), 0) == 1
        }
    }

    ///
    ///
    ///
    pub fn last(&self) -> bool {
        unsafe {
            raxSeek(self.it, RAX_MAX, ptr::null(), 0) == 1
        }
    }

    ///
    ///
    ///
    pub fn prev(&self) -> bool {
        unsafe {
            raxPrev(self.it) == 1
        }
    }

    ///
    ///
    ///
    pub fn next(&self) -> bool {
        unsafe {
            raxNext(self.it) == 1
        }
    }

    ///
    ///
    ///
    pub fn key(&self) -> K {
        unsafe { K::decode((*self.it).key, (*self.it).key_len as usize) }
    }

    ///
    ///
    ///
    pub fn key_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts((*self.it).key, (*self.it).key_len as usize) }
    }

    ///
    ///
    ///
    pub fn key_ptr(&self) -> (*const u8, usize) {
        unsafe {
            if self.it.is_null() {
                (ptr::null(), 0)
            } else {
                ((*self.it).key as *const u8, (*self.it).key_len)
            }
        }
    }

    ///
    ///
    ///
    pub fn key_mut_ptr(&self) -> (*mut u8, usize) {
        unsafe {
            if self.it.is_null() {
                (std::ptr::null_mut(), 0)
            } else {
                ((*self.it).key, (*self.it).key_len)
            }
        }
    }

    ///
    ///
    ///
    pub fn data(&self) -> Option<&V> {
        unsafe {
            let data: *mut libc::c_void = (*self.it).data;
            if data.is_null() {
                None
            } else {
                Some(std::mem::transmute(data as *mut u8))
            }
        }
    }

    pub fn seek(&self, op: *const u8, ele: &[u8]) -> bool {
        unsafe {
            raxSeek(self.it, op, ele.as_ptr(), ele.len() as libc::size_t) == 1
        }
    }
}

//
impl<K: RaxKey, V> Drop for Rax<K, V> {
    fn drop(&mut self) {
        unsafe {
            // Cleanup RAX
            raxFreeWithCallback(self.rax, rax_free_with_callback_wrapper::<V>);

            // Cleanup iterator.
            if !self.it.is_null() {
                raxStop(self.it);
                raxIteratorFree(self.it);
                self.it = std::ptr::null_mut();
            }
        }
    }
}

pub trait RaxKey<RHS = Self>: Clone + Default + std::fmt::Debug {
    type Output: RaxKey;

    fn for_encoding(self) -> Self::Output;

    fn encode(&self) -> (*const u8, usize);

    fn decode(ptr: *const u8, len: usize) -> RHS;
}

impl RaxKey for f32 {
    type Output = u32;

    #[inline]
    fn for_encoding(self) -> Self::Output {
        // Encode as u32 Big Endian
        self.to_bits().to_be()
    }

    #[inline]
    fn encode(&self) -> (*const u8, usize) {
        // This should never get called since we represent as a u32
        (self as *const _ as *const u8, std::mem::size_of::<f32>())
    }

    #[inline]
    fn decode(ptr: *const u8, len: usize) -> f32 {
        if len != size_of::<Self>() {
            return Self::default()
        }
        unsafe {
            // We used a BigEndian u32 to encode so let's reverse it
            f32::from_bits(
                u32::from_be(
                    *(ptr as *mut [u8; std::mem::size_of::<u32>()] as *mut u32)
                )
            )
        }
    }
}

impl RaxKey for f64 {
    type Output = u64;

    #[inline]
    fn for_encoding(self) -> Self::Output {
        // Encode as u64 Big Endian
        self.to_bits().to_be()
    }

    #[inline]
    fn encode(&self) -> (*const u8, usize) {
        // This should never get called since we represent as a u64
        (self as *const _ as *const u8, size_of::<f64>())
    }

    #[inline]
    fn decode(ptr: *const u8, len: usize) -> f64 {
        if len != size_of::<Self>() {
            return Self::default()
        }
        unsafe {
            // We used a BigEndian u64 to encode so let's reverse it
            f64::from_bits(
                u64::from_be(
                    *(ptr as *mut [u8; size_of::<u64>()] as *mut u64)
                )
            )
        }
    }
}

impl RaxKey for isize {
    type Output = isize;

    #[inline]
    fn for_encoding(self) -> Self::Output {
        self.to_be()
    }

    #[inline]
    fn encode(&self) -> (*const u8, usize) {
        (self as *const _ as *const u8, size_of::<isize>())
    }

    #[inline]
    fn decode(ptr: *const u8, len: usize) -> isize {
        if len != size_of::<Self>() {
            return Self::default()
        }
        unsafe { isize::from_be(*(ptr as *mut [u8; size_of::<isize>()] as *mut isize)) }
    }
}

impl RaxKey for usize {
    type Output = usize;

    #[inline]
    fn for_encoding(self) -> Self::Output {
        self.to_be()
    }

    #[inline]
    fn encode(&self) -> (*const u8, usize) {
        (self as *const _ as *const u8, std::mem::size_of::<usize>())
    }

    #[inline]
    fn decode(ptr: *const u8, len: usize) -> usize {
        if len != size_of::<Self>() {
            return Self::default()
        }
        unsafe { usize::from_be(*(ptr as *mut [u8; std::mem::size_of::<usize>()] as *mut usize)) }
    }
}

impl RaxKey for i16 {
    type Output = i16;

    #[inline]
    fn for_encoding(self) -> Self::Output {
        self.to_be()
    }

    #[inline]
    fn encode(&self) -> (*const u8, usize) {
        (self as *const _ as *const u8, 2)
    }

    #[inline]
    fn decode(ptr: *const u8, len: usize) -> i16 {
        if len != size_of::<Self>() {
            return Self::default()
        }
        unsafe { i16::from_be(*(ptr as *mut [u8; 2] as *mut i16)) }
    }
}

impl RaxKey for u16 {
    type Output = u16;

    #[inline]
    fn for_encoding(self) -> Self::Output {
        self.to_be()
    }

    #[inline]
    fn encode(&self) -> (*const u8, usize) {
        (self as *const _ as *const u8, 2)
    }

    #[inline]
    fn decode(ptr: *const u8, len: usize) -> u16 {
        if len != size_of::<Self>() {
            return Self::default()
        }
        unsafe { u16::from_be(*(ptr as *mut [u8; 2] as *mut u16)) }
    }
}

impl RaxKey for i32 {
    type Output = i32;

    #[inline]
    fn for_encoding(self) -> Self::Output {
        self.to_be()
    }

    #[inline]
    fn encode(&self) -> (*const u8, usize) {
        (self as *const _ as *const u8, 4)
    }

    #[inline]
    fn decode(ptr: *const u8, len: usize) -> i32 {
        if len != size_of::<Self>() {
            return Self::default()
        }
        unsafe { i32::from_be(*(ptr as *mut [u8; 4] as *mut i32)) }
    }
}

impl RaxKey for u32 {
    type Output = u32;

    #[inline]
    fn for_encoding(self) -> Self::Output {
        self.to_be()
    }

    #[inline]
    fn encode(&self) -> (*const u8, usize) {
        (self as *const _ as *const u8, 4)
    }

    #[inline]
    fn decode(ptr: *const u8, len: usize) -> u32 {
        if len != size_of::<Self>() {
            return Self::default()
        }
        unsafe { u32::from_be(*(ptr as *mut [u8; 4] as *mut u32)) }
    }
}

impl RaxKey for i64 {
    type Output = i64;

    #[inline]
    fn for_encoding(self) -> Self::Output {
        self.to_be()
    }

    #[inline]
    fn encode(&self) -> (*const u8, usize) {
        (self as *const _ as *const u8, 8)
    }

    #[inline]
    fn decode(ptr: *const u8, len: usize) -> i64 {
        if len != size_of::<Self>() {
            return Self::default()
        }
        unsafe { i64::from_be(*(ptr as *mut [u8; 8] as *mut i64)) }
    }
}

impl RaxKey for u64 {
    type Output = u64;

    #[inline]
    fn for_encoding(self) -> Self::Output {
        self.to_be()
    }

    #[inline]
    fn encode(&self) -> (*const u8, usize) {
        (self as *const _ as *const u8, 8)
    }

    #[inline]
    fn decode(ptr: *const u8, len: usize) -> u64 {
        if len != size_of::<Self>() {
            return Self::default()
        }
        unsafe { u64::from_be(*(ptr as *mut [u8; 8] as *mut u64)) }
    }
}

impl RaxKey for i128 {
    type Output = i128;

    #[inline]
    fn for_encoding(self) -> Self::Output {
        self.to_be()
    }

    #[inline]
    fn encode(&self) -> (*const u8, usize) {
        (self as *const _ as *const u8, 16)
    }

    #[inline]
    fn decode(ptr: *const u8, len: usize) -> i128 {
        if len != size_of::<Self>() {
            return Self::default()
        }
        unsafe { i128::from_be(*(ptr as *mut [u8; 16] as *mut i128)) }
    }
}

impl RaxKey for u128 {
    type Output = u128;

    #[inline]
    fn for_encoding(self) -> Self::Output {
        self.to_be()
    }

    #[inline]
    fn encode(&self) -> (*const u8, usize) {
        (self as *const _ as *const u8, 16)
    }

    #[inline]
    fn decode(ptr: *const u8, len: usize) -> u128 {
        if len != size_of::<Self>() {
            return Self::default()
        }
        unsafe { u128::from_be(*(ptr as *mut [u8; 16] as *mut u128)) }
    }
}

impl RaxKey for SDS {
    type Output = SDS;

    #[inline]
    fn for_encoding(self) -> Self::Output {
        self
    }

    #[inline]
    fn encode(&self) -> (*const u8, usize) {
        (self.as_ptr(), self.len())
    }

    #[inline]
    fn decode(ptr: *const u8, len: usize) -> SDS {
        SDS::from_ptr(ptr, len)
    }
}

impl<'a> RaxKey for &'a str {
    type Output = &'a str;

    #[inline]
    fn for_encoding(self) -> Self::Output {
        self
    }

    #[inline]
    fn encode(&self) -> (*const u8, usize) {
        ((*self).as_ptr(), self.len())
    }

    #[inline]
    fn decode(ptr: *const u8, len: usize) -> &'a str {
        unsafe {
            std::str::from_utf8(
                std::slice::from_raw_parts(ptr, len)
            ).unwrap_or_default()
        }
    }
}

pub struct RaxIterator<V> {
    it: *mut raxIterator,
    marker: std::marker::PhantomData<V>,
}

impl<V> RaxIterator<V> {
    pub fn new_heap_allocated() {}

    pub fn first(&self) -> bool {
        unsafe {
            raxSeek(self.it, MIN.as_ptr(), std::ptr::null(), 0) == 1
        }
    }

    pub fn last(&self) -> bool {
        unsafe {
            raxSeek(self.it, MAX.as_ptr(), std::ptr::null(), 0) == 1
        }
    }

    pub fn prev(&self) -> bool {
        unsafe {
            raxPrev(self.it) == 1
        }
    }

    pub fn next(&self) -> bool {
        unsafe {
            raxNext(self.it) == 1
        }
    }

    pub fn key(&self) -> &[u8] {
        "".as_ref()
    }

    pub fn data(&self) -> Option<&V> {
        unsafe {
            let data: *mut libc::c_void = (*self.it).data;
            if data.is_null() {
                None
            } else {
                Some(std::mem::transmute(data as *mut u8))
            }
        }
    }

    pub fn seek(&self, op: *const u8, ele: &[u8]) -> bool {
        unsafe {
            raxSeek(self.it, op, ele.as_ptr(), ele.len() as libc::size_t) == 1
        }
    }
}

impl<V> Drop for RaxIterator<V> {
    fn drop(&mut self) {
        unsafe {
            raxStop(self.it);
            raxIteratorFree(self.it);
        }
    }
}


#[derive(Clone, Copy)]
#[repr(C)]
pub struct rax;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct raxNode;

#[derive(Clone, Copy)]
#[repr(C)]
//pub struct raxStack;
pub struct raxStack {
    stack: *mut *mut libc::c_void,
    items: libc::size_t,
    maxitems: libc::size_t,
    static_items: [*mut libc::c_void; 32],
    oom: libc::c_int,
}

pub const GT: &'static str = ">";
pub const GTE: &'static str = ">=";
pub const LT: &'static str = "<";
pub const LTE: &'static str = "<=";
pub const EQ: &'static str = "=";
pub const MIN: &'static str = "^";
pub const MAX: &'static str = "$";

pub const RAX_NODE_MAX_SIZE: libc::c_int = ((1 << 29) - 1);
pub const RAX_STACK_STATIC_ITEMS: libc::c_int = 128;
pub const RAX_ITER_STATIC_LEN: libc::c_int = 128;
pub const RAX_ITER_JUST_SEEKED: libc::c_int = (1 << 0);
pub const RAX_ITER_EOF: libc::c_int = (1 << 1);
pub const RAX_ITER_SAFE: libc::c_int = (1 << 2);

#[derive(Clone, Copy)]
#[repr(C)]
//pub struct raxIterator;
pub struct raxIterator {
    pub flags: libc::c_int,
    pub rt: *mut rax,
    pub key: *mut u8,
    pub data: *mut libc::c_void,
    pub key_len: libc::size_t,
    pub key_max: libc::size_t,
    pub key_static_string: [u8; 128],
    pub node: *mut raxNode,
    pub stack: raxStack,
    pub node_cb: Option<raxNodeCallback>,
}

extern "C" fn rax_free_with_callback_wrapper<V>(v: *mut libc::c_void) {
    unsafe {
        // Re-box it so it can drop it immediately after it leaves this scope.
        Box::from_raw(v as *mut V);
    }
}

#[allow(non_camel_case_types)]
type raxNodeCallback = extern "C" fn(v: *mut libc::c_void);


type RaxFreeCallback = extern "C" fn(v: *mut libc::c_void);

#[allow(improper_ctypes)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[link(name = "redismodule", kind = "static")]
extern "C" {
    #[no_mangle]
    pub static RAX_GREATER: *const u8; // '>'
#[no_mangle]
pub static RAX_GREATER_EQUAL: *const u8; // '>='
#[no_mangle]
pub static RAX_LESSER: *const u8; // '<'
#[no_mangle]
pub static RAX_LESSER_EQUAL: *const u8; // '<='
#[no_mangle]
pub static RAX_EQUAL: *const u8; // '='
#[no_mangle]
pub static RAX_MIN: *const u8; // '^'
#[no_mangle]
pub static RAX_MAX: *const u8; // '$'

    #[no_mangle]
    pub static raxNotFound: *mut u8;

    fn raxIteratorFree(it: *mut raxIterator);

    fn raxIteratorSize() -> libc::c_int;

    fn raxNew() -> *mut rax;

    fn raxFree(rax: *mut rax);

    fn raxFreeWithCallback(rax: *mut rax, callback: RaxFreeCallback);

    fn raxInsert(rax: *mut rax,
                 s: *const u8,
                 len: libc::size_t,
                 data: *const u8,
                 old: &mut *mut u8) -> libc::c_int;

    fn raxTryInsert(rax: *mut rax,
                    s: *mut u8,
                    len: libc::size_t,
                    data: *mut libc::c_void,
                    old: *mut *mut libc::c_void) -> libc::c_int;

    fn raxRemove(rax: *mut rax,
                 s: *const u8,
                 len: libc::size_t,
                 old: &mut *mut u8) -> libc::c_int;

    fn raxFind(rax: *mut rax, s: *const u8, len: libc::size_t) -> *mut u8;

    fn raxIteratorNew(rt: *mut rax) -> *mut raxIterator;

    fn raxStart(it: *mut raxIterator, rt: *mut rax);

    fn raxSeek(it: *mut raxIterator,
               op: *const u8,
               ele: *const u8,
               len: libc::size_t) -> libc::c_int;

    fn raxNext(it: *mut raxIterator) -> libc::c_int;

    fn raxPrev(it: *mut raxIterator) -> libc::c_int;

    fn raxRandomWalk(it: *mut raxIterator, steps: libc::size_t) -> libc::c_int;

    fn raxCompare(it: *mut raxIterator,
                  op: *const u8,
                  key: *mut u8,
                  key_len: libc::size_t) -> libc::c_int;

    fn raxStop(it: *mut raxIterator);

    fn raxEOF(it: *mut raxIterator) -> libc::c_int;

    fn raxShow(rax: *mut rax);

    fn raxSize(rax: *mut rax) -> libc::uint64_t;
}

pub type RaxOp = *const u8;


#[cfg(test)]
mod tests {
    use rax::*;
    use sds;
    use std;
    use test::Bencher;
    use stopwatch::Stopwatch;

    pub struct MyMsg<'a>(&'a str);

    impl<'a> Drop for MyMsg<'a> {
        fn drop(&mut self) {
            println!("dropped -> {}", self.0);
        }
    }

    fn create_map() {}


    fn fibonacci(n: u64) -> u64 {
        match n {
            0 => 1,
            1 => 1,
            n => fibonacci(n - 1) + fibonacci(n - 2),
        }
    }

    #[bench]
    fn bench_fib(_b: &mut Bencher) {
        let r = &mut Rax::<u64, &str>::new();
        for x in 0..2000 {
            r.insert_null(x).expect("whoops!");
        }

        let sw = Stopwatch::start_new();

        for _po in 0..1000000 {
            r.find(300);
        }

        println!("Thing took {}ms", sw.elapsed_ms());
    }

    #[test]
    fn bench_tree() {
        for _ in 0..10 {
            let r = &mut std::collections::BTreeMap::<u64, &str>::new();
            for x in 0..2000 {
                r.insert(x, "");
            }

            let sw = Stopwatch::start_new();

            let xx = 300;
            for _po in 0..1000000 {
                r.get(&xx);
            }

            println!("Thing took {}ms", sw.elapsed_ms());
        }
    }

    #[test]
    fn bench_rax_find() {
        for _ in 0..10 {
            let r = &mut Rax::<u64, &str>::new();
            for x in 0..2000 {
                r.insert_null(x).expect("whoops!");
            }

            match r.find(1601) {
                Some(v) => println!("{}", v),
                None => {}
            }

            let sw = Stopwatch::start_new();

            for _po in 0..1000000 {
                r.find(1601);
            }

            println!("Thing took {}ms", sw.elapsed_ms());
        }
    }

    #[test]
    fn bench_hash_find() {
        for _ in 0..10 {
            let r = &mut std::collections::HashMap::<u64, &str>::new();
//            r.insert_null(300);
            for x in 0..2000 {
                r.insert(x, "");
            }

            let sw = Stopwatch::start_new();

            let xx = 300;
            for _po in 0..1000000 {
                r.get(&xx);
            }

            println!("Thing took {}ms", sw.elapsed_ms());
        }
    }

    #[test]
    fn bench_rax_insert() {
        for _ in 0..10 {
            let mut r = &mut Rax::<u64, &str>::new();
//
            let sw = Stopwatch::start_new();

            for x in 0..1000000 {
                r.insert(x, Box::new("")).expect("whoops!");
            }

            println!("Thing took {}ms", sw.elapsed_ms());
            println!("Size {}", r.size());
        }
    }

    #[test]
    fn bench_rax_insert_show() {
        let r = &mut Rax::<u64, &str>::new();
//
        let sw = Stopwatch::start_new();

        for x in 0..1000 {
            r.insert(x, Box::new("")).expect("whoops!");
        }

        r.show();
        println!("Thing took {}ms", sw.elapsed_ms());
        println!("Size {}", r.size());
    }

    #[test]
    fn bench_rax_replace() {
        for _ in 0..10 {
            let mut r = &mut Rax::<u64, &str>::new();

            for x in 0..1000000 {
                r.insert(x, Box::new("")).expect("whoops!");
            }
//
            let sw = Stopwatch::start_new();

            for x in 0..1000000 {
                r.insert(x, Box::new("")).expect("whoops!");
            }

            println!("Thing took {}ms", sw.elapsed_ms());
            println!("Size {}", r.size());
        }
    }

    #[test]
    fn bench_tree_insert() {
        for _ in 0..10 {
            let mut r = &mut std::collections::BTreeMap::<u64, &str>::new();
//
            let sw = Stopwatch::start_new();

            for x in 0..1000000 {
                r.insert(x, "");
            }

            println!("Thing took {}ms", sw.elapsed_ms());
        }
    }

    #[test]
    fn bench_hashmap_insert() {
        for _ in 0..10 {
            let mut r = &mut std::collections::HashMap::<u64, &str>::new();
//
            let sw = Stopwatch::start_new();

            for x in 0..1000000 {
                r.insert(x, "");
            }

            println!("Thing took {}ms", sw.elapsed_ms());
            println!("Size {}", r.len());
        }
    }

    #[test]
    fn key_str() {
        let mut r = Rax::<&str, MyMsg>::new();

        let key = "hello-way";

        r.insert(
            key,
            Box::new(MyMsg("world 80")),
        ).expect("whoops!");
        r.insert(
            "hello-war",
            Box::new(MyMsg("world 80")),
        ).expect("whoops!");

        r.insert(
            "hello-wares",
            Box::new(MyMsg("world 80")),
        ).expect("whoops!");
        r.insert(
            "hello",
            Box::new(MyMsg("world 100")),
        ).expect("whoops!");

        {
            match r.find("hello") {
                Some(v) => println!("Found {}", v.0),
                None => println!("Not Found")
            }
        }

        r.show();

        r.first();
        while r.next() {
            println!("{}", r.key());
        }
        r.last();
        while r.prev() {
            println!("{}", r.key());
        }
    }

    #[test]
    fn key_f64() {
        println!("sizeof(Rax) {}", std::mem::size_of::<Rax<f64, MyMsg>>());

        let mut r = Rax::<f64, MyMsg>::new();

        r.insert(
            100.01,
            Box::new(MyMsg("world 100")),
        ).expect("whoops!");
        r.insert(
            80.20,
            Box::new(MyMsg("world 80")),
        ).expect("whoops!");
        r.insert(
            100.00,
            Box::new(MyMsg("world 200")),
        ).expect("whoops!");
        r.insert(
            99.10,
            Box::new(MyMsg("world 1")),
        ).expect("whoops!");

        r.show();

        r.first();
        while r.next() {
            println!("{}", r.key());
        }
        r.last();
        while r.prev() {
            println!("{}", r.key());
        }
    }

    #[test]
    fn key_u64() {
        println!("sizeof(Rax) {}", std::mem::size_of::<Rax<u64, MyMsg>>());

        let mut r = Rax::<u64, MyMsg>::new();

        r.insert(
            100,
            Box::new(MyMsg("world 100")),
        ).expect("whoops!");
        r.insert(
            80,
            Box::new(MyMsg("world 80")),
        ).expect("whoops!");
        r.insert(
            200,
            Box::new(MyMsg("world 200")),
        ).expect("whoops!");
        r.insert(
            1,
            Box::new(MyMsg("world 1")),
        ).expect("whoops!");

        r.show();

        r.first();
        while r.next() {
            println!("{}", r.key());
        }
        r.last();
        while r.prev() {
            println!("{}", r.key());
        }
    }

    #[test]
    fn test_keyed() {
        let mut r = Rax::<sds::SDS, MyMsg>::new();

        r.insert(
            sds::SDS::new("hello"),
            Box::new(MyMsg("world x10")),
        ).expect("whoops!");

        match r.find(sds::SDS::new("hi")) {
            Some(v) => {
                println!("Found: {}", v.0);
            }
            None => {
                println!("Not Found");
            }
        };


        match r.find(sds::SDS::new("hello")) {
            Some(v) => {
                println!("Found: {}", v.0);
            }
            None => {
                println!("Not Found");
            }
        };

        r.remove(sds::SDS::new("hello"));
        r.insert(
            sds::SDS::new("hello"),
            Box::new(MyMsg("world x10")),
        ).expect("whoops!");
        r.insert(
            sds::SDS::new("hello-16"),
            Box::new(MyMsg("world x11")),
        ).expect("whoops!");
        r.insert(
            sds::SDS::new("hello-20"),
            Box::new(MyMsg("world x12")),
        ).expect("whoops!");
        r.insert(
            sds::SDS::new("hello-01"),
            Box::new(MyMsg("world x13")),
        ).expect("whoops!");

        r.show();

        r.first();
        while r.next() {
            println!("{}", r.key());
        }
        r.last();
        while r.prev() {
            println!("{}", r.key());
        }
    }
}