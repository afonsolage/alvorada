//! 2D Simplex Noise for procedural terrain generation.
//!
//! Based on Stefan Gustavson's "Simplex noise demystified" (2005).
//! Provides [`SimplexNoise`] for generating smooth, continuous noise
//! suitable for terrain height maps.

/// Gradient vectors for 2D simplex noise (8 directions).
const GRAD2: [(f64, f64); 8] = [
    (1.0, 0.0),
    (-1.0, 0.0),
    (0.0, 1.0),
    (0.0, -1.0),
    (0.707_106_77, 0.707_106_77),
    (-0.707_106_77, 0.707_106_77),
    (0.707_106_77, -0.707_106_77),
    (-0.707_106_77, -0.707_106_77),
];

/// A seeded 2D Simplex Noise generator.
///
/// Use [`SimplexNoise::new`] to create an instance, then call
/// [`SimplexNoise::get`] for single-octave noise or [`SimplexNoise::fbm`]
/// for multi-octave fractional Brownian motion noise.
///
/// # Example
/// ```
/// let noise = SimplexNoise::new(42);
/// let value = noise.fbm(0.5, 0.5, 4, 0.5, 2.0); // roughly in [-1, 1]
/// ```
pub struct SimplexNoise {
    /// Doubled permutation table to avoid bounds-wrapping in the hot path.
    perm: [u8; 512],
}

impl SimplexNoise {
    /// Skew factor: maps `(x, y)` to simplex grid coordinates.
    /// Equal to `(sqrt(3) − 1) / 2`.
    const F2: f64 = 0.366_025_403_784;

    /// Unskew factor: maps simplex grid back to `(x, y)`.
    /// Equal to `(3 − sqrt(3)) / 6`.
    const G2: f64 = 0.211_324_865_405;

    /// Creates a new `SimplexNoise` with the given `seed`.
    ///
    /// Different seeds produce different but fully reproducible noise fields.
    pub fn new(seed: u64) -> Self {
        // Build an identity permutation [0, 1, …, 255].
        let mut p = [0u8; 256];
        for i in 0..256usize {
            p[i] = i as u8;
        }

        // Fisher-Yates shuffle driven by a simple LCG keyed on `seed`.
        let mut state = seed;
        for i in (1..256usize).rev() {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let j = ((state >> 33) as usize) % (i + 1);
            p.swap(i, j);
        }

        // Double the table so that `perm[n]` for `n ∈ [0, 511]` is always valid.
        let mut perm = [0u8; 512];
        for i in 0..512 {
            perm[i] = p[i & 255];
        }

        Self { perm }
    }

    /// Returns the simplex noise value at `(x, y)`.
    ///
    /// Output is approximately in `[−1, 1]`.
    pub fn get(&self, x: f64, y: f64) -> f64 {
        // Skew input space to the simplex grid.
        let s = (x + y) * Self::F2;
        let i = (x + s).floor() as i32;
        let j = (y + s).floor() as i32;

        // Unskew to find distances from the origin corner.
        let t = (i + j) as f64 * Self::G2;
        let x0 = x - (i as f64 - t);
        let y0 = y - (j as f64 - t);

        // Determine which simplex triangle (upper or lower) we are in.
        let (i1, j1): (i32, i32) = if x0 > y0 { (1, 0) } else { (0, 1) };

        // Offsets to the other two corners in unskewed coordinates.
        let x1 = x0 - i1 as f64 + Self::G2;
        let y1 = y0 - j1 as f64 + Self::G2;
        let x2 = x0 - 1.0 + 2.0 * Self::G2;
        let y2 = y0 - 1.0 + 2.0 * Self::G2;

        // Wrap cell coordinates into the permutation range [0, 255].
        // `i & 255` works correctly for negative i32 values (two's complement).
        let ii = (i & 255) as usize;
        let jj = (j & 255) as usize;

        // Gradient indices for each corner.
        // Index safety proof (perm has 512 elements, ii/jj ∈ [0,255], i1/j1 ∈ {0,1}):
        //   gi0: perm[jj]            ∈ [0,255]; ii + perm[jj]         ≤ 255+255 = 510 ✓
        //   gi1: perm[jj+j1], jj+j1 ≤ 256, valid; (ii+i1)+perm[jj+j1] ≤ 256+255 = 511 ✓
        //   gi2: perm[jj+1],  jj+1  ≤ 256, valid; (ii+1) +perm[jj+1]  ≤ 256+255 = 511 ✓
        let gi0 = (self.perm[ii + self.perm[jj] as usize] & 7) as usize;
        let gi1 =
            (self.perm[ii + i1 as usize + self.perm[jj + j1 as usize] as usize] & 7) as usize;
        let gi2 = (self.perm[ii + 1 + self.perm[jj + 1] as usize] & 7) as usize;

        // Sum noise contributions from each simplex corner.
        let n0 = Self::kernel(x0, y0, GRAD2[gi0]);
        let n1 = Self::kernel(x1, y1, GRAD2[gi1]);
        let n2 = Self::kernel(x2, y2, GRAD2[gi2]);

        // Scale to approximately [−1, 1].
        70.0 * (n0 + n1 + n2)
    }

    /// Computes the noise contribution from a single simplex corner.
    fn kernel(x: f64, y: f64, g: (f64, f64)) -> f64 {
        let t = 0.5 - x * x - y * y;
        if t < 0.0 {
            0.0
        } else {
            let t2 = t * t;
            t2 * t2 * (g.0 * x + g.1 * y)
        }
    }

    /// Fractional Brownian Motion — sums multiple noise octaves for richer terrain detail.
    ///
    /// # Parameters
    /// * `octaves`     – number of noise layers to sum (more = more detail, more cost).
    /// * `persistence` – amplitude multiplier per octave (typically `0.5`).
    /// * `lacunarity`  – frequency multiplier per octave (typically `2.0`).
    ///
    /// Returns a normalised value in approximately `[−1, 1]`.
    pub fn fbm(&self, x: f64, y: f64, octaves: u32, persistence: f64, lacunarity: f64) -> f64 {
        let mut value = 0.0f64;
        let mut amplitude = 1.0f64;
        let mut frequency = 1.0f64;
        let mut max_value = 0.0f64;

        for _ in 0..octaves {
            value += self.get(x * frequency, y * frequency) * amplitude;
            max_value += amplitude;
            amplitude *= persistence;
            frequency *= lacunarity;
        }

        value / max_value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noise_is_reproducible() {
        let n1 = SimplexNoise::new(42);
        let n2 = SimplexNoise::new(42);
        assert_eq!(n1.get(1.5, 2.7), n2.get(1.5, 2.7));
    }

    #[test]
    fn noise_differs_by_seed() {
        let n1 = SimplexNoise::new(0);
        let n2 = SimplexNoise::new(1);
        assert_ne!(n1.get(1.0, 1.0), n2.get(1.0, 1.0));
    }

    #[test]
    fn noise_range_is_roughly_unit() {
        let n = SimplexNoise::new(123);
        let samples: Vec<f64> = (0..100usize)
            .flat_map(|i| (0..100usize).map(move |j| (i, j)))
            .map(|(i, j)| n.get(i as f64 * 0.13, j as f64 * 0.13))
            .collect();
        let max = samples.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min = samples.iter().cloned().fold(f64::INFINITY, f64::min);
        assert!(min >= -1.1, "min {min} out of range");
        assert!(max <= 1.1, "max {max} out of range");
    }

    #[test]
    fn fbm_range_is_roughly_unit() {
        let n = SimplexNoise::new(999);
        let samples: Vec<f64> = (0..50usize)
            .flat_map(|i| (0..50usize).map(move |j| (i, j)))
            .map(|(i, j)| n.fbm(i as f64 * 0.1, j as f64 * 0.1, 4, 0.5, 2.0))
            .collect();
        let max = samples.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min = samples.iter().cloned().fold(f64::INFINITY, f64::min);
        assert!(min >= -1.1, "fbm min {min} out of range");
        assert!(max <= 1.1, "fbm max {max} out of range");
    }
}
