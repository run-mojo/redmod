use libc;

const SDS_TYPE_5: libc::c_int = 0;
const SDS_TYPE_8: libc::c_int = 1;
const SDS_TYPE_16: libc::c_int = 2;
const SDS_TYPE_32: libc::c_int = 3;
const SDS_TYPE_64: libc::c_int = 4;

//#[repr(C)]
//pub struct Sds;

pub type Sds = *mut libc::c_char;

//#[lang = "mut_ptr"]

//impl Drop for SDS {
//    fn drop(&mut self) {
//        unsafe { sds_free(*self) }
//    }
//}


/// Create a new sds string with the content specified by the 'init' pointer
/// and 'initlen'.
/// If NULL is used for 'init' the string is initialized with zero bytes.
/// If SDS_NOINIT is used, the buffer is left uninitialized;
///
/// The string is always null-termined (all the sds strings are, always) so
/// even if you create an sds string with:
///
/// mystring = sdsnewlen("abc",3);
///
/// You can print the string with printf() as there is an implicit \0 at the
/// end of the string. However the string is binary safe and can contain
/// \0 characters in the middle, as the length is stored in the sds header.
#[inline]
pub fn new_len(s: &str) -> Sds {
    unsafe { sdsnewlen(s.as_ptr(), s.len()) }
}

/// Create a new sds string starting from a null terminated C string.
#[inline]
pub fn new(s: &str) -> Sds {
//    unsafe { sdsnew(format!("{}\0", s).as_ptr()) }
    unsafe { sdsnewlen(s.as_ptr(), s.len()) }
}

#[inline]
pub fn from_long_long(value: libc::c_longlong) -> Sds {
    unsafe  { sdsfromlonglong(value) }
}

/// Create an empty (zero length) sds string. Even in this case the string
/// always has an implicit null term.
#[inline]
pub fn empty() -> Sds {
    unsafe { sdsempty() }
}

/// Free an sds string. No operation is performed if 's' is NULL.
#[inline]
pub fn free(s: Sds) {
    unsafe { sdsfree(s) }
}

/// Duplicate an sds string.
#[inline]
pub fn dup(s: Sds) -> Sds {
    unsafe { sdsdup(s) }
}

/// Modify an sds string in-place to make it empty (zero length).
/// However all the existing buffer is not discarded but set as free space
/// so that next append operations will not require allocations up to the
/// number of bytes previously available.
#[inline]
pub fn clear(s: Sds) {
    unsafe { sdsclear(s) }
}

/// Enlarge the free space at the end of the sds string so that the caller
/// is sure that after calling this function can overwrite up to addlen
/// bytes after the end of the string, plus one more byte for nul term.
///
/// Note: this does not change the *length* of the sds string as returned
/// by sdslen(), but only the free buffer space we have.
#[inline]
pub fn make_room_for(s: Sds, addlen: libc::size_t) -> Sds {
    unsafe { sdsMakeRoomFor(s, addlen) }
}

/// Increment the sds length and decrements the left free space at the
/// end of the string according to 'incr'. Also set the null term
/// in the new end of the string.
///
/// This function is used in order to fix the string length after the
/// user calls sdsMakeRoomFor(), writes something after the end of
/// the current string, and finally needs to set the new length.
///
/// Note: it is possible to use a negative increment in order to
/// right-trim the string.
///
/// Usage example:
///
/// Using sdsIncrLen() and sdsMakeRoomFor() it is possible to mount the
/// following schema, to cat bytes coming from the kernel to the end of an
/// sds string without copying into an intermediate buffer:
///
/// oldlen = sdslen(s);
/// s = sdsMakeRoomFor(s, BUFFER_SIZE);
/// nread = read(fd, s+oldlen, BUFFER_SIZE);
/// ... check for nread <= 0 and handle it ...
/// sdsIncrLen(s, nread);
#[inline]
pub fn incr_len(s: Sds, incr: libc::ssize_t) {
    unsafe { sdsIncrLen(s, incr) }
}

/// Compare two sds strings s1 and s2 with memcmp().
///
/// Return value:
///
///     positive if s1 > s2.
///     negative if s1 < s2.
///     0 if s1 and s2 are exactly the same binary string.
///
/// If two strings share exactly the same prefix, but one of the two has
/// additional characters, the longer string is considered to be greater than
/// the smaller one.
#[inline]
pub fn cmp(s1: Sds, s2: Sds) -> libc::c_int {
    unsafe { sdscmp(s1, s2) }
}

#[inline]
pub fn len(s: Sds) -> libc::size_t {
    unsafe { sds_getlen(s) }
}


extern "C" {
    #[no_mangle]
    pub static SDS_NOINIT: *mut libc::c_char;


    fn sdsnewlen(init: *const u8, initlen: libc::size_t) -> Sds;

    fn sdsnew(init: *const u8) -> Sds;

    fn sdsempty() -> Sds;

    fn sdsfree(s: Sds);

    fn sdsfromlonglong(value: libc::c_longlong) -> Sds;

    fn sdsdup(s: Sds) -> Sds;

    fn sdsclear(s: Sds);

    fn sdsMakeRoomFor(s: Sds, addlen: libc::size_t) -> Sds;

    fn sdsIncrLen(s: Sds, incr: libc::ssize_t);

    fn sdscmp(s1: Sds, s2: Sds) -> libc::c_int;

    fn sds_getlen(s: Sds) -> libc::size_t;
    fn sds_avail(s: Sds) -> libc::size_t;

//    fn sdsll2str(s: )
}

#[cfg(test)]
mod tests {
    use sds;
    use std;

    #[test]
    fn test_len() {
        let mut s = sds::new_len("hello");

//        unsafe {sds::sdslength(s);}
//        assert_eq!(sds::sds_len(s), 5);
    }
}