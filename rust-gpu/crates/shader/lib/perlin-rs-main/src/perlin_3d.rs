use super::*;

#[inline]
fn grad_3d(hash: usize, x: f32, y: f32, z: f32) -> f32 {
    // based on
    // http://riven8192.blogspot.com/2010/08/calculate-perlinnoise-twice-as-fast.html
    // although note that the implementation in the link has a bug - the directions
    // in section 3 of [SIGGRAPH 2002 paper](https://mrl.cs.nyu.edu/~perlin/paper445.pdf)
    // only have 1,0,-1, whereas the link has a grad "x + x", which would correspond to (2,0,0)
    // in the paper.
    match hash % 16 {
        // z = 0
        0 => x + y,   // (1,1,0)
        1 => -x + y,  // (-1,1,0)
        2 => x - y,   // (1,-1,0)
        3 => -x - y,  // (-1,-1,0)
        // y = 0
        4 => x + z,   // (1,0,1)
        5 => -x + z,  // (-1,0,1)
        6 => x - z,   // (1,0,-1)
        7 => -x - z,  // (-1,0,-1)
        // x = 0
        8 => y + z,   // (0,1,1)
        9 => -y + z,  // (0,-1,1)
        10 => y - z,  // (0,1,-1)
        11 => -y - z, // (0,-1,-1)
        // repeat 4 to be multiple of 16
        12 => x + y,   // (1,1,0)
        13 => -x + y,  // (-1,1,0)
        14 => -y + z,  // (0,-1,1)
        15 => -y + -z, // (0,-1,-1)
        _ => 0.0,      // unreachable
    }
}

#[inline]
fn perlin_3d_grad(
    x: f32,
    y: f32,
    z: f32,
    h000: usize,
    h001: usize,
    h010: usize,
    h011: usize,
    h100: usize,
    h101: usize,
    h110: usize,
    h111: usize,
) -> f32 {
    // compute the gradients
    // note: each corner has its own independent direction (derived from the permutation table)
    // g000 represents the dot product of (x,y,z) with one of the directions assigned to the corner `(0,0,0)` (e.g. `(1,1,0)`)
    let g000 = grad_3d(h000, x, y, z);
    let g001 = grad_3d(h001, x, y, z - 1.0);
    let g010 = grad_3d(h010, x, y - 1.0, z);
    let g011 = grad_3d(h011, x, y - 1.0, z - 1.0);
    let g100 = grad_3d(h100, x - 1.0, y, z);
    let g101 = grad_3d(h101, x - 1.0, y, z - 1.0);
    let g110 = grad_3d(h110, x - 1.0, y - 1.0, z);
    let g111 = grad_3d(h111, x - 1.0, y - 1.0, z - 1.0);

    // smoothed (continuous second derivative)
    let u = fade(x);
    let v = fade(y);
    let w = fade(z);

    // interpolate y over x
    let l1 = lerp(v, lerp(u, g000, g100), lerp(u, g010, g110));
    let l2 = lerp(v, lerp(u, g001, g101), lerp(u, g011, g111));

    // interpolate z over y
    lerp(w, l1, l2)
}

/// Returns the evaluation of perlin noise at position (x, y, z)
/// This function does not allocate
/// It uses the improved implementation of perlin noise
/// whose reference implementation is available here: https://mrl.cs.nyu.edu/~perlin/noise/
/// The only modification was the simplification of the function `grad` to avoid some code branches (but does not change the end result).
pub fn perlin_3d(mut x: f32, mut y: f32, mut z: f32) -> f32 {
    let x0 = x as usize;
    let y0 = y as usize;
    let z0 = z as usize;

    x -= x0 as f32;
    y -= y0 as f32;
    z -= z0 as f32;
    // at this point (x, y, z) is bounded to [0, 1]
    debug_assert!((x >= 0.0) && (x <= 1.0) && (y >= 0.0) && (y <= 1.0) && (z >= 0.0) && (z <= 1.0));

    let gx = x0 % 256;
    let gy = y0 % 256;
    let gz = z0 % 256;

    // derive a permutation from the indices.
    // This behaves like a weak hash
    // note that the +1's must be consistent with the relative position in the box
    let a0 = gy + PERM[gx];
    let b0 = gy + PERM[gx + 1];
    let a000 = gz + PERM[a0];
    let a010 = gz + PERM[a0 + 1];
    let a100 = gz + PERM[b0];
    let a110 = gz + PERM[b0 + 1];

    // + 1 here means `gz+1` in a varibles => third index of the variable is increased as a result
    let h000 = PERM[a000];
    let h001 = PERM[a000 + 1];
    let h010 = PERM[a010];
    let h011 = PERM[a010 + 1];
    let h100 = PERM[a100];
    let h101 = PERM[a100 + 1];
    let h110 = PERM[a110];
    let h111 = PERM[a110 + 1];

    perlin_3d_grad(x, y, z, h000, h001, h010, h011, h100, h101, h110, h111)
}

#[cfg(test)]
mod tests {
    #[test]
    fn perlin() {
        let result = super::perlin_3d(1.5, 1.5, 1.5);
        assert_eq!(result, -0.875);
    }

    /// Given the mid point and all grads the same
    /// Then the noise is zero
    /// 
    /// This is because the mid point is always the average over every direction of every corner.
    /// When the direction of every corner is the same, the average is zero.
    #[test]
    fn grad() {
        let result = super::perlin_3d_grad(0.5, 0.5, 0.5, 0, 0, 0, 0, 0, 0, 0, 0);
        assert_eq!(result, 0.0);
        let result = super::perlin_3d_grad(0.5, 0.5, 0.5, 1, 1, 1, 1, 1, 1, 1, 1);
        assert_eq!(result, 0.0);
    }
}
