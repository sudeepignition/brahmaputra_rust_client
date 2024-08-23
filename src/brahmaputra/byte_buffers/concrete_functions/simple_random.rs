pub fn simple_random(seed: u32, pool_size: u32) -> u32 {

    // Ensure the pool size is greater than 0 to avoid division by zero
    if pool_size == 0 {
        panic!("Pool size must be greater than 0");
    }

    // Constants for a basic Linear Congruential Generator (LCG)
    const A: u32 = 1664525;
    const C: u32 = 1013904223;
    const M: u32 = 2u32.pow(31);

    // Update the seed using the LCG formula
    let new_seed = (A.wrapping_mul(seed).wrapping_add(C)) % M;

    // Return a number between 0 and pool_size - 1
    new_seed % pool_size
}