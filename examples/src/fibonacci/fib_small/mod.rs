// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::utils::compute_fib_term;
use crate::{Example, ExampleOptions, HashFunction};
use core::marker::PhantomData;
use log::debug;
use std::time::Instant;
use winterfell::{
    crypto::ElementHasher,
    math::{fields::f64::BaseElement, log2, FieldElement},
    ProofOptions, Prover, StarkProof, Trace, TraceTable, VerifierError,
};

mod air;
use air::FibSmall;

mod prover;
use prover::FibSmallProver;

#[cfg(test)]
mod tests;

// CONSTANTS AND TYPES
// ================================================================================================

const TRACE_WIDTH: usize = 2;

type Blake3_192 = winterfell::crypto::hashers::Blake3_192<BaseElement>;
type Blake3_256 = winterfell::crypto::hashers::Blake3_256<BaseElement>;
type Sha3_256 = winterfell::crypto::hashers::Sha3_256<BaseElement>;
type Rp64_256 = winterfell::crypto::hashers::Rp64_256;
type RpJive64_256 = winterfell::crypto::hashers::RpJive64_256;
type GriffinJive64_256 = winterfell::crypto::hashers::GriffinJive64_256;

// FIBONACCI EXAMPLE
// ================================================================================================

pub fn get_example(
    options: &ExampleOptions,
    sequence_length: usize,
) -> Result<Box<dyn Example>, String> {
    let (options, hash_fn) = options.to_proof_options(28, 8);

    match hash_fn {
        HashFunction::Blake3_192 => Ok(Box::new(FibExample::<Blake3_192>::new(
            sequence_length,
            options,
        ))),
        HashFunction::Blake3_256 => Ok(Box::new(FibExample::<Blake3_256>::new(
            sequence_length,
            options,
        ))),
        HashFunction::Sha3_256 => Ok(Box::new(FibExample::<Sha3_256>::new(
            sequence_length,
            options,
        ))),
        HashFunction::Rp64_256 => Ok(Box::new(FibExample::<Rp64_256>::new(
            sequence_length,
            options,
        ))),
        HashFunction::RpJive64_256 => Ok(Box::new(FibExample::<RpJive64_256>::new(
            sequence_length,
            options,
        ))),
        HashFunction::GriffinJive64_256 => Ok(Box::new(FibExample::<GriffinJive64_256>::new(
            sequence_length,
            options,
        ))),
    }
}

pub struct FibExample<H: ElementHasher> {
    options: ProofOptions,
    sequence_length: usize,
    result: BaseElement,
    _hasher: PhantomData<H>,
}

impl<H: ElementHasher> FibExample<H> {
    pub fn new(sequence_length: usize, options: ProofOptions) -> Self {
        assert!(
            sequence_length.is_power_of_two(),
            "sequence length must be a power of 2"
        );

        // compute Fibonacci sequence
        let now = Instant::now();
        let result = compute_fib_term::<BaseElement>(sequence_length);
        debug!(
            "Computed Fibonacci sequence up to {}th term in {} ms",
            sequence_length,
            now.elapsed().as_millis()
        );

        FibExample {
            options,
            sequence_length,
            result,
            _hasher: PhantomData,
        }
    }
}

// EXAMPLE IMPLEMENTATION
// ================================================================================================

impl<H: ElementHasher> Example for FibExample<H>
where
    H: ElementHasher<BaseField = BaseElement>,
{
    fn prove(&self) -> StarkProof {
        debug!(
            "Generating proof for computing Fibonacci sequence (2 terms per step) up to {}th term\n\
            ---------------------",
            self.sequence_length
        );

        // create a prover
        let prover = FibSmallProver::<H>::new(self.options.clone());

        // generate execution trace
        let now = Instant::now();
        let trace = prover.build_trace(self.sequence_length);

        let trace_width = trace.width();
        let trace_length = trace.length();
        debug!(
            "Generated execution trace of {} registers and 2^{} steps in {} ms",
            trace_width,
            log2(trace_length),
            now.elapsed().as_millis()
        );

        // generate the proof
        prover.prove(trace).unwrap()
    }

    fn verify(&self, proof: StarkProof) -> Result<(), VerifierError> {
        winterfell::verify::<FibSmall, H>(proof, self.result)
    }

    fn verify_with_wrong_inputs(&self, proof: StarkProof) -> Result<(), VerifierError> {
        winterfell::verify::<FibSmall, H>(proof, self.result + BaseElement::ONE)
    }
}
