#[macro_export]
macro_rules! impl_index {
    (@$ty:ty[$from:ty] $out:ty {
        $($res:ident),+$(,)?
    }) => {
        pastey::paste! {
            impl Index<$from> for $ty {
                type Output = $out;

                fn index(&self, index: $from) -> &Self::Output {
                    match index {
                        $($from::$res => &self.[<$res:snake>],)+
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
                        $($from::$res => &mut self.[<$res:snake>],)+
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
        }
    };
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

#[macro_export]
macro_rules! impl_reducible {
    (#[$meta:meta] $type:ident $fty:ty { $($field:ident),* $(,)? }) => {
        pastey::paste! {
            #[$meta]
            pub struct $type {
                $(pub $field: $fty,)*
            }

            #[derive(Clone, Copy, Debug, PartialEq)]
            pub enum [<$type Action>] {
                $([<$field:camel>]($fty),)*
            }

            impl $crate::utils::ReduceApply for $type {
                type Action = [<$type Action>];

                fn apply(&mut self, action: Self::Action) {
                    match action {
                        $(Self::Action::[<$field:camel>](value) => self.$field = value,)*
                    }
                }
            }

            impl yew::Reducible for $type {
                type Action = [<$type Action>];

                fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
                    let mut new = *self;
                    <Self as $crate::utils::ReduceApply>::apply(&mut new, action);
                    std::rc::Rc::new(new)
                }
            }
        }
    };
    ($type:ident $fty:ty { $($field:ident),* $(,)? }) => {
        $crate::impl_reducible! {
            #[derive(Clone, Copy, Debug, Decode, Default, Encode, PartialEq)]
            $type $fty { $($field),* }
        }
    };
}
