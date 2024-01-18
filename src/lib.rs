//!# Lazy non-const Statics
//!This crate provides a simple global allocation item to store items as statics that arent
//!necessarily constant. "Global" is a bit of a misnomer, as these are more accurately described as
//!immutable non-constant statics.
//!
//![Global] dereferences to `T`, so it can be treated as a wrapper that allows any type to be static.
//!# Usage
//![Global] stores a function pointer that produces `T`. On the first deref call, this value will be
//!produced and allocated on the heap for the lifetime of the program. Subsequent calls will return
//!this cached value.
//!```rust
//!use global::Global;
//!
//!static MY_NUM: Global<i32> = Global::new(|| 5);
//!
//!fn main() {
//!    assert_eq!(*MY_NUM + 5, 10);
//!}
//!```
//!# Limitations
//!The biggest limitation is the double-pointer indirection that arises from storing a type that
//!itself allocates memory, such as [Vec] or [Box]. It also isn't possible to store DSTs, as the
//!data needs to be returned from a function on the stack. This could be fixed with a type that
//!offsets its allocation onto the producing closure, however types like [Vec] that require extra
//!information would still not work with this system.
//!
//!You may also want statics to simply be initalized on program startup,
//!which can be done with crates like ctor.
use std::{ops::Deref, sync::OnceLock};

#[cfg(feature = "ctor")]
pub use ctor;

#[cfg(feature = "ctor")]
#[macro_export]
macro_rules! ctor_static {
    ($($name:ident: $type: ty = $init:block);*;) => {
        $(static $name: Global<$type> = Global::new(|| $init););*
        
        #[crate::ctor::ctor]
        fn _global_init() {
            $($name.init());*
        }
    };
}



///Lazily evaluated static allocation.
pub struct Global<T> {
    f: fn() -> T,
    data: OnceLock<SendPtr<T>>
}

struct SendPtr<T>(pub *const T);
unsafe impl<T> Send for SendPtr<T> {}
unsafe impl<T> Sync for SendPtr<T> {}

impl<T> Deref for SendPtr<T> {
    type Target = *const T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


impl<T> Global<T> {
    ///Constructs a new global.
    ///Rather than a value, this function takes a closure that produces a value.
    ///```rust
    ///use global::Global;
    ///
    ///static MY_TABLE: Global<Vec<&str>> = Global::new(|| vec!["a", "b", "c"]);
    pub const fn new(f: fn() -> T) -> Self {
        Self { f, data: OnceLock::new() }
    }

    ///Initializes the contents of a global. Does nothing if already initialized.
    pub fn init(&self) {
        if let None = self.data.get() { 
            let _ = unsafe { self.alloc() }; 
        }
    }

    ///Retrieves a reference to the value inside the global without allocating.
    ///This function will return `None` if the global has not been allocated.
    pub fn get(&self) -> Option<&T> {
        self.data.get().map(|ptr| {unsafe { &***ptr }})
    }

    ///Retrieves a reference to the value inside the global without allocating. Calling this function on
    ///an unallocated global is undefined behavior.
    pub unsafe fn get_unchecked(&self) -> &T {
        //lol
        &***self.data.get().unwrap_unchecked()
    } 
    
    ///Caller must ensure cell has not been already allocated
    unsafe fn alloc(&self) -> *const T {
        //box will panic if it cannot allocate
        let ptr = Box::leak(
            Box::new((self.f)())
            ) as *const T;
        self.data.set(SendPtr(ptr)).unwrap_unchecked();
        **self.data.get().unwrap_unchecked()
    }
}


impl<T> Deref for Global<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self.data.get() {
            Some(v) => unsafe { &***v },
            None => unsafe { &*self.alloc() },
        }
    }
}

static TEST: Global<u8> = Global::new(|| 5);

#[cfg(test)]
mod tests {
    use std::ops::Add;

    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(TEST.add(1), 6);
        assert_eq!(*TEST, 5);
    }

    #[test]
    #[cfg(feature = "ctor")]
    fn ctor_test() {
        ctor_static! { 
            THING: u32 = { 5 };
        };

        assert_eq!(THING.add(1), 6);
        assert_eq!(*THING, 5);
    } 
}
