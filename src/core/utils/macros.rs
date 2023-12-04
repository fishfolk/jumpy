/// Macro to "derive" ( not really a derive macro ) SystemParam for a struct.
#[macro_export]
macro_rules! impl_system_param {
    (
        $( #[$m:meta] )*
        pub struct $t:ident<'a> {
            $(
                $( #[$attrs:meta] )*
                $f_name:ident: $f_ty:ty
            ),*
            $(,)?
        }
    ) => {
        $( #[$m] )*
        pub struct $t<'a> {
            $(
                $( #[$attrs] )*
                pub $f_name: $f_ty
            ),*
        }

        impl<'a> SystemParam for $t<'a> {
            type State = (
                $(
                    <$f_ty as SystemParam>::State
                ),*
            );
            type Param<'p> = $t<'p>;

            fn get_state(world: &World) -> Self::State {
                (
                    $(
                        <$f_ty as SystemParam>::get_state(world)
                    ),*
                )
            }

            fn borrow<'s>(world: &'s World, state: &'s mut Self::State) -> Self::Param<'s> {
                let (
                    $(
                        $f_name
                    ),*
                ) = state;
                let (
                    $(
                        $f_name
                    ),*
                ) = (
                    $(
                        <$f_ty as SystemParam>::borrow(world, $f_name)
                    ),*
                );

                Self::Param {
                    $(
                        $f_name
                    ),*
                }
            }
        }
    };
}
