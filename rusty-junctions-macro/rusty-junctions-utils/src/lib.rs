mod module;
pub use crate::module::Module;

/// Macro for defining the channels that are part of the Join Pattern
#[macro_export]
macro_rules! when {
    ( $junction:ident; $initial_channel:ident, $( $other_channels:ident ),* ) => {
        $junction.when(&$initial_channel)$( .and(&$other_channels) )*
    }
}
