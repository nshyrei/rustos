use core::cell;
use core::ops;

/// Creates a public static mut field with initialization code. Based on crate `lazy-statics`, but without the lazy part.
/// # Thread safety
///  Plain types are unsafe, to have a thread safe field wrap the value into type that implements some form of mutex
/// # Usage example
/// global_fields! {aaaaaaaaaaaaaaaaaaaa
//    A: u64 = 0;
//    B: bool = false;
// }
#[macro_export]
macro_rules! global_fields {
    ($(#[$attribute : meta])* $id : ident : $varType:ty = $varInitCode : expr ; $($tail : tt)*) => {

        $(#[$attribute])*
        pub static mut $id : Accessor<$varType> = Accessor::new();

        impl Accessor<$varType> {
            fn get(&self) -> &mut $varType {
                unsafe {
                    if ((*self.value.as_ptr()).is_none()) {
                        self.value.replace(Some($varInitCode));
                    }

                    ((*self.value.as_ptr()).as_mut().unwrap())
                }
            }
        }

        impl ops::Deref for Accessor<$varType> {
            type Target = $varType;

            fn deref(&self) -> & $varType {
                unsafe { self.get() }
            }
        }

        impl ops::DerefMut for Accessor<$varType> {
            fn deref_mut(&mut self) -> &mut $varType {
                unsafe { self.get() }
            }
        }

        global_fields!($($tail)*);
    };
    () => ()
}
