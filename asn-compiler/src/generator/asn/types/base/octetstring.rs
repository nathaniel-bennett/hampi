//! Generator code for Base Type Asn1ResolvedOctetString

use proc_macro2::{Ident, TokenStream};
use quote::quote;

use crate::error::Error;

use crate::generator::Generator;
use crate::resolver::asn::structs::types::base::Asn1ResolvedOctetString;

impl Asn1ResolvedOctetString {
    pub(crate) fn generate(
        &self,
        name: &str,
        generator: &mut Generator,
    ) -> Result<TokenStream, Error> {
        let struct_name = generator.to_type_ident(name);

        let mut ty_attributes = quote! { type = "OCTET-STRING" };
        let vis = generator.get_visibility_tokens();

        if let Some(s) = self.size.as_ref() {
            let sz_attributes = self.size.as_ref().unwrap().get_ty_size_constraints_attrs();
            ty_attributes.extend(sz_attributes);

            let min = proc_macro2::Literal::i128_unsuffixed(s.root_values.min().unwrap_or(0));
            let max = proc_macro2::Literal::i128_unsuffixed(s.root_values.min().unwrap_or(128));
        
            let dir = generator.generate_derive_tokens(false); // TODO: check for lower bound and manually derive if so

            let struct_tokens = quote! {
                #dir
                #[asn(#ty_attributes)]
                #vis struct #struct_name(#vis Vec<u8>);

                impl entropic::Entropic for #struct_name {
                    #[inline]
                    fn from_entropy_source<'a, I: Iterator<Item = &'a u8>, E: EntropyScheme>(
                        source: &mut Source<'a, I, E>,
                    ) -> Result<Self, Error> {
                        let capped_max = std::cmp::min(#max, 16383);
                        let vec_len = source.get_bounded_len(#min..=capped_max)?;
                        let mut v = Vec::new();

                        for _ in 0..vec_len {
                            v.push(u8::from_entropy_source(source)?);
                        }
                        Ok(#struct_name(v))
                    }
                
                    #[inline]
                    fn to_entropy_sink<'a, I: Iterator<Item = &'a mut u8>, E: EntropyScheme>(
                        &self,
                        sink: &mut Sink<'a, I, E>,
                    ) -> Result<usize, Error> {
                        let capped_max = std::cmp::min(#max, 16383);
                        let mut length = 0;
                        length += sink.put_bounded_len(#min..=capped_max, self.0.len())?;
                        length += sink.put_slice(self.0.as_slice())?;

                        Ok(length)
                    }
                }
            };

            Ok(struct_tokens)       
        } else {
            let dir = generator.generate_derive_tokens(true); // TODO: check for lower bound and manually derive if so

            let struct_tokens = quote! {
                #dir
                #[asn(#ty_attributes)]
                #vis struct #struct_name(#vis Vec<u8>);
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
            generator.get_unique_name("OCTET STRING")
        } else {
            input.unwrap().to_string()
        };

        let item = self.generate(&unique_name, generator)?;
        generator.aux_items.push(item);

        Ok(generator.to_type_ident(&unique_name))
    }
}
