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

                Ok(quote! {
                    #dir
                    #[asn(#ty_attrs)]
                    #vis struct #seq_of_type_ident(#vis Vec<#seq_of_type>);

                    impl<'a> arbitrary::Arbitrary<'a> for #seq_of_type_ident {
                        fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {

                            let vec_length = std::cmp::max(#min, std::cmp::min(4, u.int_in_range(#min..=#max)?));
                            let mut v = Vec::new();

                            for _ in 0..vec_length {
                                v.push(u.arbitrary()?);
                            }

                            Ok(#seq_of_type_ident(v))
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
