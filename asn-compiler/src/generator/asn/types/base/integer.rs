//! Mainly 'generator' code for `Asn1ResolvedInteger`

use proc_macro2::{Ident, TokenStream};
use quote::quote;

use crate::error::Error;

use crate::generator::Generator;
use crate::resolver::asn::structs::types::base::Asn1ResolvedInteger;

impl Asn1ResolvedInteger {
    pub(crate) fn generate(
        &self,
        name: &str,
        generator: &mut Generator,
    ) -> Result<TokenStream, Error> {
        let struct_name = generator.to_type_ident(name);
        let inner_type = generator.to_inner_type(self.bits, self.signed);
        let (min, max) = self.get_min_max_constraints();
        let extensible = self.resolved_constraints.is_some()
            && self.resolved_constraints.as_ref().unwrap().has_extension();

        let lb = if min.is_some() {
            let min = format!("{}", min.unwrap());
            Some(quote! { #min })
        } else {
            None
        };

        let ub = if max.is_some() {
            let max = format!("{}", max.unwrap());
            Some(quote! { #max })
        } else {
            None
        };

        let mut ty_tokens = quote! { type = "INTEGER" };
        if lb.is_some() {
            ty_tokens.extend(quote! {
                , lb = #lb
            });
        }
        if ub.is_some() {
            ty_tokens.extend(quote! {
                , ub = #ub
            });
        }

        if extensible {
            ty_tokens.extend(quote! {
                , extensible = true
            });
        }

        let vis = generator.get_visibility_tokens();
        let dir = generator.generate_derive_tokens(false);

        let lb_int = proc_macro2::Literal::i128_unsuffixed(min.unwrap_or(0));
        let ub_int = proc_macro2::Literal::i128_unsuffixed(max.unwrap_or(127));

        let struct_tokens = quote! {
            #dir
            #[asn(#ty_tokens)]
            #vis struct #struct_name(#vis #inner_type);

            impl entropic::Entropic for #struct_name {
                fn from_finite_entropy<'a, S: EntropyScheme, I: Iterator<Item = &'a u8>>(
                    source: &mut entropic::FiniteEntropySource<'a, S, I>,
                ) -> Result<Self, entropic::Error> {
                    Ok(#struct_name(source.get_uniform_range(#lb_int..=#ub_int)?))
                }

                fn to_finite_entropy<'a, S: EntropyScheme, I: Iterator<Item = &'a mut u8>>(
                    &self,
                    sink: &mut FiniteEntropySink<'a, S, I>,
                ) -> Result<usize, Error> {
                    Ok(sink.put_uniform_range(#lb_int..=#ub_int as #inner_type, self.0)?)
                }
            }
        };

        Ok(struct_tokens)
    }

    pub(crate) fn generate_ident_and_aux_type(
        &self,
        generator: &mut Generator,
        input: Option<&String>,
    ) -> Result<Ident, Error> {
        let unique_name = if input.is_none() {
            generator.get_unique_name("INTEGER")
        } else {
            input.unwrap().to_string()
        };

        let item = self.generate(&unique_name, generator)?;
        generator.aux_items.push(item);

        Ok(generator.to_type_ident(&unique_name))
    }

    fn get_min_max_constraints(&self) -> (Option<i128>, Option<i128>) {
        if self.resolved_constraints.is_none() {
            (None, None)
        } else {
            let constraints = self.resolved_constraints.as_ref().unwrap();

            (constraints.root_values.min(), constraints.root_values.max())
        }
    }
}
