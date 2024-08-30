extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, DeriveInput, Lit, Meta, NestedMeta};

fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize = false;
    for (i, c) in s.char_indices() {
        if i == 0 {
            result.push(c.to_lowercase().next().unwrap());
        } else if c.is_uppercase() {
            if !capitalize && !result.ends_with('_') {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
            capitalize = true;
        } else if capitalize {
            result.push(c.to_uppercase().next().unwrap());
            capitalize = false;
        } else {
            result.push(c);
        }
    }
    result
}

fn parse_attributes(attrs: &[Attribute]) -> (String, String) {
    attrs
        .iter()
        .filter(|attr| attr.path.is_ident("secret_string"))
        .filter_map(|attr| attr.parse_meta().ok())
        .filter_map(|meta| match meta {
            Meta::List(meta_list) => Some(meta_list.nested),
            _ => None,
        })
        .flatten()
        .filter_map(|nested| match nested {
            NestedMeta::Meta(Meta::NameValue(nv)) => Some(nv),
            _ => None,
        })
        .fold(
            (String::new(), String::new()),
            |(mut struct_name, mut field_name), nv| {
                match (nv.path.get_ident(), &nv.lit) {
                    (Some(ident), Lit::Str(lit_str)) if ident == "struct_name" => {
                        struct_name = lit_str.value();
                    }
                    (Some(ident), Lit::Str(lit_str)) if ident == "field_name" => {
                        field_name = lit_str.value();
                    }
                    _ => {}
                }
                (struct_name, field_name)
            },
        )
}

#[proc_macro_derive(SecretString, attributes(secret_string))]
pub fn secret_string_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let name_string = name.to_string();

    let (custom_struct_name, custom_field_name) = parse_attributes(&input.attrs);
    let field_name = if custom_field_name.is_empty() {
        custom_field_name
    } else {
        to_camel_case(&name_string)
    };
    let struct_name = if custom_struct_name.is_empty() {
        custom_struct_name
    } else {
        name_string
    };

    let expanded = quote! {
        impl #name {
            pub fn expose_secret_string(&self) -> String {
                self.0.expose_secret().to_string().clone()
            }
        }

        impl PartialEq for #name {
            fn eq(&self, other: &Self) -> bool {
                self.0.expose_secret() == other.0.expose_secret()
            }
        }

        impl Serialize for #name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let mut state = serializer.serialize_struct(stringify!(#struct_name), 1)?;
                state.serialize_field(stringify!(#field_name), self.0.expose_secret())?;
                state.end()
            }
        }

        impl AsRef<Secret<String>> for #name {
            fn as_ref(&self) -> &Secret<String> {
                &self.0
            }
        }
    };

    TokenStream::from(expanded)
}
