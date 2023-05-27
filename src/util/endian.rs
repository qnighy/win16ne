use std::fmt;
use bytemuck::TransparentWrapper;

macro_rules! define_int {
    ($LT:ident, $BT:ident, $V:ident) => {
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, TransparentWrapper)]
        #[repr(transparent)]
        pub struct $LT {
            le_value: $V
        }

        impl $LT {
            pub fn new(value: $V) -> Self {
                $LT::from(value)
            }

            pub fn value(self) -> $V {
                $V::from(self)
            }
        }

        impl From<$V> for $LT {
            fn from(value: $V) -> Self {
                $LT {
                    le_value: value.to_le(),
                }
            }
        }

        impl From<$LT> for $V {
            fn from(wrapped: $LT) -> Self {
                $V::from_le(wrapped.le_value)
            }
        }

        impl fmt::Debug for $LT {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                self.value().fmt(f)
            }
        }

        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, TransparentWrapper)]
        #[repr(transparent)]
        pub struct $BT {
            be_value: $V
        }

        impl $BT {
            pub fn new(value: $V) -> Self {
                $BT::from(value)
            }

            pub fn value(self) -> $V {
                $V::from(self)
            }
        }

        impl From<$V> for $BT {
            fn from(value: $V) -> Self {
                $BT {
                    be_value: value.to_be(),
                }
            }
        }

        impl From<$BT> for $V {
            fn from(wrapped: $BT) -> Self {
                $V::from_be(wrapped.be_value)
            }
        }

        impl fmt::Debug for $BT {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                self.value().fmt(f)
            }
        }
    };
}

define_int!(Lu16, Bu16, u16);
define_int!(Lu32, Bu32, u32);
define_int!(Lu64, Bu64, u64);
define_int!(Lu128, Bu128, u128);
