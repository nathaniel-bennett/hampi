//! Codec support for ASN.1 Types.

#![allow(dead_code)]
mod per;

use std::convert::TryFrom;
use std::fmt::Debug;

#[doc(inline)]
pub use per::PerCodecData;

#[doc(inline)]
pub use per::PerCodecError;

#[doc(inline)]
pub use per::aper;

#[doc(inline)]
pub use per::uper;

//pub trait ChoiceKey: TryFrom<u128> { }


pub trait Asn1Choice {
    fn choice_key<K: TryFrom<u128> + Debug>(&self) -> K
    where <K as TryFrom<u128>>::Error: std::fmt::Debug;
}
