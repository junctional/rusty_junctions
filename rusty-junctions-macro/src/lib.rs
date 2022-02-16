pub mod client {
    pub use client_api_macro::{channel_def, junction_dec, when};
    pub use client_api_proc_macro::junction;
}

pub mod library {
    pub use library_generation_proc_macro::{
        library_generate as generate, JoinPattern, PartialPattern, TerminalPartialPattern,
    };
}
