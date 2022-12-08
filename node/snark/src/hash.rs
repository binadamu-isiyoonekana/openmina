#![allow(unused)]

use ark_ff::{BigInteger, BigInteger256, Field, FromBytes, PrimeField};
use mina_curves::pasta::Fq;
use mina_hasher::Fp;
use mina_p2p_messages::v1::StateHashStable;
use mina_p2p_messages::v2::{
    MinaBlockBlockStableV2, MinaBlockHeaderStableV2, MinaStateProtocolStateValueStableV2,
};
use oracle::{
    constants::PlonkSpongeConstantsKimchi,
    pasta::fp_kimchi::static_params,
    poseidon::{ArithmeticSponge, Sponge},
};

use crate::utils::{FpExt, SpongeParamsForField};

enum Item {
    Bool(bool),
    U2(u8),
    U8(u8),
    U32(u32),
    U48([u8; 6]),
    U64(u64),
}

impl std::fmt::Debug for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(arg0) => f.write_fmt(format_args!("{}_bool", i32::from(*arg0))),
            Self::U2(arg0) => f.write_fmt(format_args!("{}_u2", arg0)),
            Self::U8(arg0) => f.write_fmt(format_args!("{}_u8", arg0)),
            Self::U32(arg0) => f.write_fmt(format_args!("{}_u32", arg0)),
            Self::U48(arg0) => f.write_fmt(format_args!("{:?}_u48", arg0)),
            Self::U64(arg0) => f.write_fmt(format_args!("{}_u64", arg0)),
        }
    }
}

impl Item {
    fn nbits(&self) -> u32 {
        match self {
            Item::Bool(_) => 1,
            Item::U2(_) => 2,
            Item::U8(_) => 8,
            Item::U32(_) => 32,
            Item::U48(_) => 48,
            Item::U64(_) => 64,
        }
    }

    fn as_bigint(&self) -> BigInteger256 {
        match self {
            Item::Bool(v) => {
                if *v {
                    1.into()
                } else {
                    0.into()
                }
            }
            Item::U2(v) => (*v as u64).into(),
            Item::U8(v) => (*v as u64).into(),
            Item::U32(v) => (*v as u64).into(),
            Item::U48(v) => {
                let mut bytes = <[u8; 32]>::default();
                bytes[..6].copy_from_slice(&v[..]);
                BigInteger256::read(&bytes[..]).unwrap()
            }
            Item::U64(v) => (*v).into(),
        }
    }
}

pub struct Inputs {
    fields: Vec<Fp>,
    packeds: Vec<Item>,
}

impl Default for Inputs {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Inputs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Inputs")
            .field(
                &format!("fields[{:?}]", self.fields.len()),
                &self
                    .fields
                    .iter()
                    .map(|f| f.to_decimal())
                    .collect::<Vec<_>>(),
            )
            .field(&format!("packeds[{:?}]", self.packeds.len()), &self.packeds)
            .finish()
    }
}

impl Inputs {
    pub fn new() -> Self {
        Self {
            fields: Vec::with_capacity(256),
            packeds: Vec::with_capacity(256),
        }
    }

    pub fn append_bool(&mut self, value: bool) {
        self.packeds.push(Item::Bool(value));
    }

    pub fn append_u2(&mut self, value: u8) {
        self.packeds.push(Item::U2(value));
    }

    pub fn append_u8(&mut self, value: u8) {
        self.packeds.push(Item::U8(value));
    }

    pub fn append_u32(&mut self, value: u32) {
        self.packeds.push(Item::U32(value));
    }

    pub fn append_u64(&mut self, value: u64) {
        self.packeds.push(Item::U64(value));
    }

    pub fn append_u48(&mut self, value: [u8; 6]) {
        self.packeds.push(Item::U48(value));
    }

    pub fn append_field(&mut self, value: Fp) {
        self.fields.push(value);
    }

    pub fn append_bytes(&mut self, value: &[u8]) {
        const BITS: [u8; 8] = [1, 2, 4, 8, 16, 32, 64, 128];

        self.packeds.reserve(value.len() * 8);

        for byte in value {
            for bit in BITS {
                self.append_bool(byte & bit != 0);
            }
        }
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_fields(mut self) -> Vec<Fp> {
        let mut nbits = 0;
        let mut current: BigInteger256 = 0.into();

        for (item, item_nbits) in self.packeds.iter().map(|i| (i.as_bigint(), i.nbits())) {
            nbits += item_nbits;

            if nbits < 255 {
                current.muln(item_nbits);

                // Addition, but we use 'bitwise or' because we know bits of
                // `current` are zero (we just shift-left them)
                current = BigInteger256([
                    current.0[0] | item.0[0],
                    current.0[1] | item.0[1],
                    current.0[2] | item.0[2],
                    current.0[3] | item.0[3],
                ]);
            } else {
                self.fields.push(current.into());
                current = item;
                nbits = item_nbits;
            }
        }

        if nbits > 0 {
            self.fields.push(current.into());
        }

        self.fields
    }
}

fn param_to_field_impl(param: &str, default: &[u8; 32]) -> Fp {
    let param_bytes = param.as_bytes();
    let len = param_bytes.len();

    let mut fp = *default;
    fp[..len].copy_from_slice(param_bytes);

    Fp::read(&fp[..]).expect("fp read failed")
}

fn param_to_field(param: &str) -> Fp {
    const DEFAULT: [u8; 32] = [
        b'*', b'*', b'*', b'*', b'*', b'*', b'*', b'*', b'*', b'*', b'*', b'*', b'*', b'*', b'*',
        b'*', b'*', b'*', b'*', b'*', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    if param.len() > 20 {
        panic!("must be 20 byte maximum");
    }

    param_to_field_impl(param, &DEFAULT)
}

fn param_to_field_noinputs(param: &str) -> Fp {
    const DEFAULT: [u8; 32] = [0; 32];

    if param.len() > 32 {
        panic!("must be 32 byte maximum");
    }

    param_to_field_impl(param, &DEFAULT)
}

pub fn hash_with_kimchi(param: &str, fields: &[Fp]) -> Fp {
    let mut sponge = ArithmeticSponge::<Fp, PlonkSpongeConstantsKimchi>::new(static_params());

    sponge.absorb(&[param_to_field(param)]);
    sponge.squeeze();

    sponge.absorb(fields);
    sponge.squeeze()
}

pub fn hash_fields<F: Field + SpongeParamsForField<F>>(fields: &[F]) -> F {
    let mut sponge = ArithmeticSponge::<F, PlonkSpongeConstantsKimchi>::new(F::get_params());

    sponge.absorb(fields);
    sponge.squeeze()
}

pub fn hash_noinputs(param: &str) -> Fp {
    let mut sponge = ArithmeticSponge::<Fp, PlonkSpongeConstantsKimchi>::new(static_params());
    // ArithmeticSponge::<Fp, PlonkSpongeConstantsKimchi>::new(pasta::fp_kimchi::static_params());

    sponge.absorb(&[param_to_field_noinputs(param)]);
    sponge.squeeze()
}

pub trait StateHashable {
    fn as_hashable(&self) -> &MinaStateProtocolStateValueStableV2;
}

impl StateHashable for MinaBlockBlockStableV2 {
    fn as_hashable(&self) -> &MinaStateProtocolStateValueStableV2 {
        &self.header.as_hashable()
    }
}

impl StateHashable for MinaBlockHeaderStableV2 {
    fn as_hashable(&self) -> &MinaStateProtocolStateValueStableV2 {
        &self.protocol_state
    }
}

impl StateHashable for MinaStateProtocolStateValueStableV2 {
    fn as_hashable(&self) -> &MinaStateProtocolStateValueStableV2 {
        self
    }
}

pub fn state_hash<T: StateHashable>(t: &T) -> StateHashStable {
    use crate::public_input::protocol_state::MinaHash;
    use mina_p2p_messages::v1::StateHashStableV1;

    let field = t.as_hashable().hash();
    StateHashStableV1::from_bigint(field.into()).into()
}

#[cfg(test)]
mod tests {
    use o1_utils::FieldHelpers;

    #[cfg(target_family = "wasm")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    use super::*;

    #[test]
    fn test_param() {
        for (s, hex) in [
            (
                "",
                "2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a000000000000000000000000",
            ),
            (
                "hello",
                "68656c6c6f2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a000000000000000000000000",
            ),
            (
                "aaaaaaaaaaaaaaaaaaaa",
                "6161616161616161616161616161616161616161000000000000000000000000",
            ),
        ] {
            let field = param_to_field(s);
            assert_eq!(field.to_hex(), hex);
        }
    }

    #[test]
    fn test_inputs() {
        let mut inputs = Inputs::new();

        inputs.append_bool(true);
        inputs.append_u64(0); // initial_minimum_balance
        inputs.append_u32(0); // cliff_time
        inputs.append_u64(0); // cliff_amount
        inputs.append_u32(1); // vesting_period
        inputs.append_u64(0); // vesting_increment

        println!("INPUTS={:?}", inputs);
        println!("FIELDS={:?}", inputs.to_fields());

        // // Self::timing
        // match self.timing {
        //     Timing::Untimed => {
        //         roi.append_bool(false);
        //         roi.append_u64(0); // initial_minimum_balance
        //         roi.append_u32(0); // cliff_time
        //         roi.append_u64(0); // cliff_amount
        //         roi.append_u32(1); // vesting_period
        //         roi.append_u64(0); // vesting_increment
        //     }
        //     Timing::Timed {
        //         initial_minimum_balance,
        //         cliff_time,
        //         cliff_amount,
        //         vesting_period,
        //         vesting_increment,
        //     } => {
        //         roi.append_bool(true);
        //         roi.append_u64(initial_minimum_balance);
        //         roi.append_u32(cliff_time);
        //         roi.append_u64(cliff_amount);
        //         roi.append_u32(vesting_period);
        //         roi.append_u64(vesting_increment);
        //     }
        // }
    }
}
