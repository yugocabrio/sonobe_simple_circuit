#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(clippy::upper_case_acronyms)]
use ark_bn254::{constraints::GVar, Bn254, Fr, G1Projective as G1};
use ark_ff::PrimeField;
use ark_groth16::Groth16;
use ark_grumpkin::{constraints::GVar as GVar2, Projective as G2};
use ark_r1cs_std::{alloc::AllocVar, fields::FieldVar};
use ark_r1cs_std::fields::fp::FpVar;
use ark_relations::r1cs::{ConstraintSystemRef, SynthesisError};
use rand::distributions::weighted::{self, alias_method::Weight};
use std::marker::PhantomData;
use std::time::Instant;

mod utils;
use utils::init_ivc_and_decider_params;

use folding_schemes::{
    commitment::{kzg::KZG, pedersen::Pedersen},
    folding::nova::{
        decider_eth::{prepare_calldata, Decider as DeciderEth},
        Nova,
    },
    frontend::FCircuit,
    Decider, Error, FoldingScheme,
};

#[derive(Clone, Copy, Debug)]
pub struct FibonacciCircuit<F: PrimeField> {
    _f: PhantomData<F>,
}

impl<F: PrimeField> FibonacciCircuit<F> {
    pub fn new() -> Self {
        Self { _f: PhantomData }
    }
}

impl<F: PrimeField> FCircuit<F> for FibonacciCircuit<F> {
    type Params = ();

    fn new(_params: Self::Params) -> Result<Self, Error> {
        Ok(Self { _f: PhantomData })
    }

    fn state_len(&self) -> usize {
        2
    }

    fn external_inputs_len(&self) -> usize {
        0
    }

    fn step_native(
        &self,
        _i: usize,
        z_i: Vec<F>,
        _external_inputs: Vec<F>,
    ) -> Result<Vec<F>, Error> {
        let new_fib = z_i[0] + z_i[1];
        println!("Current Fibonacci numbers: {}, {}", z_i[1], new_fib);
        Ok(vec![z_i[1], new_fib])
    }

    fn generate_step_constraints(
        &self,
        cs: ConstraintSystemRef<F>,
        _i: usize,
        z_i: Vec<FpVar<F>>,
        _external_inputs: Vec<FpVar<F>>,
    ) -> Result<Vec<FpVar<F>>, SynthesisError> {
        let new_fib = &z_i[0] + &z_i[1];
        Ok(vec![z_i[1].clone(), new_fib])
    }
}

fn main() {
    let n_steps = 10;

    let z_0 = vec![Fr::from(0_u32), Fr::from(1_u32)];

    let f_circuit = FibonacciCircuit::<Fr>::new();
    let (fs_prover_params, kzg_vk, g16_pk, g16_vk) =
        init_ivc_and_decider_params::<FibonacciCircuit<Fr>>(f_circuit);

    pub type NOVA = Nova<G1, GVar, G2, GVar2, FibonacciCircuit<Fr>, KZG<'static, Bn254>, Pedersen<G2>>;
    pub type DECIDERETH_FCircuit = DeciderEth<
        G1,
        GVar,
        G2,
        GVar2,
        FibonacciCircuit<Fr>,
        KZG<'static, Bn254>,
        Pedersen<G2>,
        Groth16<Bn254>,
        NOVA,
    >;

    // initialize the folding scheme engine, in our case we use Nova
    let mut nova = NOVA::init(&fs_prover_params, f_circuit, z_0).unwrap();
    // run n steps of the folding iteration
    for i in 0..n_steps {
        let start = Instant::now();
        nova.prove_step(vec![]).unwrap();
        println!("Nova::prove_step {}: {:?}", i, start.elapsed());
    }

    let rng = rand::rngs::OsRng;
    let start = Instant::now();
    let proof = DECIDERETH_FCircuit::prove(
        (g16_pk, fs_prover_params.cs_params.clone()),
        rng,
        nova.clone(),
    )
    .unwrap();
    println!("generated Decider proof: {:?}", start.elapsed());

    let verified = DECIDERETH_FCircuit::verify(
        (g16_vk.clone(), kzg_vk.clone()),
        nova.i,
        nova.z_0.clone(),
        nova.z_i.clone(),
        &nova.U_i,
        &nova.u_i,
        &proof,
    )
    .unwrap();
    assert!(verified);
    println!("Decider proof verification: {}", verified);
}
