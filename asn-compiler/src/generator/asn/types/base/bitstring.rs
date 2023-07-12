//! Generator code for Base Type Asn1ResolvedBitString

use proc_macro2::{Ident, TokenStream};
use quote::quote;

use crate::error::Error;

use crate::generator::Generator;
use crate::resolver::asn::structs::types::base::Asn1ResolvedBitString;

impl Asn1ResolvedBitString {
    pub(crate) fn generate(
        &self,
        name: &str,
        generator: &mut Generator,
    ) -> Result<TokenStream, Error> {
        let struct_name = generator.to_type_ident(name);

        let mut ty_attributes = quote! { type = "BITSTRING" };

        let vis = generator.get_visibility_tokens();


        if let Some(s) = self.size.as_ref() {
            let sz_attributes = s.get_ty_size_constraints_attrs();
            ty_attributes.extend(sz_attributes);

            let min = proc_macro2::Literal::i128_unsuffixed(s.root_values.min().unwrap());
            let max = proc_macro2::Literal::i128_unsuffixed(s.root_values.max().unwrap());
            

            let dir = generator.generate_derive_tokens(false);

            let struct_tokens = quote! {
                #dir
                #[asn(#ty_attributes)]
                #vis struct #struct_name(#vis bitvec::vec::BitVec<u8, bitvec::order::Msb0>);

                impl<'a> arbitrary::Arbitrary<'a> for #struct_name {
                    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
                        let mut bv = bitvec::vec::BitVec::EMPTY;
                        for _ in 0..#min {
                            bv.push(u.arbitrary()?);
                        }
                        
                        if #max > #min {
                            for _ in 0..u.int_in_range(0..=#max - #min - 1)? {
                                bv.push(u.arbitrary()?);
                            }
                        }
                
                        Ok(#struct_name(bv)) 
                    }
                }
            };

            Ok(struct_tokens)
        } else {
            let dir = generator.generate_derive_tokens(true);

            let struct_tokens = quote! {
                #dir
                #[asn(#ty_attributes)]
                #vis struct #struct_name(#vis bitvec::vec::BitVec<u8, bitvec::order::Msb0>);
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
            generator.get_unique_name("BIT STRING")
        } else {
            input.unwrap().to_string()
        };

        let item = self.generate(&unique_name, generator)?;
        generator.aux_items.push(item);

        Ok(generator.to_type_ident(&unique_name))
    }
}
