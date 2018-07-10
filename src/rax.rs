use libc;
use std;
use error::RedError;
use std::ptr;

//#[derive(Clone)]
pub struct Rax<V> {
    rax: *mut rax,
    it: *mut raxIterator,
    marker: std::marker::PhantomData<V>,
}

impl<V> Rax<V> {
    pub fn new_without_iterator() -> Rax<V> {
        unsafe {
            let mut r = raxNew();
            Rax {
                rax: r,
                it: std::ptr::null_mut(),
                marker: std::marker::PhantomData,
            }
        }
    }
    pub fn new() -> Rax<V> {
        unsafe {
            let mut r = raxNew();
            Rax {
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

extern "C" fn free_callback_wrapper<V>(v: *mut libc::c_void) {
    unsafe {
        // Re-box it so it can drop it immediately after it leaves this scope.
        Box::from_raw(v as *mut V);
    }
}

//
impl<V> Drop for Rax<V> {
    fn drop(&mut self) {
        unsafe {
            // Cleanup RAX
            raxFreeWithCallback(self.rax, free_callback_wrapper::<V>);

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
        println!("Dropping RaxIterator");
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

pub type raxNodeCallback = extern "C" fn(v: *mut libc::c_void);

pub type free_callback = extern "C" fn(v: *mut libc::c_void);


#[allow(improper_ctypes)]
#[allow(non_snake_case)]
extern "C" {
    #[no_mangle]
    pub static raxNotFound: *mut u8;

//    fn raxNotFoundGet() -> *mut u8;

    fn raxIteratorFree(it: *mut raxIterator);

    fn raxIteratorSize() -> libc::c_int;

    fn raxNew() -> *mut rax;

    fn raxFree(rax: *mut rax);

    fn raxFreeWithCallback(rax: *mut rax, callback: free_callback);

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
    use rax::MyMsg;
    use rax::Rax;
    use std;
    use std::collections;

    fn create_map() {}

    #[test]
    fn it_works_2() {
        let mut r: ::rax::Rax<MyMsg> = Rax::new();
        let mut r2: ::rax::Rax<i32> = Rax::new();

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