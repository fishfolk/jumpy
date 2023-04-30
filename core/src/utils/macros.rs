/// Macro to "derive" ( not really a derive macro ) SystemParam for a struct.
#[macro_export]
macro_rules! impl_system_param {
    (
        pub struct $t:ident<'a> {
            $(
                $( #[$attrs:meta] )*
                $f_name:ident: $f_ty:ty
            ),*
            $(,)?
        }
    ) => {
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

            fn initialize(world: &mut World) {
                $(
                    <$f_ty as SystemParam>::initialize(world);
                )*
            }

            fn get_state(world: &World) -> Self::State {
                (
                    $(
                        <$f_ty as SystemParam>::get_state(world)
                    ),*
                )
            }

            fn borrow(state: &mut Self::State) -> Self::Param<'_> {
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
                        <$f_ty as SystemParam>::borrow($f_name)
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
