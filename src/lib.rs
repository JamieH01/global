#![doc = include_str!("../README.md")]

#![cfg_attr(docsrs, feature(doc_cfg))]
use std::{ops::Deref, sync::OnceLock, fmt::{Debug, Display}};


#[cfg_attr(docsrs, doc(cfg(feature = "ctor")))]
#[cfg(feature = "ctor")]
pub use ctor;

#[cfg_attr(docsrs, doc(cfg(feature = "ctor")))]
#[cfg(feature = "ctor")]
#[macro_export]
/// Generate a static with a ctor procedure.
/// 
///```rust
///# use global_static::ctor_static;
///fn spit_a_number() -> i32 { 42 }
///
///ctor_static! {
///    pub MY_NUM: i32 = { 5 };
///    MY_OTHER_NUM: i32 = spit_a_number;
///    pub default DEFAULT_NUM: i32;
///};
///```
///This code will expand to the following:
///```rust
///# use global_static::*;
///# fn spit_a_number() -> i32 { 42 }
///pub static MY_NUM: Global<i32> = Global::new(|| { 5 });
///static MY_OTHER_NUM: Global<i32> = Global::new(spit_a_number);
///pub static DEFAULT_NUM: Global<i32> = Global::default();
///
///#[global_static::ctor::ctor]
///fn _global_init() {
///    MY_NUM.init();
///    MY_OTHER_NUM.init();
///    DEFAULT_NUM.init();
///}
///```
macro_rules! ctor_static {
    () => {};
    ($($body:tt)*) => {
        $crate::ctor_gen_defs!($($body)*);
        #[$crate::ctor::ctor]
        fn _global_init() {
            $crate::ctor_gen_inits!($($body)*);
        }
    };
}

///Internal macro. Do not use.
#[macro_export]
#[doc(hidden)]
macro_rules! ctor_gen_defs {
    () => {};

    ($name:ident: $type: ty = $init:block; $($tail:tt)*) => {
        static $name: $crate::Global<$type> = $crate::Global::new(|| $init);
        $crate::ctor_gen_defs!($($tail)*);
    };
    (pub $name:ident: $type: ty = $init:block; $($tail:tt)*) => {
        pub static $name: $crate::Global<$type> = $crate::Global::new(|| $init);
        $crate::ctor_gen_defs!($($tail)*);
    };

    ($name:ident: $type: ty = $init:expr; $($tail:tt)*) => {
        static $name: $crate::Global<$type> = $crate::Global::new($init);
        $crate::ctor_gen_defs!($($tail)*);
    };
    (pub $name:ident: $type: ty = $init:expr; $($tail:tt)*) => {
        pub static $name: $crate::Global<$type> = $crate::Global::new($init);
        $crate::ctor_gen_defs!($($tail)*);
    };

    (default $name:ident: $type: ty; $($tail:tt)*) => {
        static $name: $crate::Global<$type> = $crate::Global::default();
        $crate::ctor_gen_defs!($($tail)*);
    };
    (pub default $name:ident: $type: ty; $($tail:tt)*) => {
        pub static $name: $crate::Global<$type> = $crate::Global::default();
        $crate::ctor_gen_defs!($($tail)*);
    };

}

///Internal macro. Do not use.
#[macro_export]
#[doc(hidden)]
macro_rules! ctor_gen_inits {
    () => {};
    ($name:ident: $type: ty = $init:block; $($tail:tt)*) => {
        $name.init();
        $crate::ctor_gen_inits!($($tail)*);
    };
    (pub $name:ident: $type: ty = $init:block; $($tail:tt)*) => {
        $name.init();
        $crate::ctor_gen_inits!($($tail)*);
    };

    ($name:ident: $type: ty = $init:expr; $($tail:tt)*) => {
        $name.init();
        $crate::ctor_gen_inits!($($tail)*);
    };
    (pub $name:ident: $type: ty = $init:expr; $($tail:tt)*) => {
        $name.init();
        $crate::ctor_gen_inits!($($tail)*);
    };

    (default $name:ident: $type: ty; $($tail:tt)*) => {
        $name.init();
        $crate::ctor_gen_inits!($($tail)*);
    };
    (pub default $name:ident: $type: ty; $($tail:tt)*) => {
        $name.init();
        $crate::ctor_gen_inits!($($tail)*);
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
    ///# use global_static::Global;
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

impl<T: Default> Global<T> {
    ///Constructs a new global, using the [`Default`] implementation for `T` as the initializer.
    //cant use trait cus not const
    pub const fn default() -> Self {
        Self::new(T::default)
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

impl<T: Debug> Debug for Global<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self.deref())
    }
}
impl<T: Display> Display for Global<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.deref())
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
            pub THING2: u32 = { 5 };
        };

        assert_eq!(THING.add(1), 6);
        assert_eq!(*THING, 5);
    } 
}
