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

            impl tutorlolv2::yew::ReduceApply for $type {
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
                    <Self as tutorlolv2::yew::ReduceApply>::apply(&mut new, action);
                    std::rc::Rc::new(new)
                }
            }
        }
    };
}
