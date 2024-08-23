use std::hash::{DefaultHasher, Hash, Hasher};

pub fn select_partition(input: String, total_partition_size: u64) -> u32 {
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let hash_value = hasher.finish();

    // Get a number between 1 and 3
    let random_number = (hash_value % total_partition_size) + 1;
    random_number as u32
}