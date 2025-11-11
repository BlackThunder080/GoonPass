pub fn hash(message: &[u8]) -> [u8; 32] {
    let mut hash = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];

    let (chunks, remainder) = message.as_chunks::<64>();
    for block in chunks {
        hash_block(block, &mut hash);
    }

    let len = remainder.len();

    if len == 0 {
        let mut block = [0; 64];
        block[55] = 0x80;
        hash_block(&block, &mut hash);
    } else if len < 56 {
        let mut block = [0; 64];
        block[..len].copy_from_slice(remainder);
        block[len] = 0x80;
        block[56..].copy_from_slice(&(len as u64 * 8).to_be_bytes());

        hash_block(&block, &mut hash);
    } else if len == 56 {
        let mut block = [0; 64];
        block[..len].copy_from_slice(remainder);
        block[len] = 0x80;
        hash_block(&block, &mut hash);

        let mut block = [0; 64];
        block[56..].copy_from_slice(&(len as u64 * 8).to_be_bytes());
        hash_block(&block, &mut hash);
    }

    let mut digest = [0; 32];
    for (i, word) in digest.chunks_exact_mut(4).enumerate() {
        word.copy_from_slice(&hash[i].to_be_bytes());
    }

    digest
}

fn hash_block(block: &[u8; 64], hash: &mut [u32; 8]) {
    let mut w = [0_u32; 64];

    for i in 0..16 {
        w[i] = ((block[i * 4 + 0] as u32) << 24)
            | ((block[i * 4 + 1] as u32) << 16)
            | ((block[i * 4 + 2] as u32) << 8)
            | ((block[i * 4 + 3] as u32) << 0);
    }

    for i in 16..64 {
        w[i] = sigma1l(w[i - 2])
            .wrapping_add(w[i - 7])
            .wrapping_add(sigma0l(w[i - 15]))
            .wrapping_add(w[i - 16]);
    }

    let mut a = hash[0];
    let mut b = hash[1];
    let mut c = hash[2];
    let mut d = hash[3];
    let mut e = hash[4];
    let mut f = hash[5];
    let mut g = hash[6];
    let mut h = hash[7];

    for round in 0..64 {
        let t1 = h
            .wrapping_add(sigma1u(e))
            .wrapping_add(ch(e, f, g))
            .wrapping_add(K[round])
            .wrapping_add(w[round]);

        let t2 = sigma0u(a).wrapping_add(maj(a, b, c));

        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(t1);
        d = c;
        c = b;
        b = a;
        a = t1.wrapping_add(t2);
    }

    hash[0] = hash[0].wrapping_add(a);
    hash[1] = hash[1].wrapping_add(b);
    hash[2] = hash[2].wrapping_add(c);
    hash[3] = hash[3].wrapping_add(d);
    hash[4] = hash[4].wrapping_add(e);
    hash[5] = hash[5].wrapping_add(f);
    hash[6] = hash[6].wrapping_add(g);
    hash[7] = hash[7].wrapping_add(h);
}

const fn ch(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (!x & z)
}

const fn maj(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (x & z) ^ (y & z)
}

const fn sigma0u(x: u32) -> u32 {
    x.rotate_right(2) ^ x.rotate_right(13) ^ x.rotate_right(22)
}

const fn sigma1u(x: u32) -> u32 {
    x.rotate_right(6) ^ x.rotate_right(11) ^ x.rotate_right(25)
}

const fn sigma0l(x: u32) -> u32 {
    x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3)
}

const fn sigma1l(x: u32) -> u32 {
    x.rotate_right(17) ^ x.rotate_right(19) ^ (x >> 10)
}

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

#[test]
fn test() {
    assert_eq!(
        hash(&[0x61, 0x62, 0x63]),
        [
            0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae,
            0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61,
            0xf2, 0x00, 0x15, 0xad
        ]
    );
}
