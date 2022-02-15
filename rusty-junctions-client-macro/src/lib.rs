pub use rusty_junctions_client_proc_macro::junction;

#[macro_export]
macro_rules! when {
    ( $junction:ident; $initial_channel:ident, $( $other_channels:ident ),* ) => {
        $junction.when(&$initial_channel)$( .and(&$other_channels) )*
    }
}

#[macro_export]
macro_rules! channel_def {
    (Send, $junction:ident, $name:ident, $type:ty) => {
        let $name = $junction.send_channel::<$type>();
    };
    (Recv, $junction:ident, $name:ident, $type:ty) => {
        let $name = $junction.recv_channel::<$type>();
    };
    (Bidir, $junction:ident, $name:ident, $type:ty) => {
        let $name = $junction.bidir_channel::<$type>();
    };
}

// TODO: Consider using a TT-Muncher pattern
// This should prevent the user having to import the
// other nested macros into their crate.
#[macro_export]
macro_rules! junction_dec {
    ( $( $channel_name:ident as $mode:ident :: $type:ty),* $(,)+
      $( | $( $args:ident ),* | $function_block:block  ),* $(,)+ ) => {{
        let mut j = rusty_junctions::Junction::new();

        $( channel_def!($mode, j, $channel_name, $type); )*

        $(
            when!(j; $( $args ),* ).then_do(| $( $args ),* | $function_block );
        )*

        let controller_handle = j
            .controller_handle()
            .expect("Failed to get controller handle");

        ( $( $channel_name ),* , controller_handle)
    }};
}
