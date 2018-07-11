use error::RedError;
use libc;
use sds;
use sds::SDS;
use sds::Sds;
use std;
use std::ptr;

/// Redis has a beautiful Radix Tree implementation in ANSI C.
/// This brings it to Rust. Great effort went into trying this zero overhead.
/// If you catch something that could be better go ahead and share it.
pub trait RAX<K: RaxKey, V> {
    fn show(&self);

    fn find(&self, key: K) -> Option<&V>;

    fn insert(&mut self, mut key: K, data: Box<V>) -> Result<(libc::c_int, Option<Box<V>>), RedError>;

    fn remove(&mut self, key: K) -> (bool, Option<Box<V>>);
}

pub trait RAXIter<K: RaxKey, V>: RAX<K, V> {}


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

    ///
    /// Insert a new entry into the RAX
    ///
    pub fn insert(&mut self, mut key: K, data: Box<V>) -> Result<(i32, Option<Box<V>>), RedError> {
        unsafe {
            // Allocate a pointer to catch the old value.
            let mut old: &mut *mut u8 = &mut std::ptr::null_mut();

            // Leak the boxed value as we hand it over to Rax to keep track of.
            // These must be heap allocated unless we want to store sizeof(usize) or
            // less bytes, then the value can be the pointer.
            let mut value: &mut V = Box::leak(data);

            // Integer values require Big Endian to allow the Rax to fully optimize
            // storing them since it will be able to compress the prefixes especially
            // for 64/128bit numbers.
            key = key.encode();
            let (ptr, len) = key.as_ptr();

            let r = raxInsert(
                self.rax,
//                sds::new("hello") as *const u8,
//                sds::SDS::new_from_str("hello").0 as *const u8,
                // Grab a raw pointer to the key. Keys are most likely allocated
                // on the stack. The rax will keep it's own copy of the key so we
                // don't want to keep in in the heap twice and it exists in the
                // rax in it's compressed form.
                ptr,
//                &key as *const _ as *const u8,
//                key.len(),
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
    pub fn remove(&mut self, mut key: K) -> (bool, Option<Box<V>>) {
        unsafe {
            let mut old: &mut *mut u8 = &mut std::ptr::null_mut();
            key = key.encode();
            let (ptr, len) = key.as_ptr();

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
    pub fn find_exists(&self, mut key: K) -> (bool, Option<&V>) {
        unsafe {
            key = key.encode();
            let (ptr, len) = key.as_ptr();

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
                (true, Some(std::mem::transmute(value)))
            }
        }
    }

    ///
    ///
    ///
    pub fn find(&self, mut key: K) -> Option<&V> {
        unsafe {
            key = key.encode();
            let (ptr, len) = key.as_ptr();

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
    pub fn exists(&self, mut key: K) -> bool {
        unsafe {
            key = key.encode();
            let (ptr, len) = key.as_ptr();

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
            raxSeek(self.it, MIN.as_ptr(), std::ptr::null(), 0) == 1
        }
    }

    ///
    ///
    ///
    pub fn last(&self) -> bool {
        unsafe {
            raxSeek(self.it, MAX.as_ptr(), std::ptr::null(), 0) == 1
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
    pub fn key_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts((*self.it).key, (*self.it).key_len as usize) }
    }

    ///
    ///
    ///
    pub fn key_raw(&self) -> (*mut u8, usize) {
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
            let data: *mut libc::c_void = ((*self.it).data);
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
    fn as_ptr(&self) -> (*const u8, usize);

    fn encode(self) -> RHS;

    fn decode(ptr: *const u8, len: usize) -> RHS;
}

impl RaxKey for f32 {
    #[inline]
    fn as_ptr(&self) -> (*const u8, usize) {
        (self as *const _ as *const u8, std::mem::size_of::<f32>())
    }

    #[inline]
    fn encode(self) -> f32 {
        self
    }

    #[inline]
    fn decode(ptr: *const u8, len: usize) -> f32 {
        unsafe { (*(ptr as *mut [u8; std::mem::size_of::<f32>()] as *mut f32)) }
    }
}

impl RaxKey for f64 {
    #[inline]
    fn as_ptr(&self) -> (*const u8, usize) {
        (self as *const _ as *const u8, std::mem::size_of::<f64>())
    }

    #[inline]
    fn encode(self) -> f64 {
        self
    }

    #[inline]
    fn decode(ptr: *const u8, len: usize) -> f64 {
        unsafe { (*(ptr as *mut [u8; std::mem::size_of::<f64>()] as *mut f64)) }
    }
}

impl RaxKey for isize {
    #[inline]
    fn as_ptr(&self) -> (*const u8, usize) {
        (self as *const _ as *const u8, std::mem::size_of::<isize>())
    }

    #[inline]
    fn encode(self) -> isize {
        self.to_be()
    }

    #[inline]
    fn decode(ptr: *const u8, len: usize) -> isize {
        unsafe { isize::from_be((*(ptr as *mut [u8; std::mem::size_of::<isize>()] as *mut isize))) }
    }
}

impl RaxKey for usize {
    #[inline]
    fn as_ptr(&self) -> (*const u8, usize) {
        (self as *const _ as *const u8, std::mem::size_of::<usize>())
    }

    #[inline]
    fn encode(self) -> usize {
        self.to_be()
    }

    #[inline]
    fn decode(ptr: *const u8, len: usize) -> usize {
        unsafe { usize::from_be((*(ptr as *mut [u8; std::mem::size_of::<usize>()] as *mut usize))) }
    }
}

impl RaxKey for i16 {
    #[inline]
    fn as_ptr(&self) -> (*const u8, usize) {
        (self as *const _ as *const u8, 2)
    }

    #[inline]
    fn encode(self) -> i16 {
        self.to_be()
    }

    #[inline]
    fn decode(ptr: *const u8, len: usize) -> i16 {
        unsafe { i16::from_be((*(ptr as *mut [u8; 2] as *mut i16))) }
    }
}

impl RaxKey for u16 {
    #[inline]
    fn as_ptr(&self) -> (*const u8, usize) {
        (self as *const _ as *const u8, 2)
    }

    #[inline]
    fn encode(self) -> u16 {
        self.to_be()
    }

    #[inline]
    fn decode(ptr: *const u8, len: usize) -> u16 {
        unsafe { u16::from_be((*(ptr as *mut [u8; 2] as *mut u16))) }
    }
}

impl RaxKey for i32 {
    #[inline]
    fn as_ptr(&self) -> (*const u8, usize) {
        (self as *const _ as *const u8, 4)
    }

    #[inline]
    fn encode(self) -> i32 {
        self.to_be()
    }

    #[inline]
    fn decode(ptr: *const u8, len: usize) -> i32 {
        unsafe { i32::from_be((*(ptr as *mut [u8; 4] as *mut i32))) }
    }
}

impl RaxKey for u32 {
    #[inline]
    fn as_ptr(&self) -> (*const u8, usize) {
        (self as *const _ as *const u8, 4)
    }

    #[inline]
    fn encode(self) -> u32 {
        self.to_be()
    }

    #[inline]
    fn decode(ptr: *const u8, len: usize) -> u32 {
        unsafe { u32::from_be((*(ptr as *mut [u8; 4] as *mut u32))) }
    }
}

impl RaxKey for i64 {
    #[inline]
    fn as_ptr(&self) -> (*const u8, usize) {
        (self as *const _ as *const u8, 8)
    }

    #[inline]
    fn encode(self) -> i64 {
        self.to_be()
    }

    #[inline]
    fn decode(ptr: *const u8, len: usize) -> i64 {
        unsafe { i64::from_be((*(ptr as *mut [u8; 8] as *mut i64))) }
    }
}

impl RaxKey for u64 {
    #[inline]
    fn as_ptr(&self) -> (*const u8, usize) {
        (self as *const _ as *const u8, 8)
    }

    #[inline]
    fn encode(self) -> u64 {
        self.to_be()
    }

    #[inline]
    fn decode(ptr: *const u8, len: usize) -> u64 {
        unsafe { u64::from_be((*(ptr as *mut [u8; 8] as *mut u64))) }
    }
}

impl RaxKey for i128 {
    #[inline]
    fn as_ptr(&self) -> (*const u8, usize) {
        (self as *const _ as *const u8, 16)
    }

    #[inline]
    fn encode(self) -> i128 {
        self.to_be()
    }

    #[inline]
    fn decode(ptr: *const u8, len: usize) -> i128 {
        unsafe { i128::from_be((*(ptr as *mut [u8; 16] as *mut i128))) }
    }
}

impl RaxKey for u128 {
    #[inline]
    fn as_ptr(&self) -> (*const u8, usize) {
        (self as *const _ as *const u8, 16)
    }

    #[inline]
    fn encode(self) -> u128 {
        self.to_be()
    }

    #[inline]
    fn decode(ptr: *const u8, len: usize) -> u128 {
        unsafe { u128::from_be((*(ptr as *mut [u8; 16] as *mut u128))) }
    }
}

impl RaxKey for SDS {
    #[inline]
    fn as_ptr(&self) -> (*const u8, usize) {
        (self.as_ptr(), self.len())
    }

    #[inline]
    fn encode(self) -> SDS {
        self
    }

    #[inline]
    fn decode(ptr: *const u8, len: usize) -> SDS {
        SDS::new_from_ptr(ptr, len)
    }
}

impl<'a> RaxKey for &'a str {
    #[inline]
    fn as_ptr(&self) -> (*const u8, usize) {
        ((*self).as_ptr(), self.len())
    }

    #[inline]
    fn encode(self) -> &'a str {
        self
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


//#[derive(Clone)]
pub struct RawRax<V> {
    rax: *mut rax,
    it: *mut raxIterator,
    marker: std::marker::PhantomData<V>,
}

impl<V> RawRax<V> {
    pub fn new_without_iterator() -> RawRax<V> {
        unsafe {
            let mut r = raxNew();
            RawRax {
                rax: r,
                it: std::ptr::null_mut(),
                marker: std::marker::PhantomData,
            }
        }
    }
    pub fn new() -> RawRax<V> {
        unsafe {
            let mut r = raxNew();
            RawRax {
                rax: r,
                it: raxIteratorNew(r),
                marker: std::marker::PhantomData,
            }
        }
    }

    pub fn size(&self) -> libc::uint64_t {
        unsafe { raxSize(self.rax) }
    }

    pub fn show(&self) {
        unsafe { raxShow(self.rax) }
    }

    pub fn insert_raw_val(&mut self, key: *const u8, key_len: usize, data: Box<V>) -> Result<(libc::c_int, Option<Box<V>>), RedError> {
        unsafe {
            let mut old: &mut *mut u8 = &mut std::ptr::null_mut();

            // Leak the boxed value as we hand it over to Rax to keep track of.
            let mut leaked: &mut V = Box::leak(data);

            let r = raxInsert(
                self.rax,
                key,
                key_len,
                leaked as *mut V as *mut u8,
                old,
            );

            if (old).is_null() {
                Ok((r, None))
            } else {
                // Box the previous value since Rax is done with it and it's our
                // responsibility now to drop it. Once this Box goes out of scope
                // the value is dropped and memory reclaimed.
                Ok((r, Some(Box::from_raw(*old as *mut V))))
            }
        }
    }

    pub fn insert_str(&mut self, key: &str, data: Box<V>) -> Result<(libc::c_int, Option<Box<V>>), RedError> {
        self.insert(key.as_ref(), data)
    }

    pub fn insert(&mut self, key: &[u8], data: Box<V>) -> Result<(libc::c_int, Option<Box<V>>), RedError> {
        unsafe {
            let mut old: &mut *mut u8 = &mut std::ptr::null_mut();

            // Leak the boxed value as we hand it over to Rax to keep track of.
            let mut leaked: &mut V = Box::leak(data);

            let r = raxInsert(
                self.rax,
                key.as_ptr(),
                key.len(),
                leaked as *mut V as *mut u8,
                old,
            );

            if (old).is_null() {
                Ok((r, None))
            } else {
                // Box the previous value since Rax is done with it and it's our
                // responsibility now to drop it. Once this Box goes out of scope
                // the value is dropped and memory reclaimed.
                Ok((r, Some(Box::from_raw(*old as *mut V))))
            }
        }
    }

    pub fn remove(&mut self, key: &[u8]) -> (bool, Option<Box<V>>) {
        unsafe {
            let mut old: &mut *mut u8 = &mut std::ptr::null_mut();

            let r = raxRemove(
                self.rax,
                key.as_ptr(),
                key.len(),
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
    pub fn find_exists(&self, key: &[u8]) -> (bool, Option<&V>) {
        unsafe {
            let value = raxFind(
                self.rax,
                key.as_ptr(),
                key.len(),
            );

            if value.is_null() {
                (true, None)
            } else if value == raxNotFound {
                (false, None)
            } else {
                // transmute to the value so we don't drop the actual value accidentally.
                // While the key associated to the value is in the RAX then we cannot
                // drop it.
                (true, Some(std::mem::transmute(value)))
            }
        }
    }

    pub fn find(&self, key: &[u8]) -> Option<&V> {
        unsafe {
            let value = raxFind(
                self.rax,
                key.as_ptr(),
                key.len(),
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

    pub fn iterator(&self) -> RaxIterator<V> {
        unsafe {
            RaxIterator {
                it: raxIteratorNew(self.rax),
                marker: std::marker::PhantomData,
            }
        }
    }

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
        unsafe { std::slice::from_raw_parts((*self.it).key, (*self.it).key_len as usize) }
    }

    pub fn data(&self) -> Option<&V> {
        unsafe {
            let data: *mut libc::c_void = ((*self.it).data);
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
impl<V> Drop for RawRax<V> {
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

pub struct RaxIterator<V> {
    //    rax: *mut Rax<V>,
//    it: *mut raxIterator,
    it: *mut raxIterator,
    //    it2: raxIterator,
    marker: std::marker::PhantomData<V>,
}

impl<V> RaxIterator<V> {
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
            let data: *mut libc::c_void = ((*self.it).data);
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

pub struct MyMsg<'a>(&'a str);

impl<'a> Drop for MyMsg<'a> {
    fn drop(&mut self) {
        println!("dropped -> {}", self.0);
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

pub type raxNodeCallback = extern "C" fn(v: *mut libc::c_void);

pub type rax_free_callback = extern "C" fn(v: *mut libc::c_void);


#[allow(improper_ctypes)]
#[allow(non_snake_case)]
extern "C" {
    #[no_mangle]
    pub static raxNotFound: *mut u8;

    fn raxIteratorFree(it: *mut raxIterator);

    fn raxIteratorSize() -> libc::c_int;

    fn raxNew() -> *mut rax;

    fn raxFree(rax: *mut rax);

    fn raxFreeWithCallback(rax: *mut rax, callback: rax_free_callback);

    fn raxInsert(rax: *mut rax,
                 s: *const u8,
                 len: libc::size_t,
                 data: *const u8,
//                 data: *mut Box<&[u8]>,
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


#[cfg(test)]
mod tests {
    use key;
    use rax;
    use rax::MyMsg;
    use rax::RawRax;
    use rax::Rax;
    use sds;
    use std;
    use std::collections;

    fn create_map() {}

    #[test]
    fn key_str() {
        let mut r = Rax::<&str, MyMsg>::new();
        let mut z: u64 = 1;

        r.insert(
            "hello-way",
            Box::new(MyMsg("world 80")),
        );
        r.insert(
            "hello-war",
            Box::new(MyMsg("world 80")),
        );

        r.insert(
            "hello-wares",
            Box::new(MyMsg("world 80")),
        );
        r.insert(
            "hello",
            Box::new(MyMsg("world 100")),
        );

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
        let mut z: u64 = 1;


        r.insert(
            100.01,
            Box::new(MyMsg("world 100")),
        );
        r.insert(
            80.20,
            Box::new(MyMsg("world 80")),
        );
        r.insert(
            100.00,
            Box::new(MyMsg("world 200")),
        );
        r.insert(
            99.10,
            Box::new(MyMsg("world 1")),
        );

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
    fn key_f64_as_u64() {
        println!("sizeof(Rax) {}", std::mem::size_of::<Rax<u64, MyMsg>>());

        let mut r = Rax::<u64, MyMsg>::new();
        let mut z: u64 = 1;

        r.insert(
            100.01_f64.to_bits(),
            Box::new(MyMsg("world 100")),
        );
        r.insert(
            80.20_f64.to_bits(),
            Box::new(MyMsg("world 80")),
        );
        r.insert(
            100.00_f64.to_bits(),
            Box::new(MyMsg("world 200")),
        );
        r.insert(
            99.10_f64.to_bits(),
            Box::new(MyMsg("world 1")),
        );

        r.show();

        r.first();
        while r.next() {
            println!("{}", f64::from_bits(r.key()));
        }
        r.last();
        while r.prev() {
            println!("{}", f64::from_bits(r.key()));
        }
    }


    #[test]
    fn key_u64() {
        println!("sizeof(Rax) {}", std::mem::size_of::<Rax<u64, MyMsg>>());

        let mut r = Rax::<u64, MyMsg>::new();
        let mut z: u64 = 1;


        r.insert(
            100,
            Box::new(MyMsg("world 100")),
        );
        r.insert(
            80,
            Box::new(MyMsg("world 80")),
        );
        r.insert(
            200,
            Box::new(MyMsg("world 200")),
        );
        r.insert(
            1,
            Box::new(MyMsg("world 1")),
        );

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
            sds::SDS::new_from_str("hello"),
            Box::new(MyMsg("world x10")),
        );

        match r.find(sds::SDS::new_from_str("hi")) {
            Some(v) => {
                println!("Found: {}", v.0);
            }
            None => {
                println!("Not Found");
            }
        };


        match r.find(sds::SDS::new_from_str("hello")) {
            Some(v) => {
                println!("Found: {}", v.0);
            }
            None => {
                println!("Not Found");
            }
        };

        r.remove(sds::SDS::new_from_str("hello"));
        r.insert(
            sds::SDS::new_from_str("hello"),
            Box::new(MyMsg("world x10")),
        );
        r.insert(
            sds::SDS::new_from_str("hello-16"),
            Box::new(MyMsg("world x11")),
        );
        r.insert(
            sds::SDS::new_from_str("hello-20"),
            Box::new(MyMsg("world x12")),
        );
        r.insert(
            sds::SDS::new_from_str("hello-01"),
            Box::new(MyMsg("world x13")),
        );

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
    fn it_works_2() {
        let mut r: ::rax::RawRax<MyMsg> = RawRax::new();
        let mut r2: ::rax::RawRax<i32> = RawRax::new();

        r2.insert("1".as_ref(), Box::new(1));
//        let mut map = collections::HashMap::new();
//        map.insert("hi", Box::new(MyMsg("world 2")));

        match r.insert("hi".as_ref(), Box::new(MyMsg("world 1"))) {
            Ok((result, previous)) => {
                match previous {
                    Some(value) => {
                        println!("PREV -> {}", value.0)
                    }
                    None => {
                        println!("New Record!!!")
                    }
                }
            }
            Err(_) => {}
        }

        match r.find("hi".as_ref()) {
            Some(v) => {
                println!("Found: {}", v.0);
            }
            None => {
                println!("Not Found");
            }
        };

        match r.find("hi".as_ref()) {
            Some(v) => {
                println!("Found: {}", v.0);
            }
            None => {
                println!("Not Found");
            }
        };

        match r.insert("hippies".as_ref(), Box::new(MyMsg("world 2"))) {
            Ok((result, previous)) => {
                match previous {
                    Some(value) => {
                        println!("PREV -> {}", value.0)
                    }
                    None => {
                        println!("New Record!!!")
                    }
                }
            }
            Err(_) => {}
        }

        match r.insert("hill".as_ref(), Box::new(MyMsg("world 3"))) {
            Ok((result, previous)) => {
                match previous {
                    Some(value) => {
                        println!("PREV -> {}", value.0)
                    }
                    None => {
                        println!("New Record!!!")
                    }
                }
            }
            Err(_) => {}
        }

        match r.insert("hi".as_ref(), Box::new(MyMsg("world 4"))) {
            Ok((result, previous)) => {
                match previous {
                    Some(value) => {
                        println!("PREV -> {}", value.0)
                    }
                    None => {
                        println!("New Record!!!")
                    }
                }
            }
            Err(_) => {}
        }

        match r.insert("hit".as_ref(), Box::new(MyMsg("world 5"))) {
            Ok((result, previous)) => {
                match previous {
                    Some(value) => {
                        println!("PREV -> {}", value.0)
                    }
                    None => {
                        println!("New Record!!!")
                    }
                }
            }
            Err(_) => {}
        }

        r.show();

        println!("First: {}", r.first());
        while r.next() {
            match r.data() {
                Some(v) => {
                    println!("next() -> {} - {}", std::str::from_utf8(r.key()).unwrap(), v.0);
                }
                None => {
                    println!("NO DATA!!!");
                }
            }
        }

        r.last();
        println!("now in reverse...");
        while r.prev() {
            match r.data() {
                Some(v) => {
                    println!("prev() -> {} - {}", std::str::from_utf8(r.key()).unwrap(), v.0);
                }
                None => {
                    println!("NO DATA!!!");
                }
            }
        }
        r.last();
        println!("now in reverse...");
        while r.prev() {
            match r.data() {
                Some(v) => {
                    println!("prev() -> {} - {}", std::str::from_utf8(r.key()).unwrap(), v.0);
                }
                None => {
                    println!("NO DATA!!!");
                }
            }
        }

        let (exists, value) = r.find_exists("h".as_ref());
        if !exists {
            println!("NOT EXISTS!!!");
        }

        match r.find("h".as_ref()) {
            Some(v) => {
                println!("Found: {}", v.0);
            }
            None => {
                println!("Not Found");
            }
        };
        println!("finished iterator");

//std::slice::from_raw_parts()
//        r.insert("hidfdsafsdafsaddaeewwefewfwe", &mut MyMsg("world"));
    }
}