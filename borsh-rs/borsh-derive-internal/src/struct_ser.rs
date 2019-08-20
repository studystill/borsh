use crate::attribute_helpers::contains_skip;
use quote::quote;
use syn::export::{Span, TokenStream2};
use syn::{Fields, Index, ItemStruct};

pub fn struct_ser(input: &ItemStruct) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let mut body = TokenStream2::new();
    match &input.fields {
        Fields::Named(fields) => {
            for field in &fields.named {
                if contains_skip(&field.attrs) {
                    continue;
                }
                let field_name = field.ident.as_ref().unwrap();
                let delta = quote! {
                    borsh::Serializable::write(&self.#field_name, writer)?;
                };
                body.extend(delta);
            }
        }
        Fields::Unnamed(fields) => {
            for field_idx in 0..fields.unnamed.len() {
                let field_idx = Index {
                    index: field_idx as u32,
                    span: Span::call_site(),
                };
                let delta = quote! {
                    borsh::Serializable::write(&self.#field_idx, writer)?;
                };
                body.extend(delta);
            }
        }
        Fields::Unit => {}
    }
    Ok(quote! {
        impl borsh::ser::Serializable for #name {
            fn write<W: std::io::Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
                #body
                Ok(())
            }
        }
    })
}

// Rustfmt removes comas.
#[rustfmt::skip]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing_tools::assert_eq;

    #[test]
    fn simple_struct() {
        let item_struct: ItemStruct = syn::parse2(quote!{
            struct A {
                x: u64,
                y: String,
            }
        }).unwrap();

        let actual = struct_ser(&item_struct).unwrap();
        let expected = quote!{
            impl borsh::ser::Serializable for A {
                fn write<W: std::io::Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
                    borsh::Serializable::write(&self.x, writer)?;
                    borsh::Serializable::write(&self.y, writer)?;
                    Ok(())
                }
            }
        };
        assert_eq(expected, actual);
    }
}