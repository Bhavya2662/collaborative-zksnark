use derivative::Derivative;
use log::debug;
use rand::Rng;
use zeroize::Zeroize;

use ark_ff::bytes::{FromBytes, ToBytes};
use ark_ff::prelude::*;
use ark_ff::FftField;
use ark_serialize::{
    CanonicalDeserialize, CanonicalDeserializeWithFlags, CanonicalSerialize,
    CanonicalSerializeWithFlags, Flags, SerializationError,
};
use mpc_trait::MpcWire;

use std::fmt::{self, Debug, Display, Formatter};
use std::io::{self, Read, Write};
use std::iter::{Product, Sum};
use std::marker::PhantomData;
use std::ops::*;

use super::super::share::field::ScalarShare;
use super::super::share::BeaverSource;
use mpc_trait::Reveal;
use mpc_net;

#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MpcField<F: Field, S: ScalarShare<F>> {
    Public(F),
    Shared(S),
}

impl_basics_2!(ScalarShare, Field, MpcField);

#[derive(Derivative)]
#[derivative(Default(bound = ""), Clone(bound = ""), Copy(bound = ""))]
pub struct DummyScalarTripleSource<T, S> {
    _scalar: PhantomData<T>,
    _share: PhantomData<S>,
}

impl<T: Field, S: ScalarShare<T>> BeaverSource<S, S, S> for DummyScalarTripleSource<T, S> {
    #[inline]
    fn triple(&mut self) -> (S, S, S) {
        (
            S::from_add_shared(if mpc_net::am_first() {T::one()} else {T::zero()}),
            S::from_add_shared(if mpc_net::am_first() {T::one()} else {T::zero()}),
            S::from_add_shared(if mpc_net::am_first() {T::one()} else {T::zero()}),
        )
    }
    #[inline]
    fn inv_pair(&mut self) -> (S, S) {
        (
            S::from_add_shared(if mpc_net::am_first() {T::one()} else {T::zero()}),
            S::from_add_shared(if mpc_net::am_first() {T::one()} else {T::zero()}),
        )
    }
}

impl<T: Field, S: ScalarShare<T>> MpcField<T, S> {
    #[inline]
    pub fn inv(self) -> Option<Self> {
        match self {
            Self::Public(x) => x.inverse().map(MpcField::Public),
            Self::Shared(x) => Some(MpcField::Shared(
                x.inv(&mut DummyScalarTripleSource::default()),
            )),
        }
    }
}
impl<T: Field, S: ScalarShare<T>> Mul for MpcField<T, S> {
    type Output = Self;
    #[inline]
    fn mul(self, other: Self) -> Self::Output {
        match (self, other) {
            (MpcField::Public(x), MpcField::Public(y)) => MpcField::Public(x.mul(y)),
            (MpcField::Public(x), MpcField::Shared(y)) => MpcField::Shared(y.scale(&x)),
            (MpcField::Shared(x), MpcField::Public(y)) => MpcField::Shared(x.scale(&y)),
            (MpcField::Shared(x), MpcField::Shared(y)) => {
                MpcField::Shared(x.mul(y, &mut DummyScalarTripleSource::default()))
            }
        }
    }
}
impl<T: Field, S: ScalarShare<T>> One for MpcField<T, S> {
    #[inline]
    fn one() -> Self {
        MpcField::Public(T::one())
    }
}
impl<T: Field, S: ScalarShare<T>> Product for MpcField<T, S> {
    #[inline]
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::one(), Mul::mul)
    }
}
impl<'a, T: Field, S: ScalarShare<T> + 'a> Product<&'a MpcField<T, S>> for MpcField<T, S> {
    #[inline]
    fn product<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::one(), |x, y| x.mul(y.clone()))
    }
}

impl<T: Field, S: ScalarShare<T>> Div for MpcField<T, S> {
    type Output = Self;
    #[inline]
    fn div(self, other: Self) -> Self::Output {
        let src = &mut DummyScalarTripleSource::default();
        match (self, other) {
            (MpcField::Public(x), MpcField::Public(y)) => MpcField::Public(x.div(y)),
            (MpcField::Public(x), MpcField::Shared(y)) => MpcField::Shared(y.inv(src).scale(&x)),
            (MpcField::Shared(x), MpcField::Public(y)) => {
                MpcField::Shared(x.scale(&y.inverse().unwrap()))
            }
            (MpcField::Shared(x), MpcField::Shared(y)) => MpcField::Shared(x.div(y, src)),
        }
    }
}

impl_ref_ops!(
    Mul,
    MulAssign,
    mul,
    mul_assign,
    Field,
    ScalarShare,
    MpcField
);
impl_ref_ops!(
    Add,
    AddAssign,
    add,
    add_assign,
    Field,
    ScalarShare,
    MpcField
);
impl_ref_ops!(
    Div,
    DivAssign,
    div,
    div_assign,
    Field,
    ScalarShare,
    MpcField
);
impl_ref_ops!(
    Sub,
    SubAssign,
    sub,
    sub_assign,
    Field,
    ScalarShare,
    MpcField
);

impl<T: Field, S: ScalarShare<T>> MpcWire for MpcField<T, S> {
    #[inline]
    fn publicize(&mut self) {
        match self {
            MpcField::Shared(s) => {
                *self = MpcField::Public(s.open());
            }
            _ => {}
        }
        debug_assert!({
            let self_val = if let MpcField::Public(s) = self {
                s.clone()
            } else {
                unreachable!()
            };
            super::macros::check_eq(self_val.clone());
            true
        })
    }
    #[inline]
    fn set_shared(&mut self, shared: bool) {
        if shared != self.is_shared() {
            match self {
                Self::Shared(s) => {
                    let p = s.unwrap_as_public();
                    *self = Self::Public(p);
                }
                Self::Public(s) => {
                    *self = Self::Shared(S::wrap_as_shared(*s));
                }
            }
        }
    }
    #[inline]
    fn is_shared(&self) -> bool {
        match self {
            MpcField::Shared(_) => true,
            MpcField::Public(_) => false,
        }
    }
}

impl<T: Field, S: ScalarShare<T>> Reveal for MpcField<T, S> {
    type Base = T;
    #[inline]
    fn reveal(self) -> Self::Base {
        let result = match self {
            Self::Shared(s) => s.reveal(),
            Self::Public(s) => s,
        };
        super::macros::check_eq(result.clone());
        result
    }
    #[inline]
    fn from_public(b: Self::Base) -> Self {
        MpcField::Public(b)
    }
    #[inline]
    fn from_add_shared(b: Self::Base) -> Self {
        MpcField::Shared(S::from_add_shared(b))
    }
}

from_prim!(bool, Field, ScalarShare, MpcField);
from_prim!(u8, Field, ScalarShare, MpcField);
from_prim!(u16, Field, ScalarShare, MpcField);
from_prim!(u32, Field, ScalarShare, MpcField);
from_prim!(u64, Field, ScalarShare, MpcField);
from_prim!(u128, Field, ScalarShare, MpcField);

impl<T: PrimeField, S: ScalarShare<T>> std::str::FromStr for MpcField<T, S> {
    type Err = T::Err;
    #[inline]
    fn from_str(s: &str) -> Result<Self, T::Err> {
        T::from_str(s).map(Self::Public)
    }
}

impl<F: PrimeField, S: ScalarShare<F>> Field for MpcField<F, S> {
    type BasePrimeField = Self;
    #[inline]
    fn extension_degree() -> u64 {
        unimplemented!("extension_degree")
    }
    #[inline]
    fn from_base_prime_field_elems(b: &[<Self as ark_ff::Field>::BasePrimeField]) -> Option<Self> {
        assert!(b.len() > 0);
        let shared = b[0].is_shared();
        assert!(b.iter().all(|e| e.is_shared() == shared));
        let base_values = b.iter().map(|e| e.unwrap_as_public()).collect::<Vec<_>>();
        F::from_base_prime_field_elems(&base_values).map(|val| Self::new(val, shared))
    }
    #[inline]
    fn double(&self) -> Self {
        Self::Public(F::from(2u8)) * self
    }
    #[inline]
    fn double_in_place(&mut self) -> &mut Self {
        *self *= Self::Public(F::from(2u8));
        self
    }
    #[inline]
    fn from_random_bytes_with_flags<Fl: Flags>(b: &[u8]) -> Option<(Self, Fl)> {
        F::from_random_bytes_with_flags(b).map(|(val, f)| (Self::Shared(S::from_public(val)), f))
    }
    #[inline]
    fn square(&self) -> Self {
        self.clone() * self
    }
    #[inline]
    fn square_in_place(&mut self) -> &mut Self {
        *self *= self.clone();
        self
    }
    #[inline]
    fn inverse(&self) -> Option<Self> {
        self.inv()
    }
    #[inline]
    fn inverse_in_place(&mut self) -> Option<&mut Self> {
        self.inv().map(|i| {
            *self = i;
            self
        })
    }
    #[inline]
    fn frobenius_map(&mut self, _: usize) {
        unimplemented!("frobenius_map")
    }
}

impl<F: PrimeField, S: ScalarShare<F>> FftField for MpcField<F, S> {
    type FftParams = F::FftParams;
    #[inline]
    fn two_adic_root_of_unity() -> Self {
        Self::from_public(F::two_adic_root_of_unity())
    }
    #[inline]
    fn large_subgroup_root_of_unity() -> Option<Self> {
        F::large_subgroup_root_of_unity().map(Self::from_public)
    }
    #[inline]
    fn multiplicative_generator() -> Self {
        Self::from_public(F::multiplicative_generator())
    }
}

impl<F: PrimeField, S: ScalarShare<F>> PrimeField for MpcField<F, S> {
    type Params = F::Params;
    type BigInt = F::BigInt;
    #[inline]
    fn from_repr(r: <Self as PrimeField>::BigInt) -> Option<Self> {
        // F::from_repr(r.val).map(|v| MpcVal::new(v, r.shared))
        F::from_repr(r).map(|v| Self::from_public(v))
    }
    // We're assuming that into_repr is linear
    #[inline]
    fn into_repr(&self) -> <Self as PrimeField>::BigInt {
        // MpcVal::new(self.val.into_repr(), self.shared)
        self.unwrap_as_public().into_repr()
    }
}

impl<F: PrimeField, S: ScalarShare<F>> SquareRootField for MpcField<F, S> {
    #[inline]
    fn legendre(&self) -> ark_ff::LegendreSymbol {
        todo!()
    }
    #[inline]
    fn sqrt(&self) -> Option<Self> {
        todo!()
    }
    #[inline]
    fn sqrt_in_place(&mut self) -> Option<&mut Self> {
        todo!()
    }
}