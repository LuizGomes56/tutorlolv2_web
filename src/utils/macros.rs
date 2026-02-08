#[macro_export]
macro_rules! impl_index {
    ($ty:ty[$from:ty] $out:ty {
        $($arm:pat => $res:ident),+$(,)?
    }) => {
        impl Index<$from> for $ty {
            type Output = $out;

            fn index(&self, index: $from) -> &Self::Output {
                match index {
                    $($arm => &self.$res,)+
                    #[allow(unreachable_patterns)]
                    _ => panic!(concat!(
                        stringify!($ty),
                        "[",
                        stringify!($from),
                        "] was called with an invalid index"
                    ))
                }
            }
        }

        impl IndexMut<$from> for $ty {
            fn index_mut(&mut self, index: $from) -> &mut Self::Output {
                match index {
                    $($arm => &mut self.$res,)+
                    #[allow(unreachable_patterns)]
                    _ => panic!(concat!(
                        stringify!($ty),
                        "[",
                        stringify!($from),
                        "] was called with an invalid index"
                    ))
                }
            }
        }
    };
}
