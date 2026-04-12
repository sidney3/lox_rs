#[macro_export]
macro_rules! usize_id {
    ($name:ident) => {
        #[derive(Debug, Hash, Eq, PartialEq, Clone, Copy, PartialOrd, Ord)]
        pub struct $name(pub usize);

        impl From<$name> for usize {
            fn from(t: $name) -> usize {
                t.0
            }
        }
        impl From<usize> for $name {
            fn from(i: usize) -> $name {
                $name(i)
            }
        }
    };
}
