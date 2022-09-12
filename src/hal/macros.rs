/// Define the Chip8 HAL, Generic structs and Mock structs
macro_rules! chip8_hal {
   (
       $(
           impl $prop: ident
           where $($bound_type: ident: $bound_trait: ident),+
           {
                type $type: ident;
                struct $struct: ident;

                $(#[$hardware_doc: meta])*
                trait $trait: ident {
                    $(
                        $(#[$method_meta: meta])*
                        fn $method: ident $(<
                            $($gen_type: ident: $gen_bound: ident),+
                        >)? (
                            &mut self $(, $($arg: ident: $arg_type: ty),+ )?
                        ) -> $ret: ty;
                    )+
                }

                $(#[$mock_meta: meta])*
                struct $mock_struct: ident {
                    $(pub $mock_arg: ident: $mock_arg_type: ty
                        = $mock_arg_default: expr;)*

                    $(impl {
                        $($mock_impl: item)+
                    })?

                    impl trait {
                        $($mock_trait_impl: item)+
                    }
                }
           }
       )+
   ) => {
        /// HardwareExt trait docs
        pub trait HardwareExt
        where
            $(Self::$type: $trait<Error = Self::Error>),+
        {
            type Error;
            $(type $type;)+

            fn hardware(&mut self) -> Hardware<'_, $(Self::$type),+>;

            $(
            $(#[$hardware_doc])*
            fn $prop(&mut self) -> &mut Self::$type {
                self.hardware().$prop
            }
            )+
        }

        /// Hardware struct docs
        pub struct Hardware<'a, $($type),+>
        where $($type: $trait),+
        {
            $(pub $prop: &'a mut $type),+
        }

        /// GenericHardwareError docs
        pub enum GenericHardwareError<$($type),+>
        where $($type: $trait),+ {
            $($type($type::Error)),+
        }

        /// GenericHardware docs
        pub struct GenericHardware<$($type),+>
        where $($type: $trait),+
        {
            $($prop: $struct<$($bound_type),+>),+
        }

        impl<$($type),+> GenericHardware<$($type),+>
        where $($type: $trait),+
        {
            pub fn new($($prop: $type),+) -> Self {
                Self {
                    $(
                        $prop: $struct {
                            inner: $prop,
                            _marker: (
                                $(core::marker::PhantomData::<$bound_type>),+
                            ),
                        }
                    ),+
                }
            }
        }

        impl<$($type),+> HardwareExt for GenericHardware<$($type),+>
        where $($type: $trait),+
        {
            type Error = GenericHardwareError<$($type),+>;
            $(
                type $type = $struct<$($bound_type),+>;
            )+

            fn hardware(&mut self) -> Hardware<'_, $(Self::$type),+> {
                let GenericHardware { $($prop),+ } = self;
                Hardware { $($prop),+ }
            }
        }

        /// MockHardware docs
        #[cfg(test)]
        pub struct MockHardware {
            $($prop: $mock_struct),+
        }

        #[cfg(test)]
        impl core::default::Default for MockHardware {
            fn default() -> Self {
                Self { $($prop: $mock_struct::default()),+ }
            }
        }

        #[cfg(test)]
        impl MockHardware {
            pub fn new() -> Self {
                Self::default()
            }
        }

        #[cfg(test)]
        impl HardwareExt for MockHardware {
            type Error = ();
            $(type $type = $mock_struct;)+

            fn hardware(&mut self) -> Hardware<'_, $($mock_struct),+> {
                let MockHardware { $($prop),+ } = self;
                Hardware { $($prop),+ }
            }
        }

        // For each type of hardware:
        $(
            // Create the trait

            $(#[$hardware_doc])*
            pub trait $trait {
                type Error;
                $(
                    $(#[$method_meta])*
                    fn $method $( <$($gen_type: $gen_bound),+> )? (
                        &mut self $(, $($arg: $arg_type),+ )?
                    ) -> Result<$ret, Self::Error>;
                )+
            }


            // Create the wrapper struct

            $(#[$hardware_doc])*
            pub struct $struct<$($bound_type: $bound_trait),+> {
                inner: $type,
                _marker: ( $(core::marker::PhantomData<$bound_type>),+ )
            }

            impl<$($bound_type),+> $trait for $struct<$($bound_type),+>
            where $($bound_type: $bound_trait),+ {
                type Error = GenericHardwareError<$($bound_type),+>;

                $(
                    fn $method $( <$($gen_type: $gen_bound),+> )? (
                        &mut self $(, $($arg: $arg_type),+ )?
                    ) -> Result<$ret, Self::Error> {
                        self.inner.$method( $($($arg),+)? ).map_err(
                            GenericHardwareError::$type
                        )
                    }
                )+
            }

            impl<$($bound_type),+> core::ops::Deref for $struct<$($bound_type),+>
            where $($bound_type: $bound_trait),+ {
                type Target = $type;
                fn deref(&self) -> &Self::Target {
                    &self.inner
                }
            }

            impl<$($bound_type),+> core::ops::DerefMut for $struct<$($bound_type),+>
            where $($bound_type: $bound_trait),+ {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.inner
                }
            }


            // Create the mock struct

            #[cfg(test)]
            $(#[$mock_meta])*
            pub struct $mock_struct {
                $(pub $mock_arg: $mock_arg_type),*
            }

            #[cfg(test)]
            impl $mock_struct {
                pub fn new($($mock_arg: $mock_arg_type),*) -> Self {
                    Self { $($mock_arg),* }
                }

                $($($mock_impl)+)?
            }

            #[cfg(test)]
            impl core::default::Default for $mock_struct {
                fn default() -> Self {
                    Self { $($mock_arg: $mock_arg_default),* }
                }
            }

            #[cfg(test)]
            impl $trait for $mock_struct {
                type Error = ();
                $($mock_trait_impl)+
            }
       )+

   };
}

pub(super) use chip8_hal;
