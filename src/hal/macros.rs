/// Define the Chip8 hardware elements, and create a generic
/// hardware struct and error-type.
macro_rules! hal {
   (
       $(
           $(#[$hardware_doc: meta])*
           impl $prop: ident
           where $($bound_type: ident: $bound_trait: ident),+
           {
               type $type: ident;
               trait $trait: ident;
               struct $struct: ident;
               $(
                   $(#[$method_meta: meta])*
                   fn $method: ident $(<
                       $($gen_type: ident: $gen_bound: ident),+
                   >)? (
                       &mut self $(, $($arg: ident: $arg_type: ty),+ )?
                   ) -> $ret: ty;
               )+
           };
       )+
   ) => {
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

       pub struct Hardware<'a, $($type),+>
       where $($type: $trait),+
       {
           $(pub $prop: &'a mut $type),+
       }

       pub enum GenericHardwareError<$($type),+>
       where $($type: $trait),+ {
           $($type($type::Error)),+
       }

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

       $(
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
       )+

   };
}

pub(crate) use hal;
