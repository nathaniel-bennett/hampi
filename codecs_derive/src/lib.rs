use proc_macro::TokenStream;
use quote::quote;
use syn::Meta::List;
use syn::{parse_macro_input, DeriveInput};

mod attrs;

mod symbol;

#[proc_macro_derive(AperCodec, attributes(asn))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let name = &ast.ident;

    let codec_params = attrs::parse_variant_meta_as_codec_params(&ast.attrs);

    let codec_params = attrs::get_codec_params_from_meta_items(&ast.attrs).unwrap();
    let lb = codec_params.lb;
    let ub = codec_params.ub;

    let tokens = quote! {

        impl asn_codecs::aper::AperCodec for #name {
            type Output = Self;

            fn decode(data: &mut asn_codecs::aper::AperCodecData) -> Result<Self::Output, asn_codecs::aper::AperCodecError> {

                asn_codecs::aper::decode_choice_idx(#lb, #ub);

                Ok(Self{})
            }
        }
    };

    TokenStream::from(tokens)
}
