//! Generator code for Base Type Asn1ResolvedCharacterString

use proc_macro2::{Ident, TokenStream};
use quote::quote;

use crate::error::Error;

use crate::generator::Generator;
use crate::resolver::asn::structs::types::base::Asn1ResolvedCharacterString;

impl Asn1ResolvedCharacterString {
    pub(crate) fn generate(
        &self,
        name: &str,
        generator: &mut Generator,
    ) -> Result<TokenStream, Error> {
        let struct_name = generator.to_type_ident(name);
        let char_str_type: proc_macro2::TokenStream =
            format!("\"{}\"", self.str_type).parse().unwrap();

        let mut ty_attributes = quote! { type = #char_str_type };

        let vis = generator.get_visibility_tokens();

        if let Some(s) = self.size.as_ref() {
            let sz_attributes = s.get_ty_size_constraints_attrs();
            ty_attributes.extend(sz_attributes);

            let min = proc_macro2::Literal::i128_unsuffixed(s.root_values.min().unwrap());
            let max = proc_macro2::Literal::i128_unsuffixed(s.root_values.max().unwrap());

            let dir = generator.generate_derive_tokens(false); // TODO: make custom derive if size.is_some()

            let struct_tokens = quote! {
                #dir
                #[asn(#ty_attributes)]
                #vis struct #struct_name(#vis String);

                impl<'a> arbitrary::Arbitrary<'a> for #struct_name {
                    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
                        let str_length = std::cmp::max(#min, std::cmp::min(4, u.int_in_range(#min..=#max)?));
                        let mut s = String::new();
                        for _ in 0..str_length {
                            s.push(u.arbitrary::<char>()?);
                        }
                        Ok(#struct_name(s))
                    }
                }
            };

            Ok(struct_tokens)
        } else {
            let dir = generator.generate_derive_tokens(true); // TODO: make custom derive if size.is_some()

            let struct_tokens = quote! {
                #dir
                #[asn(#ty_attributes)]
                #vis struct #struct_name(#vis String);
            };
            Ok(struct_tokens)
        }
    }

    pub(crate) fn generate_ident_and_aux_type(
        &self,
        generator: &mut Generator,
        input: Option<&String>,
    ) -> Result<Ident, Error> {
        let unique_name = if input.is_none() {
            generator.get_unique_name("CharacterString")
        } else {
            input.unwrap().to_string()
        };

        let item = self.generate(&unique_name, generator)?;
        generator.aux_items.push(item);

        Ok(generator.to_type_ident(&unique_name))
    }
}
