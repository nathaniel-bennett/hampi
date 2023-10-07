//! Implementation of Code Generation for `SEQUENCE OF` ASN Type

use proc_macro2::TokenStream;
use quote::quote;

use crate::error::Error;
use crate::generator::Generator;
use crate::resolver::asn::structs::types::{
    constructed::ResolvedConstructedType, Asn1ResolvedType,
};

impl ResolvedConstructedType {
    pub(crate) fn generate_sequence_of(
        &self,
        name: &str,
        generator: &mut Generator,
    ) -> Result<TokenStream, Error> {
        if let ResolvedConstructedType::SequenceOf {
            ref ty,
            ref size_values,
            ..
        } = self
        {
            let seq_of_type_ident = generator.to_type_ident(name);
            let input_type_name = format!("{}_Entry", name); // BUG: entry types are sometimes empty

            // TODO: put in here??
            let seq_of_type = Asn1ResolvedType::generate_name_maybe_aux_type(
                ty,
                generator,
                Some(&input_type_name),
            )?;

            let vis = generator.get_visibility_tokens();

            let mut ty_attrs = quote! { type = "SEQUENCE-OF" };
            if let Some(sz) = size_values.as_ref() {
                ty_attrs.extend(sz.get_ty_size_constraints_attrs());

                let min = proc_macro2::Literal::i128_unsuffixed(sz.root_values.min().unwrap_or(0));
                let max = proc_macro2::Literal::i128_unsuffixed(sz.root_values.max().unwrap_or(128));

                let dir = generator.generate_derive_tokens(false);

                // We intentionally cap SeqOf types at 10 instances; any number much higher tends to deplete entropy unduly, while any lower number may inhibit all necessary fields from being expressed
                Ok(quote! {
                    #dir
                    #[asn(#ty_attrs)]
                    #vis struct #seq_of_type_ident(#vis Vec<#seq_of_type>);

                    impl entropic::Entropic for #seq_of_type_ident {
                        fn from_finite_entropy<'a, S: EntropyScheme, I: Iterator<Item = &'a u8>>(
                            source: &mut entropic::FiniteEntropySource<'a, S, I>,
                        ) -> Result<Self, entropic::Error> {
                            let capped_max = std::cmp::min(#max, 10);
                            let vec_len = source.get_bounded_len(#min..=capped_max)?;
                            let mut v = Vec::new();
                            for _ in 0..vec_len {
                                v.push(#seq_of_type::from_finite_entropy(source)?);
                            }
                            Ok(#seq_of_type_ident(v))
                        }

                        fn to_finite_entropy<'a, S: EntropyScheme, I: Iterator<Item = &'a mut u8>>(
                            &self,
                            sink: &mut FiniteEntropySink<'a, S, I>,
                        ) -> Result<usize, Error> {
                            let mut length = 0;
                            let capped_max = std::cmp::min(#max, 10);
                            length += sink.put_bounded_len(#min..=capped_max, self.0.len())?;
                            for item in self.0.iter() {
                                length += sink.put_entropic(item)?;
                            }

                            Ok(length)
                        }
                    }
                })
            } else {
                let dir = generator.generate_derive_tokens(true);

                Ok(quote! {
                    #dir
                    #[asn(#ty_attrs)]
                    #vis struct #seq_of_type_ident(#vis Vec<#seq_of_type>);
                })
            }
        } else {
            Ok(TokenStream::new())
        }
    }
}
