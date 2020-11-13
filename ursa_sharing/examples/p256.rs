// Copyright 2020 Hyperledger Ursa Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use generic_array::{typenum::U32, GenericArray};
use rand::{CryptoRng, RngCore};
use ursa_sharing::{error::*, tests::*, Field};

use ff::Field as FFField;
use p256::elliptic_curve::ops::Neg;
use p256::{
    elliptic_curve::{
        sec1::{FromEncodedPoint, ToEncodedPoint},
        Group,
    },
    AffinePoint, EncodedPoint, FieldBytes, ProjectivePoint, Scalar,
};

struct P256Scalar(Scalar);

impl Clone for P256Scalar {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Field<P256Scalar> for P256Scalar {
    type FieldSize = U32;

    fn zero() -> Self {
        Self(Scalar::zero())
    }

    fn one() -> Self {
        Self(Scalar::one())
    }

    fn from_usize(value: usize) -> Self {
        Self(Scalar::from(value as u64))
    }

    fn from_bytes<B: AsRef<[u8]>>(value: B) -> SharingResult<Self> {
        let value = value.as_ref();
        if value.len() <= 32 {
            let mut s = [0u8; 32];
            s[..value.len()].copy_from_slice(value);
            Ok(Self(Scalar::from_bytes_reduced(FieldBytes::from_slice(&s))))
        } else {
            Err(SharingError::ShareInvalidSecret)
        }
    }

    fn random(rng: &mut (impl RngCore + CryptoRng)) -> Self {
        Self(Scalar::random(rng))
    }

    fn is_zero(&self) -> bool {
        self.0.is_zero().unwrap_u8() == 1
    }

    fn is_valid(&self) -> bool {
        self.0.is_zero().unwrap_u8() == 0
    }

    fn negate(&mut self) {
        self.0 = self.0.neg()
    }

    fn add_assign(&mut self, rhs: &Self) {
        self.0 += rhs.0
    }

    fn sub_assign(&mut self, rhs: &Self) {
        self.0 -= rhs.0
    }

    fn mul_assign(&mut self, rhs: &P256Scalar) {
        self.0 *= rhs.0
    }

    fn div_assign(&mut self, rhs: &P256Scalar) {
        self.0 *= rhs.0.invert().unwrap()
    }

    fn to_bytes(&self) -> GenericArray<u8, Self::FieldSize> {
        let mut c = [0u8; 32];
        c.copy_from_slice(self.0.to_bytes().as_slice());
        c.into()
    }
}

struct P256Point(ProjectivePoint);

impl Clone for P256Point {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Field<P256Scalar, P256Point> for P256Point {
    type FieldSize = U32;

    fn zero() -> Self {
        Self(ProjectivePoint::identity())
    }

    fn one() -> Self {
        Self(ProjectivePoint::generator())
    }

    fn from_usize(_: usize) -> Self {
        unimplemented!()
    }

    fn from_bytes<B: AsRef<[u8]>>(value: B) -> SharingResult<Self> {
        match EncodedPoint::from_bytes(value.as_ref()) {
            Ok(ept) => {
                let apt = AffinePoint::from_encoded_point(&ept);
                if apt.is_some().unwrap_u8() == 1 {
                    let ppt = ProjectivePoint::from(apt.unwrap());
                    Ok(Self(ppt))
                } else {
                    Err(SharingError::InvalidPoint)
                }
            }
            Err(_) => Err(SharingError::InvalidPoint),
        }
    }

    fn random(rng: &mut (impl RngCore + CryptoRng)) -> Self {
        Self(ProjectivePoint::random(rng))
    }

    fn is_zero(&self) -> bool {
        self.0.is_identity().unwrap_u8() == 1
    }

    fn is_valid(&self) -> bool {
        self.0.is_identity().unwrap_u8() == 0
    }

    fn negate(&mut self) {
        self.0 = -self.0
    }

    fn add_assign(&mut self, rhs: &P256Point) {
        self.0 += rhs.0;
    }

    fn sub_assign(&mut self, rhs: &P256Point) {
        self.0 -= rhs.0;
    }

    fn mul_assign(&mut self, rhs: &P256Scalar) {
        self.0 *= rhs.0;
    }

    fn div_assign(&mut self, rhs: &P256Scalar) {
        self.0 *= rhs.0.invert().unwrap()
    }

    fn to_bytes(&self) -> GenericArray<u8, U32> {
        let mut c = [0u8; 32];
        c.copy_from_slice(
            self.0
                .to_affine()
                .to_encoded_point(true)
                .to_bytes()
                .as_ref(),
        );
        c.into()
    }
}

fn main() {
    println!("Splitting");
    split_invalid_args::<P256Scalar>();
    println!("Combine invalid fail");
    combine_invalid::<P256Scalar>();
    println!("Combine single success");
    combine_single::<P256Scalar, P256Point>();
    println!("Combine combinations success");
    combine_all_combinations::<P256Scalar, P256Point>();
}
