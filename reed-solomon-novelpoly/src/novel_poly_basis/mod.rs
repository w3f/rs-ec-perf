// Encoding/erasure decoding for Reed-Solomon codes over binary extension fields
//
// Derived impl of `RSAErasureCode.c`.
//
// Lin, Han and Chung, "Novel Polynomial Basis and Its Application to Reed-Solomon Erasure Codes," FOCS14.
// (http://arxiv.org/abs/1404.3458)

use std::marker::PhantomData;
use std::convert::TryInto;
use core::ops::{Mul};

use crate::errors::*;
use crate::{Shard};
use reed_solomon_field::{FieldAdd, TruncateTo, Logarithm, Additive, walsh, AfftField, tweaked_formal_derivative};

pub mod availability_util;
pub use availability_util::*;

//use crate::shard::ShardHold;
pub use super::util::*;
/// Reed-Solomon erasure code encoder/decoder.
/// # Example
///
/// let r: ReedSolomon<Field> = ReedSolomon::new(3, 2).unwrap();
///
#[derive(Debug)]
pub struct ReedSolomon<F: AfftField> where
   <F::Wide as TryInto<F::Element>>::Error : core::fmt::Debug,
    [(); F::FIELD_BYTES]: Sized,
{
    /// Avoid copying unnecessary chunks.
    wanted_n: usize,
    /// total number of message symbols to send
    /// Invariant is a power of base 2
    n: usize,
    /// number of information containing chunks
    /// Invariant is a power of base 2, `k < n`
    k: usize,
    _marker: PhantomData<*const F>,
}

impl<F: AfftField> ReedSolomon<F>
where
[(); F::FIELD_BYTES]: Sized,
[(); F::ONEMASK_USIZE]: Sized,
[(); F::FIELD_SIZE >> 1]: Sized,
<F::Wide as TryInto<F::Element>>::Error : core::fmt::Debug
{
    /// Returns the total number of data shard
    /// consumed by the code. That is equal the total number of symbols
    /// can be encoded in a one block of code.
    /// current algorithm always expect that this number is a power of 2
    pub fn get_number_of_data_shards(&self) -> usize {
        self.k
    }

    /// Returns the total number of encoded shards. The number of encoded shards
    /// computed by the algorithm internally is always power of 2 but it only
    /// hands over only as many as requested shards.
    pub fn get_number_of_all_shards(&self) -> usize {
        self.wanted_n
    }

    /// Returns the size per shard in bytes
    pub fn shard_len(&self, payload_size: usize) -> usize {
        let payload_symbols = (payload_size + 1) / 2;
        let shard_symbols_ceil = (payload_symbols + self.k - 1) / self.k;
        let shard_bytes = shard_symbols_ceil * 2;
        shard_bytes
    }

    /// `k` the intended number of data shards needed to recover.
    /// `n` the intended number of resulting shards.
    ///
    /// Assures that the derived paramters retain at most the given coding
    /// rate, and as such assure recoverability with at least an equiv fraction
    /// as provided by the input `n`, and `k` parameterset.
    pub(crate) fn new(n: usize, k: usize) -> Result<Self> {
        if n < 2 {
            return Err(Error::WantedShardCountTooLow(n));
        }
        if k < 1 {
            return Err(Error::WantedPayloadShardCountTooLow(k));
        }
        let k_po2 = next_lower_power_of_2(k);
        let n_po2 = next_higher_power_of_2(n);
        // If the coding rate of the power of 2 variants, is higher,
        // we would have to lower k by one order of magnitude base 2
        // which is true by definition
        assert!(n * k_po2 <= n_po2 * k);

        if n_po2 > F::FIELD_SIZE as usize {
            return Err(Error::WantedShardCountTooHigh(n));
        }

        // make a reed-solomon instance.
        Ok(Self { wanted_n: n, n: n_po2, k: k_po2, _marker: PhantomData })
    }

    pub fn encode<S: Shard<F>>(&self, bytes: &[u8]) -> Result<Vec<S>> {

        println!("Original data");
        println!("{:?}", bytes);
        
        if bytes.is_empty() {
            return Err(Error::PayloadSizeIsZero);
        }

        // setup the shards, n is likely _larger_, so use the truely required number of shards

        // required shard length in bytes, rounded to full symbols
        let shard_len = self.shard_len(bytes.len());
        assert!(shard_len > 0);
        // collect all sub encoding runs

        let validator_count = self.wanted_n;
        let k2 = self.k * 2;
        // prepare one wrapped shard per validator
        let mut shards = vec![
            <S as From<Vec<u8>>>::from({
                let mut v = Vec::<u8>::with_capacity(shard_len);
                unsafe { v.set_len(shard_len) }
                v
            });
            validator_count
        ];

        for (chunk_idx, i) in (0..bytes.len()).into_iter().step_by(k2).enumerate() {
            let end = std::cmp::min(i + k2, bytes.len());
            assert_ne!(i, end);
            let data_piece = &bytes[i..end];
            assert!(!data_piece.is_empty());
            assert!(data_piece.len() <= k2);
            let encoding_run = self.encode_sub(data_piece)?;
            println!("encoding_run at encode: {:?}", encoding_run);
            for val_idx in 0..validator_count {
                shards[val_idx].set_chunk(
                    chunk_idx,
                    encoding_run[val_idx].to_be_bytes()[..]
                        .try_into()
                        .expect("F::FIELD_BYTES and FieldAdd::FIELD_BYTES are the same. q.e.d"),
                );

                println!("shard evolution at encode");
                Self::print_shards(&shards);

            }
        }

        println!("original encoded shards at encode");
        Self::print_shards(&shards);
        Ok(shards)
    }

    ///Verifies if all shards have the same length and they can
    ///be propely converted to a slice of underlying field elements
    ///return the uniform shard length
    fn verify_reconstructiblity<S: Shard<F>>(
        &self,
        received_shards: &Vec<Option<S>>,
    ) -> Result<usize> {
        //if all shards empty there is nothig to reconstruct hence reject.
        let maybe_first_available_shard =
            AsRef::<[Option<S>]>::as_ref(&received_shards).iter().find(|optional_shard| match optional_shard {
                Some(_) => true,
                None => false,
            });
        let first_available_shard = match maybe_first_available_shard.as_ref() {
            None => Err(Error::PayloadSizeIsZero)?,
            Some(first_available_shard) => {
                first_available_shard.as_ref().expect("Already has checked it is not none. q.e.d")
            }
        };

        let uniform_shard_len = AsRef::<[u8]>::as_ref(&first_available_shard).len();

        if uniform_shard_len == 0 {
            Err(Error::ZeroLengthShards)?;
        }

        if uniform_shard_len % F::FIELD_BYTES != 0 {
            Err(Error::UndivisableShardLength { shard_length: uniform_shard_len, field_bytes: F::FIELD_BYTES })?;
        }

        for optional_shard in received_shards {
            //  { AsRef::<[Option<S>]>.as_ref(&self)
            match optional_shard {
                Some(shard) => {
                    if AsRef::<[u8]>::as_ref(&shard).len() != uniform_shard_len {
                        Err(Error::InconsistentShardLengths {
                            first: uniform_shard_len,
                            other: AsRef::<[u8]>::as_ref(&shard).len(),
                        })?;
                    }
                }
                _ => (),
            }
        }

        return Ok(uniform_shard_len / F::FIELD_BYTES);
    }

    ///make set of the shard to have exactly as many shard as
    ///the number of symbols in an encoded word, by either adding
    ///empty shards or removing extra shards.
    fn equalize_shards_number_with_code_block_length<'a, S: Shard<F>>(
        &self,
        received_shards: Vec<Option<S>>,
    ) -> Vec<Option<S>> {
        let code_block_length = self.n;
        let gap = code_block_length.saturating_sub(received_shards.len()); //== max(code_block_length - self.as_ref().len(), 0): minimum number of missing shards, some received shard might be None

        //This might be too naive you might be removing none empty shards and leaving empty shards in place. nonetheless given the placement of the shard in the slice are important it is not possible to rescue beyond block length data without major restructuring of the reconstruction code
        received_shards
            .into_iter()
            .take(code_block_length)
            .chain(std::iter::repeat(None).take(gap))
            .collect::<Vec<Option<S>>>()
    }

    /// each shard contains one symbol of one run of erasure coding
    pub fn reconstruct<S: Shard<F>>(&self, received_shards: Vec<Option<S>>) -> Result<Vec<u8>>
    where
        F: FieldAdd,
        [(); F::FIELD_SIZE]: Sized,
    {
        //println!("original erased shards: {:?}", erasures);
        println!("original encoded shards at reconstrct:");
        Self::print_optional_shards(&received_shards);

        let shard_len_in_syms = self.verify_reconstructiblity(&received_shards)?;

        let received_shards = self.equalize_shards_number_with_code_block_length(received_shards);

        println!("extended encoded shards at reconstrct:");
        Self::print_optional_shards(&received_shards);

        assert_eq!(received_shards.len(), self.n);

        // must be collected after expanding `received_shards` to the anticipated size
        let mut existential_count = 0_usize;
        let erasures = received_shards
            .iter()
            .map(|x| x.is_none())
            .inspect(|erased| existential_count += !*erased as usize)
            .collect::<Vec<bool>>();

        println!("erased shards: {:?}", erasures);

        if existential_count < self.k {
             return Err(Error::NeedMoreShards { have: existential_count, min: self.k, all: self.n });
        }

        // //Evaluate error locator polynomial only once
        let mut error_poly_in_log = [Logarithm::<F>(F::ZERO_ELEMENT); F::FIELD_SIZE];
        let mut error_poly_in_log = vec![Logarithm::<F>(F::ZERO_ELEMENT); F::FIELD_SIZE];
        self.eval_error_polynomial(&erasures[..], &mut error_poly_in_log[..]);

        let mut acc = Vec::<u8>::with_capacity(shard_len_in_syms * 2 * self.k);
        for i in 0..shard_len_in_syms {
             // take the i-th element of all shards and try to recover
             let decoding_run = received_shards
                 .iter()
                 .map(|x| {
                     x.as_ref().map(|x| {
                        //let z = x.get_chunk(i);
                        let z = AsRef::<[[u8; <F as FieldAdd>::FIELD_BYTES]]>::as_ref(&x)[i];
                        //let z1 : [u8; Additive::FIELD_BYTES] = [z[0], z[1]];                        
                        Additive::<F>::from_be_bytes(z[..].try_into().expect("F::FIELD_BYTES and FieldAdd>::FIELD_BYTES are the same. q.e.d"))
                    })
                })
                .collect::<Vec<Option<Additive<F>>>>();

            println!("received_shards as additive at reconstruct: {:?}", decoding_run);

            assert_eq!(decoding_run.len(), self.n);

            // reconstruct from one set of symbols which was spread over all erasure chunks
            let piece = self.reconstruct_sub(&decoding_run[..], &erasures, &error_poly_in_log).unwrap();
            acc.extend_from_slice(&piece[..]);
        }

        println!("Reconstructed data");
        println!("{:?}", acc);


        Ok(acc)
    }

    /// Encoding alg for k/n < 0.5: message is a power of two
    pub fn encode_low(&self, data: &[Additive<F>], codeword: &mut [Additive<F>]) {
        assert!(self.k + self.k <= self.n);
        assert_eq!(codeword.len(), self.n);
        assert_eq!(data.len(), self.n);

        assert!(is_power_of_2(self.n));
        assert!(is_power_of_2(self.k));

        // k | n is guaranteed
        assert_eq!((self.n / self.k) * self.k, self.n);

        // move the data to the codeword
        codeword.copy_from_slice(data);

        // split after the first k
        let (codeword_first_k, codeword_skip_first_k) = codeword.split_at_mut(self.k);

        F::inverse_afft(codeword_first_k, self.k, 0);

        // the first codeword is now the basis for the remaining transforms
        // denoted `M_topdash`

        for shift in (self.k..self.n).into_iter().step_by(self.k) {
            let codeword_at_shift = &mut codeword_skip_first_k[(shift - self.k)..shift];
            // copy `M_topdash` to the position we are currently at, the n transform
            codeword_at_shift.copy_from_slice(codeword_first_k);
            F::afft(codeword_at_shift, self.k, shift);
        }

        // restore `M` from the derived ones
        (&mut codeword[0..self.k]).copy_from_slice(&data[0..self.k]);
    }

    // TODO: Make encode_high actually work again!  Add tests!

    //data: message array. parity: parity array. mem: buffer(size>= n-k)
    //Encoding alg for k/n>0.5: parity is a power of two.
    pub fn encode_high(&self, data: &[Additive<F>], parity: &mut [Additive<F>], mem: &mut [Additive<F>]) {
        let t: usize = self.n - self.k;

        // mem_zero(&mut parity[0..t]);
        for i in 0..t {
            parity[i] = Additive::<F>(F::ZERO_ELEMENT);
        }

        let mut i = t;
        while i < self.n {
            (&mut mem[..t]).copy_from_slice(&data[(i - t)..t]);

            F::inverse_afft(mem, t, i);
            for j in 0..t {
                parity[j] ^= mem[j];
            }
            i += t;
        }
        F::afft(parity, t, 0);
    }

    /// Bytes shall only contain payload data
    pub fn encode_sub(&self, bytes: &[u8]) -> Result<Vec<Additive<F>>> {
        assert!(is_power_of_2(self.n), "Algorithm only works for 2^i sizes for N");
        assert!(is_power_of_2(self.k), "Algorithm only works for 2^i sizes for K");
        assert!(bytes.len() <= self.k << 1);
        assert!(self.k <= self.n / 2);

        // must be power of 2
        let dl = bytes.len();
        let l = if is_power_of_2(dl) {
            dl
        } else {
            let l = log2(dl);
            let l = 1 << l;
            let l = if l >= dl { l } else { l << 1 };
            l
        };
        assert!(is_power_of_2(l));
        assert!(l >= dl);

        // tuple_windows are only used here
        use itertools::Itertools;

        // pad the incoming bytes with trailing 0s
        // so we get a buffer of size `N` in `GF` symbols
        let zero_bytes_to_add = self.n * 2 - dl;
        let data: Vec<Additive<F>> = vec![Additive::<F>(F::ZERO_ELEMENT); self.n];
        let data: Vec<Additive<F>> = bytes
            .into_iter()
            .copied()
            .chain(std::iter::repeat(0u8).take(zero_bytes_to_add)).collect::<Vec<u8>>()
            .array_windows::<{ F::FIELD_BYTES }>()
            .step_by(F::FIELD_BYTES)
            .map(|a| Additive::<F>(F::from_be_bytes_to_element(*a)))
            .collect::<Vec<Additive<F>>>();

        println!("data before being encoded {:?}", data);

        // update new data bytes with zero padded bytes
        // `l` is now `GF(2^16)` symbols
        let l = data.len();
        assert_eq!(l, self.n);

        let mut codeword = data.clone();
        assert_eq!(codeword.len(), self.n);

        self.encode_low(&data[..], &mut codeword[..]);

        Ok(codeword)
    }

    pub fn reconstruct_sub(
        &self,
        codewords: &[Option<Additive<F>>],
        erasures: &[bool],
        error_poly: &[Logarithm<F>],
        //error_poly: &[Logarithm<F>; F::FIELD_SIZE],
    ) -> Result<Vec<u8>> {
        assert!(is_power_of_2(self.n), "Algorithm only works for 2^i sizes for N");
        assert!(is_power_of_2(self.k), "Algorithm only works for 2^i sizes for K");
        assert_eq!(codewords.len(), self.n);
        assert!(self.k <= self.n / 2);
        // The recovered _payload_ chunks AND parity chunks
        let mut recovered = vec![Additive::<F>(F::ZERO_ELEMENT); self.k];

        // get rid of all `None`s
        let mut codeword = codewords
            .into_iter()
            .enumerate()
            .map(|(idx, sym)| {
                // fill the gaps with `0_u16` codewords
                if let Some(sym) = sym {
                    (idx, *sym)
                } else {
                    (idx, Additive::<F>(F::ZERO_ELEMENT))
                }
            })
            .map(|(idx, codeword)| {
                if idx < recovered.len() {
                    recovered[idx] = codeword;
                }
                codeword
            })
            .collect::<Vec<Additive<F>>>();

        // filled up the remaining spots with 0s
        assert_eq!(codeword.len(), self.n);

        //---------Erasure decoding----------------

        self.decode_main(&mut codeword[..], &erasures[..], &error_poly[..]);

        for idx in 0..self.k {
            if erasures[idx] {
                recovered[idx] = codeword[idx];
            };
        }

        println!("recoverd at reconst sub {:?}", recovered);

        let mut recovered_bytes = Vec::with_capacity(self.k * 2);
        recovered.into_iter().take(self.k).for_each(|x| recovered_bytes.extend_from_slice(&F::from_element_to_be_bytes(x.0)[..]));
        Ok(recovered_bytes)
    }

    /// recover determines how many shards to recover (starting from 0)
    // technically we only need to recover
    // the first `k` instead of all `n` which
    // would include parity chunks.
    pub(crate) fn decode_main(&self, codeword: &mut [Additive<F>], erasure: &[bool], log_walsh2: &[Logarithm<F>]) {
        assert_eq!(codeword.len(), self.n);
        assert!(self.n >= self.k);
        assert_eq!(erasure.len(), self.n);

        for i in 0..self.n {
            codeword[i] = if erasure[i] { Additive::<F>(F::ZERO_ELEMENT) } else { Mul::<Logarithm<F>>::mul(codeword[i],log_walsh2[i]) };
        }

        F::inverse_afft(codeword, self.n, 0);

        tweaked_formal_derivative(codeword, self.n);

        F::afft(codeword, self.n, 0);

        for i in 0..self.k {
            codeword[i] = if erasure[i] { codeword[i].mul(log_walsh2[i]) } else { Additive::<F>(F::ZERO_ELEMENT) };
        }
    }

    // Compute the evaluations of the error locator polynomial
    // `fn decode_init`
    // since this has only to be called once per reconstruction
    //TODO to check: This function was accepeting a parameter n but it was sent as FIELD_SIZE.
    // that looks like an unharmful bug
    pub fn eval_error_polynomial(&self, erasure: &[bool], log_walsh2: &mut [Logarithm<F>]) {
        let n = F::FIELD_SIZE;
        let z = std::cmp::min(n, erasure.len());
        for i in 0..z {
            log_walsh2[i] = Logarithm::<F>(erasure[i].into());
        }
        for i in z..n {
            log_walsh2[i] = Logarithm::<F>(F::ZERO_ELEMENT);
        }
        walsh(log_walsh2, F::FIELD_SIZE);
        for i in 0..n {
            let tmp = log_walsh2[i].to_wide() * F::LOG_WALSH[i].to_wide();
            log_walsh2[i] = Logarithm::<F>(TruncateTo::<F>::truncate(tmp % F::ONEMASK_WIDE));
        }
        walsh(log_walsh2, F::FIELD_SIZE);
        for i in 0..z {
            if erasure[i] {
                log_walsh2[i] = Logarithm::<F>(Logarithm::<F>(F::ONEMASK).0 - log_walsh2[i].0);
            }
        }
    }

    
    /// print received and recovered shard for debug reason
    fn print_shards<S: Shard<F>>(shards: &Vec<S>) {
        for (i, current_shard) in shards.iter().enumerate() {
            println!("{}: {:?}", i, current_shard);
        }
        
    }
        
    /// print received and recovered shard for debug reason
    fn print_optional_shards<S: Shard<F>>(shards: &Vec<Option<S>>) {
        for (i, current_shard) in shards.iter().enumerate() {
            println!("{}: {:?}", i, current_shard);
        }
        
    }

}

#[cfg(test)]
mod tests;

#[cfg(test)]
pub use tests::roundtrip;


