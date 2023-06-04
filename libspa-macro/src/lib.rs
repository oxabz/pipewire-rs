mod enum_from_sys;

extern crate proc_macro;
use std::fs::File;
use std::io::Write;

use proc_macro::TokenStream;



#[proc_macro]
pub fn enum_from_sys(item: TokenStream) -> TokenStream {
    let  arg = syn::parse_macro_input!(item as enum_from_sys::MacroArgs);
    enum_from_sys::enum_from_sys_(arg)
}