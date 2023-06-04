use std::{collections::HashSet};

use proc_macro::{TokenStream, Span};
use quote::ToTokens;
use syn::{Item, ItemConst};

#[allow(unused)]
pub struct MacroArgs{
    ident: syn::Ident,
    comma: syn::Token![,],
    reg: syn::LitStr
}

impl syn::parse::Parse for MacroArgs{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(
            MacroArgs { ident: input.parse()?, comma: input.parse()?, reg: input.parse()? }
        )
    }
}

lazy_static::lazy_static!{
    static ref ID_CONSTANTS : HashSet<String> = {
        let mut constants = HashSet::new();
    
        let file: syn::File = syn::parse_file(include_str!("bindings.rs")).unwrap();
        
        for item in file.items{
            if let Item::Const(ItemConst{ident, ..}) = item{
                // TODO: Add type check to make sure its an int
                constants.insert(ident.to_string());
            }
        }

        constants
    };
}

pub(crate) fn enum_from_sys_(args: MacroArgs)->TokenStream{
    let MacroArgs{
        ident,
        reg,
        ..
    } = args;

    let reg = reg.value();
    let Ok(reg) = regex::Regex::new(&reg) else{
        return syn::Error::new(Span::call_site().into(), &format!("Invalid regex : {reg}")).to_compile_error().into();
    };

    let it = ID_CONSTANTS.iter()
        .filter_map(|c|Some((c, reg.captures(c)?)))
        .filter_map(|(c, matc)|{
            let id_ident = syn::Ident::new(c, Span::call_site().into());
            let mut item_ident = matc.get(1)?.as_str().to_string();
            item_ident[0..1].make_ascii_uppercase();
            let item_ident = syn::Ident::new(&item_ident, Span::call_site().into());

            Some((id_ident, item_ident))
        });

    
    let uses = it.clone().map(|(id_ident, _)|{
        quote::quote!(
            #id_ident,
        )
    });

    let variants = it.clone().map(|(id_ident, item_ident)|{
        quote::quote!(
            #item_ident = #id_ident as isize,
        )
    });

    let from_u32_matches = it.clone().map(|(id_ident, item_ident)|{
        quote::quote!(
            #id_ident => Self::#item_ident,
        )
    });

    let into_u32_matches = it.map(|(id_ident, item_ident)|{
        quote::quote!(
            Self::#item_ident => #id_ident,
        )
    });
    
    let mod_ident = syn::Ident::new(&format!("{}_generated", ident.to_string().to_lowercase()), Span::call_site().into());

    quote::quote!(
        mod #mod_ident{
            use spa_sys::{
                #(#uses)*
            };


            pub enum #ident{
                #(#variants)*
            }

            impl From<u32> for #ident {
                fn from(value: u32) -> Self {
                    match value {
                        #(#from_u32_matches)*
                        _ => panic!("invalid id")
                    }
                }
            }

            
            impl Into<u32> for #ident {
                fn into(self) -> u32 {
                    match self {
                        #(#into_u32_matches)*
                    }
                }
            }

        }

        pub use #mod_ident::#ident;
    ).into_token_stream().into()
}