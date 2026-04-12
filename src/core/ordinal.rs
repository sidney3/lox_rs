use std::fmt::{Debug, Display};
use std::hash::Hash;

/// An ordinal describes a finite type with a unique integer
/// representation (and thus a _finite_ integer representation)
///
/// We use it to model our enumeration types (which are much
/// cheaper to pass around and describe than the full types)
pub trait Ordinal: Debug + Hash + Eq + Clone + Copy + PartialEq + Display {
    const COUNT: usize;
    fn ord(&self) -> usize;
}

#[macro_export]
macro_rules! ordinal_enum {
    ( $name:ident { $( $variant:ident ),* $(,)? } ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name {
            $( $variant ),*
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Debug::fmt(self, f)
            }
        }

        impl Ordinal for $name {
            const COUNT: usize = [ $( stringify!($variant) ),* ].len();

            fn ord(&self) -> usize {
                *self as usize
            }
        }
    }
}
