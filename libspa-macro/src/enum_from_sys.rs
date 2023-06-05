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
    static ref ID_CONSTANTS : HashSet<(String,String)> = {
        let mut constants = HashSet::new();
    
        let file: syn::File = syn::parse_file(include_str!("bindings.rs")).unwrap();
        
        for item in file.items{
            if let Item::Const(ItemConst{ident, expr, ..}) = item{
                // TODO: Add type check to make sure its an int
                constants.insert((ident.to_string(), expr.to_token_stream().to_string()));
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

    let mut values = HashSet::new();
    let constant = ID_CONSTANTS.iter()
        .filter_map(|(c_id, c_val)|Some((c_id, c_val, reg.captures(c_id)?)))
        .filter_map(|(c, c_val, matc)|{
            let id_ident = syn::Ident::new(c, Span::call_site().into());
            let mut item_ident = matc.get(1)?.as_str().to_string();
            item_ident[0..1].make_ascii_uppercase();
            let item_ident = syn::Ident::new(&item_ident, Span::call_site().into());

            Some((c_val, (id_ident, item_ident)))
        })
        .filter_map(|(val,res)|{
            if values.contains(val){
                return None;
            }
            values.insert(val.to_string());
            Some(res)
        })
        .collect::<Vec<_>>();

    
    let uses = constant.iter().map(|(id_ident, _)|{
        quote::quote!(
            #id_ident,
        )
    });

    let variants = constant.iter().map(|(id_ident, item_ident)|{
        quote::quote!(
            #item_ident = #id_ident as isize,
        )
    });

    let from_u32_matches = constant.iter().map(|(id_ident, item_ident)|{
        quote::quote!(
            #id_ident => #ident::#item_ident,
        )
    });

    let into_u32_matches = constant.iter().map(|(id_ident, item_ident)|{
        quote::quote!(
            #ident::#item_ident => #id_ident,
        )
    });
    
    let mod_ident = syn::Ident::new(&format!("{}_generated", ident.to_string().to_lowercase()), Span::call_site().into());

    quote::quote!(
        mod #mod_ident{
            use crate::utils::Id;
            use crate::pod::{FixedSizedPod, PodSerialize, serialize};
            use spa_sys::{ 
                #(#uses)*
            };

            #[repr(isize)]
            #[derive(Debug, Copy, Clone, Eq, PartialEq)]
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

            impl Into<Id> for #ident {
                fn into(self) -> Id {
                    Id(self.into())
                }
            }

            impl From<Id> for #ident {
                fn from(value: Id) -> Self {
                    Self::from(value.0)
                }
            }

            impl From<&Id> for #ident {
                fn from(value: &Id) -> Self {
                    Self::from(value.0)
                }
            }

            impl super::IdEnum for #ident {}
        }

        pub use #mod_ident::#ident;
    ).into_token_stream().into()
}