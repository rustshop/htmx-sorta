use std::hash::Hasher;

/// A trivial (hopefully fast) hasher that just XORs the data into the final
/// hash
#[derive(Default, Clone, Copy)]
pub struct XorHasher {
    hash: u64,
}

impl Hasher for XorHasher {
    fn finish(&self) -> u64 {
        self.hash
    }

    fn write(&mut self, bytes: &[u8]) {
        for u in bytes_to_u64s(bytes) {
            self.hash ^= u;
        }
    }
}

fn bytes_to_u64s(data: &[u8]) -> impl Iterator<Item = u64> + '_ {
    let full_chunks = data.chunks_exact(8).map(|chunk| {
        let arr = [
            chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7],
        ];
        u64::from_le_bytes(arr)
    });

    let remainder = data.chunks_exact(8).remainder();
    let mut remainder_val = 0u64;
    for (i, &byte) in remainder.iter().enumerate() {
        remainder_val |= (byte as u64) << (i * 8);
    }

    full_chunks
        .chain(std::iter::once(remainder_val))
        .filter(|&val| val != 0)
}

#[test]
fn bytes_to_u64s_test() {
    assert_eq!(
        bytes_to_u64s(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]).collect::<Vec<_>>(),
        vec![0x807060504030201, 0xb0a09]
    );
    assert_eq!(bytes_to_u64s(&[1]).collect::<Vec<_>>(), vec![0x01]);
}
