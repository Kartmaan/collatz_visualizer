// File collatz.rs
// This module contains the logic related to the Collatz conjecture.
// It defines how to generate a Collatz sequence and how to calculate statistics on this sequence.

/// Computes the Collatz sequence for a positive integer `start`.
///
/// The Syracuse conjecture defines a sequence as follows:
/// - We start with an integer `n > 0`.
/// - If it is even, the next term is `n / 2`.
/// - If it is odd, the next term is `3 * n + 1`.
/// - The sequence continues until it reaches the value 1.
///
/// # Arguments
/// * `start` - The positive integer (`u64`) from which the sequence begins. `u64` stands for "unsigned 64-bit integer",
/// an unsigned integer (positive or zero) stored in 64 bits.
///
/// # Returns
/// * `Vec<u64>` - A vector (a dynamic list) containing all the numbers in the sequence,
/// starting with `start` and ending with 1.
///
/// # Note
/// The conjecture states that any Collatz sequence reaches 1 for any positive starting integer.
/// This function includes a check to avoid a potential overflow
/// when calculating `3 * n + 1` for very large `u64` numbers.
pub fn generate_sequence(start: u64) -> Vec<u64> {
    // Special case for 0. Although the conjecture concerns integers > 0,
    // we handle this case to avoid an infinite loop (0 -> 0).
    if start == 0 {
        return vec![0]; // Returns a vector containing only 0.
    }

    let mut sequence = Vec::new(); // Create an empty vector to store the sequence.
    
    // `current` will store the current value in the sequence. We start with the starting value.
    let mut current = start;
    
    sequence.push(current); // Adds the starting value to the sequence.
    
    // Loop until the current value is 1 (the sequence's stop condition).
    while current != 1 {
        // Check if the current number is even.
        if current % 2 == 0 {
            // If even, divide by 2 to get the next number.
            current = current / 2;
        } else {
            // If odd, multiply by 3 and add 1.
            // We need to check for potential integer overflow before performing the calculation (3 * n + 1).
            // `u64::MAX` is the maximum value a u64 can hold.
            // If `current` is greater than `(u64::MAX - 1) / 3`, then `3 * current + 1` would overflow.
            if current > (u64::MAX - 1) / 3 {
                // If there is a risk of overshoot, the sequence is stopped
                sequence.push(current);
                break;
            }
            // Perform the calculation for odd numbers.
            current = 3 * current + 1;
        }
        // Add the newly calculated number to the sequence vector.
        sequence.push(current);
    }
    // Return the complete sequence.
    sequence
}

/// Holds statistics calculated from a Collatz sequence.
pub struct CollatzStats {
    pub length: usize,           // Sequence length (total flight time)
    pub max_value: u64,          // Maximum value reached (altitude)
    pub max_value_index: usize,  // Position of the maximum value
    pub even_count: usize,       // Number of even values
    pub odd_count: usize,        // Number of odd values
    pub stopping_time: usize,    // Stop time (number of steps to reach a value < start)
}

/// Calculates various statistics for a given Collatz sequence.
///
/// # Arguments
///
/// * `sequence` - A slice (`&[u64]`) representing a previously generated Collatz sequence.
///
/// # Returns
///
/// * `CollatzStats` - A struct containing the calculated statistics.
///                    Returns default/zero stats if the input sequence is empty.
pub fn calculate_stats(sequence: &[u64]) -> CollatzStats {
    // If the sequence is empty, return default statistics.
    if sequence.is_empty() {
        return CollatzStats {
            length: 0,
            max_value: 0,
            max_value_index: 0,
            even_count: 0,
            odd_count: 0,
            stopping_time: 0,
        };
    }

    // Get the starting value (the first element) of the sequence. Needed for stopping time calculation.
    let start_value = sequence[0];
    // Get the total length (number of steps + 1) of the sequence.
    let length = sequence.len();
    
    // Find the maximum value in the sequence and its index.
    // `enumerate()` pairs each element with its index (0, val0), (1, val1), ...
    // `max_by_key()` finds the element (in this case, a tuple `(index, &value)`)
    // that yields the maximum value based on the provided key function (`|&(_, &value)| value`).
    // `unwrap_or` is used here as a safeguard, although an empty sequence is handled above.
    // It returns a tuple `(index, &value)`. We destructure it to get the index and the value itself.
    let (max_value_index, max_value) = sequence.iter()
        .enumerate()
        .max_by_key(|&(_, &value)| value)
        .unwrap_or((0, &0)); // Default to index 0, value 0 if something unexpected happens

    
    // Count the number of even numbers in the sequence.
    // `filter()` iterates through the sequence and keeps only the elements satisfying the condition (`n % 2 == 0`).
    // `count()` returns the number of elements remaining after filtering.
    let even_count = sequence.iter().filter(|&&n| n % 2 == 0).count();
    
    // The count of odd numbers is simply the total length minus the count of even numbers.
    let odd_count = length - even_count;
    
    // Calculate the stopping time: the number of steps until the sequence value
    // drops strictly below the starting value for the first time.
    // sequence.iter().enumerate() Gets pairs of (index, &value).
    // `skip(1)` skips the first element (the starting value itself).
    // `find()` searches for the first element `(index, &value)` that satisfies the condition `value < start_value`.
    // `map(|(index, _)| index)` extracts the index if an element is found.
    // `unwrap_or(length - 1)` provides a default value if no element smaller than `start_value` is found
    // (e.g., for sequence [1] or [2, 1]). In this case, we consider the stopping time to be the index of the last element (1).
    let stopping_time = sequence.iter()
        .enumerate()
        .skip(1)
        .find(|&(_, &value)| value < start_value)
        .map(|(index, _)| index)
        .unwrap_or(length - 1); 
    
    // Return the populated statistics struct.
    CollatzStats {
        length,
        max_value: *max_value,
        max_value_index,
        even_count,
        odd_count,
        stopping_time,
    }
}

// Test module: Contains unit tests for the functions in this file.
// This code only runs when you execute `cargo test`.
#[cfg(test)]
mod tests {
    // Import everything from the parent module (the code above).
    use super::*;

    // Test function for `generate_sequence`.
    #[test]
    fn test_generate_sequence() {
        // Test case 1: Starting with n = 6.
        let sequence = generate_sequence(6);
        // Assert that the generated sequence matches the expected sequence.
        assert_eq!(sequence, vec![6, 3, 10, 5, 16, 8, 4, 2, 1]);
        
        // Test case 2: Starting with n = 1 (base case).
        let sequence = generate_sequence(1);
        assert_eq!(sequence, vec![1]);
    }

    // Test function for `calculate_stats`.
    #[test]
    fn test_calculate_stats() {
        // Test case 1: Using the sequence for n = 6.
        let sequence = generate_sequence(6); // vec![6, 3, 10, 5, 16, 8, 4, 2, 1]
        let stats = calculate_stats(&sequence);
        
        // Assert that each statistic matches the expected value.
        assert_eq!(stats.length, 9);
        assert_eq!(stats.max_value, 16);
        assert_eq!(stats.max_value_index, 4);
        assert_eq!(stats.even_count, 6);
        assert_eq!(stats.odd_count, 3);
        assert_eq!(stats.stopping_time, 1);
    }
}
