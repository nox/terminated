#![cfg_attr(all(feature = "enable", terminated_unstable), feature(proc_macro))]

#[cfg(all(feature = "enable", terminated_unstable))]
extern crate proc_macro;
#[cfg(all(feature = "enable", terminated_unstable))]
extern crate syn;

#[cfg(all(feature = "enable", terminated_unstable))]
mod macros;

#[cfg(all(feature = "enable", terminated_unstable))]
#[proc_macro]
pub fn ntstr(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    macros::ntstr(input)
}
