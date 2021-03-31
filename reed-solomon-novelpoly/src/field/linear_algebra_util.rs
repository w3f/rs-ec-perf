use bit_vec::BitVec;

// fn main() {
// let mut m1: Vec<Vec<f64>> = vec![vec![1.0,2.0],vec![3.0,4.0]];
//     let mut r_m1 = &mut m1;
//     let rr_m1 = &mut r_m1;
//     let mut m2: Vec<Vec<f64>> = vec![vec![1.0, 2.0, 3.0, 4.0], vec![4.0, 5.0, 6.0, 7.0], vec![7.0, 8.0, 9.0, 10.0], vec![10.0, 11.0, 12.0, 13.0]];
//     let mut r_m2 = &mut m2;
//     let rr_m2 = &mut r_m2;
//     let mut m3: Vec<Vec<f64>> = vec![vec![0.0, 1.0, 2.0, 3.0, 4.0],
//                                 vec![5.0, 6.0, 7.0, 8.0, 9.0],
//                                 vec![10.0, 11.0, 12.0, 13.0, 14.0],
//                                 vec![15.0, 16.0, 17.0, 18.0, 19.0], 
//                                 vec![20.0, 21.0, 22.0, 23.0, 24.0]];
//     let mut r_m3 = &mut m3;
//     let rr_m3 = &mut r_m3;
// 
//     println!("Determinant of m1: {}", determinant(rr_m1));
//     println!("Permanent of m1: {}", permanent(rr_m1));
// 
//     println!("Determinant of m2: {}", determinant(rr_m2));
//     println!("Permanent of m2: {}", permanent(rr_m2));
//
//     println!("Determinant of m3: {}", determinant(rr_m3));
//     println!("Permanent of m3: {}", permanent(rr_m3));
// 
// 
//}

//auxalury functions
#[inline]
fn bool_to_u8(bin_bool_val: bool) -> u8 {
    if bin_bool_val { 1 } else { 0 }
}

// pub fn bit_vec_u8_from_elem(nbits: usize, bit: bool) -> BitVec<u8> {
//     let nblocks = nbits / 8;

//     let extra_bits = nbits % 8;
//     if extra_bits > 0 {
//         nblocks += 1;
//     }

//     let mask = ((1 as u8)  << extra_bits) - 1 as u8;
//     let storage = vec![if bit { !(0 as u8) } else { (0 as u8) }; nblocks];
//     //fix last byte
//     if extra_bits > 0 {
//         storage[nblocks - 1] &= mask;
//     }
    
//     let mut bit_vec = BitVec {
//         storage: storage,
//         nbits,
//     };
    
//     bit_vec
// }

pub fn minor( a: &mut Vec<BitVec>, x: usize, y: usize) ->  Vec<BitVec> {
    let mut out_vec: Vec<BitVec> = vec![BitVec::from_elem(a.len() -1, false); a.len() -1];
    for i in 0..a.len()-1 {
        for j in 0..a.len()-1 {
            match () {
                _ if (i < x && j < y) => {
                    out_vec[i].set(j,a[i][j]);
                },
                _ if (i >= x && j < y) => {
                    out_vec[i].set(j, a[i + 1][j]);
                },
                _ if (i < x && j >= y) => {
                    out_vec[i].set(j, a[i][j + 1]);
                },
                _ => { //i > x && j > y 
                    out_vec[i].set(j, a[i + 1][j + 1]);
                },
            }
        }
    }
    out_vec
}

pub fn determinant (matrix: &mut Vec<BitVec>) -> bool {
    match () {
        _ if (matrix.len() == 1) => {
            matrix[0][0]
        },
        _ => {
            let mut sign : u8 = 1;
            let mut sum = 0;
            for i in 0..matrix.len() {
                sum = sum + sign * bool_to_u8(matrix[0][i]) * bool_to_u8(determinant(&mut minor(matrix, 0, i)));
                sign = sign * 1; //-1 == 1 mod 2
            }
            sum % 2 != 0
        }
    }
}
