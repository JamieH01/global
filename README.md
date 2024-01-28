# Lazy non-const Statics
This crate provides a simple global allocation item to store items as statics that arent
necessarily constant. "Global" is a bit of a misnomer, as these are more accurately described as
immutable non-constant statics.

`Global` dereferences to `T`, so it can be treated as a wrapper that allows any type to be static.
# Usage
`Global` stores a function pointer that produces `T`. On the first deref call, this value will be
produced and allocated on the heap for the lifetime of the program. Subsequent calls will return
this cached value.
```rust
use global_static::Global;

static MY_NUM: Global<i32> = Global::new(|| 5);

fn main() {
    assert_eq!(*MY_NUM + 5, 10);
}
```
## ctor Feature
If you use the `ctor` feature flag, a macro is provided to initalize a global on startup.
```rust
use global_static::ctor_static;

ctor_static! {
    MY_NUM: i32 = { 5 };
    MY_OTHER_NUM: i32 = { *MY_NUM * 2 };
};
```

## singleton Feature
Most usecases for `Global` involve a struct that is parsed and then placed in a static like so:
```rust,ignore
use global_static::Global;
pub static CONFIG: Global<Config> = Global::new(Default::default);

pub struct Config {
    name: String
}

impl Default for Config {
    fn default() -> Self { /* implementation */ }
}
```
The `singleton` attribute generates a static item for a given struct.
```rust,ignore
use global_static::singleton;
#[singleton] //generates the CONFIG static
pub struct Config {
    name: String
}
impl Default for Config {
    fn default() -> Self { /* implementation */ }
}
```

# Limitations
The biggest limitation is the double-pointer indirection that arises from storing a type that
itself allocates memory, such as `Vec` or `Box`. It also isn't possible to store DSTs, as the
data needs to be returned from a function on the stack. This could be fixed with a type that
offsets its allocation onto the producing closure, however types like `Vec` that require extra
information would still not work with this system.
