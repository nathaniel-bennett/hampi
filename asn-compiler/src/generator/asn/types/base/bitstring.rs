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

                impl entropic::Entropic for #struct_name {
                    fn from_finite_entropy<'a, S: EntropyScheme, I: Iterator<Item = &'a u8>>(
                        source: &mut entropic::FiniteEntropySource<'a, S, I>,
                    ) -> Result<Self, entropic::Error> {
                        let mut bv = bitvec::vec::BitVec::EMPTY;

                        let capped_max = std::cmp::min(#max, 16383);

                        let total_bitlen = source.get_bounded_len(#min..=capped_max)?;
                        assert!(total_bitlen <= capped_max);
                        let bytes = total_bitlen / 8;
                        let bits = total_bitlen & 0b111; // Mod 8

                        for _ in 0..bytes {
                            bv.extend_from_raw_slice(source.entropic::<u8>()?.to_ne_bytes().as_slice())
                        }
                        
                        for _ in 0..bits {
                            bv.push(source.entropic()?);
                        }

                        Ok(#struct_name(bv))
                    }

                    fn to_finite_entropy<'a, S: EntropyScheme, I: Iterator<Item = &'a mut u8>>(
                        &self,
                        sink: &mut FiniteEntropySink<'a, S, I>,
                    ) -> Result<usize, Error> {
                        assert!(self.0.len() >= #min);

                        let capped_max = std::cmp::min(#max, 16383);

                        let mut length = 0;
                        length += sink.put_bounded_len(#min..=capped_max, self.0.len())?;
                        let bytes = self.0.len() / 8;
                        let bits = self.0.len() & 0b111;

                        for idx in 0..bytes {
                            length += sink.put_entropic(&self.0.as_raw_slice()[idx])?;
                        }

                        for idx in 0..bits {
                            length += sink.put_entropic(&self.0[(8 * bytes) + idx])?;
                        }

                        Ok(length)
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
