//! Implementation of Code Generation for ASN.1 `SEQUENCE` Type.

use proc_macro2::TokenStream;
use quote::quote;

use crate::error::Error;
use crate::generator::Generator;
use crate::resolver::asn::structs::types::{
    constructed::ResolvedConstructedType, Asn1ResolvedType,
};

impl ResolvedConstructedType {
    pub(crate) fn generate_sequence(
        &self,
        name: &str,
        generator: &mut Generator,
    ) -> Result<TokenStream, Error> {
        if let ResolvedConstructedType::Sequence {
            ref components,
            ref extensible,
            ..
        } = self
        {
            let type_name = generator.to_type_ident(name);

            let extensible = if *extensible {
                quote! { true }
            } else {
                quote! { false }
            };

            let vis = generator.get_visibility_tokens();

            let mut from_entropy_defs = TokenStream::new();
            let mut from_entropy_fields = TokenStream::new();
            let mut to_entropy_defs = TokenStream::new();

            let mut key_field = None;
            let mut key_value_name = None;
            let mut comp_tokens = TokenStream::new();
            let mut optional_fields = 0;
            for (idx, c) in components.iter().enumerate() {
                let comp_field_ident = generator.to_value_ident(&c.component.id);
                let comp_ty_suffix = generator.to_type_ident(&c.component.id);
                let input_comp_ty_ident = format!("{}{}", name, comp_ty_suffix);
                let comp_ty_ident = Asn1ResolvedType::generate_name_maybe_aux_type(
                    &c.component.ty,
                    generator,
                    Some(&input_comp_ty_ident),
                )?;
                let mut fld_attrs = vec![];

                let (ty_ident, fld_tokens) = if c.optional {
                    let idx: proc_macro2::TokenStream =
                        format!("{}", optional_fields).parse().unwrap();
                    fld_attrs.push(quote! { optional_idx = #idx,  });

                    optional_fields += 1;

                    (quote! { Option<#comp_ty_ident> }, quote! { #vis #comp_field_ident: Option<#comp_ty_ident>, })
                } else {
                    (quote! { #comp_ty_ident }, quote! { #vis #comp_field_ident: #comp_ty_ident, })
                };

                if !c.key_field && (idx + 1 == components.len()) {
                    key_value_name = Some(comp_field_ident.clone());
                }

                from_entropy_fields.extend(quote! { #comp_field_ident, });

                // TODO: temporary hack
                if c.key_field {
                    fld_attrs.push(quote! { key_field = true });
                    key_field = Some((comp_field_ident, ty_ident));
                } else if comp_field_ident.to_string().as_str() == "ie_extensions" { 
                    from_entropy_defs.extend(quote!{ let #comp_field_ident: #ty_ident = None; });
                } else {
                    from_entropy_defs.extend(quote!{ let #comp_field_ident: #ty_ident = source.get_entropic()?; });
                    to_entropy_defs.extend(quote!{ __entropic_internal_length += self.#comp_field_ident.to_entropy_sink(sink)?; });
                }

                let fld_attr_tokens = if !fld_attrs.is_empty() {
                    quote! { #[asn(#(#fld_attrs),*)] }
                } else {
                    quote! {}
                };

                comp_tokens.extend(quote! {
                    #fld_attr_tokens #fld_tokens
                });
            }

            // Should only happen if c.key_field == true
            if let (Some((comp_field_ident, ty_ident)), Some(key_value_field_ident)) = (key_field, key_value_name) {
                from_entropy_defs.extend(quote! { let #comp_field_ident: #ty_ident = #ty_ident(#key_value_field_ident.choice_key()); });
                // to_entropy_defs doesn't need anything here as no entropy was expended
            }

            let mut ty_tokens = quote! { type = "SEQUENCE", extensible = #extensible };

            if optional_fields > 0 {
                let optflds: proc_macro2::TokenStream =
                    format!("{}", optional_fields).parse().unwrap();
                ty_tokens.extend(quote! { , optional_fields = #optflds });
            }

            let dir = generator.generate_derive_tokens(false); // TODO: eventually fix this so that fields are sequentially optional
            Ok(quote! {
                #dir
                #[asn(#ty_tokens)]
                #vis struct #type_name {
                    #comp_tokens
                }

                impl entropic::Entropic for #type_name {
                    #[inline]
                    fn from_entropy_source<'a, I: Iterator<Item = &'a u8>, E: EntropyScheme>(
                        source: &mut Source<'a, I, E>,
                    ) -> Result<Self, Error> {
                        #from_entropy_defs
                        Ok(Self {
                            #from_entropy_fields
                        })
                    }
                
                    #[inline]
                    fn to_entropy_sink<'a, I: Iterator<Item = &'a mut u8>, E: EntropyScheme>(
                        &self,
                        sink: &mut Sink<'a, I, E>,
                    ) -> Result<usize, Error> {
                        let mut __entropic_internal_length = 0;
                        #to_entropy_defs
                        Ok(__entropic_internal_length)
                    }
                }
            })
        } else {
            Ok(TokenStream::new())
        }
    }
}
