use super::*;

#[inline]
fn grad_2d(hash: usize, x: f32, y: f32) -> f32 {
    // in 2D, we only select from 4 different gradients.
    // http://riven8192.blogspot.com/2010/08/calculate-perlinnoise-twice-as-fast.html
    match hash % 4 {
        0 => x + y,  // (1, 1)
        1 => -x + y, // (-1, 1)
        2 => x - y,  // (1, -1)
        3 => -x - y, // (-1, -1)
        _ => 0.0,    // unreachable
    }
}

#[inline]
fn perlin_2d_grad(x: f32, y: f32, g00: usize, g10: usize, g01: usize, g11: usize) -> f32 {
    // compute the gradients
    // note: each corner has its own independent direction (derived from the permutation table)
    // g00 represents the dot product of (x,y) with one of the directions assigned to the corner `(0,0)` (e.g. `(1,1)`)
    let g00 = grad_2d(g00, x, y); // (x,y) - (0,0)
    let g10 = grad_2d(g10, x - 1.0, y); // (x,y) - (1,0)

    let g01 = grad_2d(g01, x, y - 1.0); // (x,y) - (0,1)
    let g11 = grad_2d(g11, x - 1.0, y - 1.0); // (x,y) - (1,1)

    // smoothed x (continuous second derivative)
    let u = fade(x);
    // smoothed y (continuous second derivative)
    let v = fade(y);

    // g00 + f(x) * (g10 - g00) + f(y) * (g01 + f(x) * (g11 - g01) - (g00 + f(x) * (g10 - g00)))
    lerp(v, lerp(u, g00, g10), lerp(u, g01, g11))
    // in particular
    // x = 0 and y = 0 => g00 => 0
    // x = 1 and y = 0 => g10 => 0
    // x = 0 and y = 1 => g01 => 0
    // x = 1 and y = 1 => g11 => 0
    // i.e. noise at each corner equals to zero
    // x = 0.5 and y = 0.5 => (g00+g10+g01+g11)/4
    // i.e. noise at the center equals to the average of the gradients
}

/// Returns the evaluation of perlin noise at position (x, y)
/// This function does not allocate
/// It uses the improved implementation of perlin noise
/// whose reference implementation is available here: https://mrl.cs.nyu.edu/~perlin/noise/
/// The modifications are:
/// * made it 2d, ignoring the z coordinate
/// * the grad computation was modified
pub fn perlin_2d(mut x: f32, mut y: f32) -> f32 {
    let x0 = x as usize;
    let y0 = y as usize;

    x -= x0 as f32;
    y -= y0 as f32;
    // at this point (x, y) is bounded to [0, 1]
    debug_assert!((x >= 0.0) && (x <= 1.0) && (y >= 0.0) && (y <= 1.0));

    let gx = x0 % 256;
    let gy = y0 % 256;

    // derive a permutation from the indices.
    // This behaves like a weak hash
    // note that the +1's must be consistent with the relative position in the box
    let a00 = gy + PERM[gx];
    let a10 = gy + PERM[gx + 1];

    let g00 = PERM[a00];
    let g10 = PERM[a10];
    let g01 = PERM[1 + a00];
    let g11 = PERM[1 + a10];

    perlin_2d_grad(x, y, g00, g10, g01, g11)
}

#[cfg(test)]
mod tests {
    #[test]
    fn perlin() {
        let result = super::perlin_2d(1.5, 1.5);
        assert_eq!(result, 0.0);
    }

    /// the middle point with all gradients point to (1,1) result in:
    ///     (g00+g10+g01+g11)/4
    ///     = ((x + y) + (x-1 + y) + (x + y-1) + (x-1 + y-1))/4
    ///     = (4x + 4y-4)/4
    ///     = x + y - 1 = 0.5 + 0.5 - 1 = 0
    #[test]
    fn grad() {
        let result = super::perlin_2d_grad(0.5, 0.5, 0, 0, 0, 0);
        assert_eq!(result, 0.0);
    }
}
