// Encoding/erasure decoding for Reed-Solomon codes over binary extension fields
// Author: Sian-Jheng Lin (King Abdullah University of Science and Technology (KAUST), email: sianjheng.lin@kaust.edu.sa)

// This program is the implementation of
// Lin, Han and Chung, "Novel Polynomial Basis and Its Application to Reed-Solomon Erasure Codes," FOCS14.
// (http://arxiv.org/abs/1404.3458)
use super::*;

use rand::distributions::{Distribution, Uniform};
use rand::RngCore;

type GFSymbol = u16;

const FIELD_BITS: usize = 16;

const GENERATOR: GFSymbol = 0x2D; //x^16 + x^5 + x^3 + x^2 + 1

//Cantor basis
const BASE: [GFSymbol; FIELD_BITS] =
	[1_u16, 44234, 15374, 5694, 50562, 60718, 37196, 16402, 27800, 4312, 27250, 47360, 64952, 64308, 65336, 39198];

const FIELD_SIZE: usize = 1_usize << FIELD_BITS;

const MODULO: GFSymbol = (FIELD_SIZE - 1) as GFSymbol;

static mut LOG_TABLE: [GFSymbol; FIELD_SIZE] = [0_u16; FIELD_SIZE];
static mut EXP_TABLE: [GFSymbol; FIELD_SIZE] = [0_u16; FIELD_SIZE];

//-----Used in decoding procedure-------
//twisted factors used in FFT
static mut SKEW_FACTOR: [GFSymbol; MODULO as usize] = [0_u16; MODULO as usize];

//factors used in formal derivative
static mut B: [GFSymbol; FIELD_SIZE >> 1] = [0_u16; FIELD_SIZE >> 1];

//factors used in the evaluation of the error locator polynomial
static mut LOG_WALSH: [GFSymbol; FIELD_SIZE] = [0_u16; FIELD_SIZE];

//return a*EXP_TABLE[b] over GF(2^r)
fn mul_table(a: GFSymbol, b: GFSymbol) -> GFSymbol {
	if a != 0_u16 {
		unsafe {
			let offset = (LOG_TABLE[a as usize] as u32 + b as u32 & MODULO as u32)
				+ (LOG_TABLE[a as usize] as u32 + b as u32 >> FIELD_BITS);
			EXP_TABLE[offset as usize]
		}
	} else {
		0_u16
	}
}

const fn log2(mut x: GFSymbol) -> GFSymbol {
	let mut o = 0;
	while x > 1 {
		x >>= 1;
		o += 1;
	}
	o
}

//fast Walsh–Hadamard transform over modulo mod
fn walsh(data: &mut [GFSymbol], size: usize) {
	let mut depart_no = 1_usize;
	while depart_no < size {
		let mut j = 0;
		while j < size {
			for i in j..(depart_no + j) {
				let tmp2: u32 = data[i] as u32 + MODULO as u32 - data[i + depart_no] as u32;
				data[i] = ((data[i] as u32 + data[i + depart_no] as u32 & MODULO as u32)
					+ (data[i] as u32 + data[i + depart_no] as u32 >> FIELD_BITS)) as GFSymbol;
				data[i + depart_no] = ((tmp2 & MODULO as u32) + (tmp2 >> FIELD_BITS)) as GFSymbol;
			}
			j += depart_no << 1;
		}
		depart_no <<= 1;
	}
}

//formal derivative of polynomial in the new basis
fn formal_derivative(cos: &mut [GFSymbol], size: usize) {
	for i in 1..size {
		let length = ((i ^ i - 1) + 1) >> 1;
		for j in (i - length)..i {
			cos[j] ^= cos[j + length];
		}
	}
	let mut i = size;
	while i < FIELD_SIZE {
		for j in 0..size {
			cos[j] ^= cos[j + i];
		}
		i <<= 1;
	}
}

//IFFT in the proposed basis
fn inverse_fft_in_novel_poly_basis(data: &mut [GFSymbol], size: usize, index: usize) {
	let mut depart_no = 1_usize;
	while depart_no < size {
		let mut j = depart_no;
		while j < size {
			for i in (j - depart_no)..j {
				data[i + depart_no] ^= data[i];
			}

			let mut skew = unsafe { SKEW_FACTOR[j + index - 1] };
			if skew != MODULO {
				for i in (j - depart_no)..j {
					data[i] ^= mul_table(data[i + depart_no], skew);
				}
			}

			j += depart_no << 1;
		}
		depart_no <<= 1;
	}
}

//FFT in the proposed basis
fn fft_in_novel_poly_basis(data: &mut [GFSymbol], size: usize, index: usize) {
	let mut depart_no = size >> 1_usize;
	while depart_no > 0 {
		let mut j = depart_no;
		while j < size {
			let mut skew = unsafe { SKEW_FACTOR[j + index - 1] };
			if skew != MODULO {
				for i in (j - depart_no)..j {
					data[i] ^= mul_table(data[i + depart_no], skew);
				}
			}
			for i in (j - depart_no)..j {
				data[i + depart_no] ^= data[i];
			}
			j += depart_no << 1;
		}
		depart_no >>= 1;
	}
	return;
}

//initialize LOG_TABLE[], EXP_TABLE[]
unsafe fn init() {
	let mas: GFSymbol = (1 << FIELD_BITS - 1) - 1;
	let mut state: usize = 1;
	for i in 0_usize..(MODULO as usize) {
		EXP_TABLE[state] = i as GFSymbol;
		if (state >> FIELD_BITS - 1) != 0 {
			state &= mas as usize;
			state = state << 1_usize ^ GENERATOR as usize;
		} else {
			state <<= 1;
		}
	}
	EXP_TABLE[0] = MODULO;

	LOG_TABLE[0] = 0;
	for i in 0..FIELD_BITS {
		for j in 0..(1 << i) {
			LOG_TABLE[j + (1 << i)] = LOG_TABLE[j] ^ BASE[i];
		}
	}
	for i in 0..FIELD_SIZE {
		LOG_TABLE[i] = EXP_TABLE[LOG_TABLE[i] as usize];
	}

	for i in 0..FIELD_SIZE {
		EXP_TABLE[LOG_TABLE[i] as usize] = i as GFSymbol;
	}
	EXP_TABLE[MODULO as usize] = EXP_TABLE[0];
}

//initialize SKEW_FACTOR[], B[], LOG_WALSH[]
unsafe fn init_dec() {
	let mut base: [GFSymbol; FIELD_BITS - 1] = Default::default();

	for i in 1..FIELD_BITS {
		base[i - 1] = 1 << i;
	}

	for m in 0..(FIELD_BITS - 1) {
		let step = 1 << (m + 1);
		SKEW_FACTOR[(1 << m) - 1] = 0;
		for i in m..(FIELD_BITS - 1) {
			let s = 1 << (i + 1);

			let mut j = (1 << m) - 1;
			while j < s {
				SKEW_FACTOR[j + s] = SKEW_FACTOR[j] ^ base[i];
				j += step;
			}
		}

		let idx = mul_table(base[m], LOG_TABLE[(base[m] ^ 1_u16) as usize]);
		base[m] = MODULO - LOG_TABLE[idx as usize];

		for i in (m + 1)..(FIELD_BITS - 1) {
			let b = (LOG_TABLE[(base[i] as u16 ^ 1_u16) as usize] as u32 + base[m] as u32);
			let b = b % MODULO as u32;
			base[i] = mul_table(base[i], b as u16);
		}
	}
	for i in 0..(MODULO as usize) {
		SKEW_FACTOR[i] = LOG_TABLE[SKEW_FACTOR[i] as usize];
	}

	base[0] = MODULO - base[0];
	for i in 1..(FIELD_BITS - 1) {
		base[i] = (MODULO - base[i] + base[i - 1]) % MODULO;
	}

	B[0] = 0;
	for i in 0..(FIELD_BITS - 1) {
		let depart = 1 << i;
		for j in 0..depart {
			B[j + depart] = (B[j] + base[i]) % MODULO;
		}
	}

	mem_cpy(&mut LOG_WALSH[..], &LOG_TABLE[..]);
	LOG_WALSH[0] = 0;
	walsh(&mut LOG_WALSH[..], FIELD_SIZE);
}

//Encoding alg for k/n<0.5: message is a power of two
fn encode_low(data: &[GFSymbol], k: usize, codeword: &mut [GFSymbol]) {
	mem_cpy(&mut codeword[0..k], &data[0..k]);

	inverse_fft_in_novel_poly_basis(codeword, k, 0);

	let (first_k, skip_first_k) = codeword.split_at_mut(k);
	let mut i = 0;
	while i < FIELD_SIZE {
		// mem_cpy(&mut codeword[i..(i+k)], &codeword[0..k]);
		// fft_in_novel_poly_basis(&mut codeword[i..(i+k)], k, i);
		mem_cpy(&mut skip_first_k[i..(i + k)], first_k);
		fft_in_novel_poly_basis(&mut skip_first_k[i..(i + k)], k, i + k);
		i += k;
	}

	mem_cpy(&mut codeword[0..k], &data[0..k]);
}

fn mem_zero(zerome: &mut [GFSymbol]) {
	for i in 0..zerome.len() {
		zerome[i] = 0_u16;
	}
}

fn mem_cpy(dest: &mut [GFSymbol], src: &[GFSymbol]) {
	let sl = src.len();
	debug_assert_eq!(dest.len(), sl);
	for i in 0..sl {
		dest[i] = src[i];
	}
}

//data: message array. parity: parity array. mem: buffer(size>= n-k)
//Encoding alg for k/n>0.5: parity is a power of two.
fn encode_high(data: &[GFSymbol], k: usize, parity: &mut [GFSymbol], mem: &mut [GFSymbol]) {
	let t: usize = FIELD_SIZE - k;

	mem_zero(&mut parity[0..t]);

	let mut i = t;
	while i < FIELD_SIZE {
		mem_cpy(&mut mem[..t], &data[(i - t)..t]);

		inverse_fft_in_novel_poly_basis(mem, t, i);
		for j in 0..t {
			parity[j] ^= mem[j];
		}
		i += t;
	}
	fft_in_novel_poly_basis(parity, t, 0);
}

//Compute the evaluations of the error locator polynomial
fn decode_init(erasure: &[bool], log_walsh2: &mut [GFSymbol]) {
	for i in 0..FIELD_SIZE {
		log_walsh2[i] = erasure[i] as u16;
	}
	walsh(log_walsh2, FIELD_SIZE);
	for i in 0..FIELD_SIZE {
		log_walsh2[i] = (log_walsh2[i] as usize * unsafe { LOG_WALSH[i] } as usize % MODULO as usize) as GFSymbol;
	}
	walsh(log_walsh2, FIELD_SIZE);
	for i in 0..FIELD_SIZE {
		if erasure[i] {
			log_walsh2[i] = MODULO - log_walsh2[i];
		}
	}
}

fn decode_main(codeword: &mut [GFSymbol], erasure: &[bool], log_walsh2: &[GFSymbol]) {
	let k2 = FIELD_SIZE; //k2 can be replaced with k
	for i in 0..FIELD_SIZE {
		codeword[i] = if erasure[i] { mul_table(codeword[i], log_walsh2[i]) } else { 0_u16 };
	}
	inverse_fft_in_novel_poly_basis(codeword, FIELD_SIZE, 0);

	let modulo = MODULO;

	//formal derivative
	let mut i = 0;
	while i < FIELD_SIZE {
		let b = unsafe { B[i >> 1] };
		codeword[i] = mul_table(codeword[i], modulo - b);
		codeword[i + 1] = mul_table(codeword[i + 1], modulo - b);
		i += 2;
	}
	formal_derivative(codeword, k2);
	let mut i = 0;
	while i < k2 {
		let b = unsafe { B[i >> 1] };
		codeword[i] = mul_table(codeword[i], b);
		codeword[i + 1] = mul_table(codeword[i + 1], b);
		i += 2;
	}

	fft_in_novel_poly_basis(codeword, k2, 0);
	for i in 0..k2 {
		codeword[i] = if erasure[i] { mul_table(codeword[i], log_walsh2[i]) } else { 0_u16 };
	}
}

pub fn encode(data: &[u8]) -> Vec<WrappedShard> {
	unsafe { init() };

	unimplemented!("foo")
}

pub fn reconstruct(mut received_shards: Vec<Option<WrappedShard>>) -> Option<Vec<u8>> {
	unsafe { init_dec() };
	unimplemented!("foo")
}

#[cfg(test)]
mod test {
	use super::*;

	/// Generate a random index
	fn rand() -> usize {
		let mut rng = rand::thread_rng();
		let uni = Uniform::<usize>::new_inclusive(0, FIELD_SIZE - 1);
		uni.sample(&mut rng)
	}

	#[test]
	fn novel_poly_basis() {
		unsafe {
			init(); //fill log table and exp table
			init_dec(); //compute factors used in erasure decoder
		}

		// message size `k`s
		let k = FIELD_SIZE / 2;
		//-----------Generating message----------
		//message array
		let mut data: [GFSymbol; FIELD_SIZE] = [0; FIELD_SIZE];

		for i in (FIELD_SIZE - k)..FIELD_SIZE {
			data[i] = rand() as GFSymbol & MODULO; //filled with random numbers
		}

		println!("Message(First n-k are zeros): ");
		for i in 0..FIELD_SIZE {
			print!("{:02x} ", data[i]);
		}
		println!("");

		//---------encoding----------
		let mut codeword = [0_u16; FIELD_SIZE];

		let (data_till_t, data_skip_t) = data.split_at_mut(FIELD_SIZE - k);
		encode_high(data_skip_t, k, data_till_t, &mut codeword[..]);
		//encode_low(data, k, codeword);

		mem_cpy(&mut codeword[..], &data[..]);

		println!("Codeword:");
		for i in 0..FIELD_SIZE {
			print!("{:02x} ", codeword[i]);
		}
		println!("");

		//--------erasure simulation---------

		//Array indicating erasures
		let mut erasure: [bool; FIELD_SIZE] = [false; FIELD_SIZE];
		for i in k..FIELD_SIZE {
			erasure[i] = true;
		}

		//permuting the erasure array
		let mut i = FIELD_SIZE - 1;
		while i > 0 {
			let pos: usize = rand() % (i + 1);
			if (i != pos) {
				erasure.swap(i, pos);
			}
			i -= 1;
		}

		for i in 0..FIELD_SIZE {
			//erasure codeword symbols
			if erasure[i] {
				codeword[i] = 0;
			}
		}

		println!("Erasure (XX is erasure):");
		for i in 0..FIELD_SIZE {
			if erasure[i] {
				print!("XX ");
			} else {
				print!("{:02x} ", codeword[i]);
			}
		}
		println!("");

		//---------Erasure decoding----------------
		let mut log_walsh2: [GFSymbol; FIELD_SIZE] = [0_u16; FIELD_SIZE];
		decode_init(&erasure[..], &mut log_walsh2[..]); //Evaluate error locator polynomial
												//---------main processing----------
		decode_main(&mut codeword[..], &erasure[..], &log_walsh2[..]);

		println!("Decoded result:");
		for i in 0..FIELD_SIZE {
			if erasure[i] {
				print!("{:02x}", codeword[i]);
			} else {
				print!("XX ");
			};
		}
		println!("");

		for i in 0..FIELD_SIZE {
			//Check the correctness of the result
			if erasure[i] {
				if (data[i] != codeword[i]) {
					println!("Decoding Error!");
					return;
				}
			}
		}
		println!("Decoding is successful!");
		return;
	}
}
