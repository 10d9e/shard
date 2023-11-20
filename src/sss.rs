extern crate rand;

use core::fmt;
use gf256::gf256;
use rand::Rng;
use serde::{
    de::{SeqAccess, Visitor},
    ser::SerializeSeq,
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::collections::HashMap;

/// Represents a polynomial over the Galois field GF(2^8).
///
/// Each polynomial is represented by its coefficients, stored in a vector.
/// Coefficients are elements of the GF(2^8) field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Polynomial {
    /// The coefficients of the polynomial, where each coefficient is an element of GF(2^8).
    pub coefficients: Vec<gf256>,
}

impl Polynomial {
    /// Constructs a new polynomial of a given degree with random coefficients,
    /// where the constant term is the provided secret.
    ///
    /// # Arguments
    ///
    /// * `degree` - The degree of the polynomial.
    /// * `secret` - The secret (constant term) of the polynomial.
    pub fn new(degree: usize, secret: gf256) -> Self {
        let mut rng = rand::thread_rng();
        let mut coefficients = vec![secret; degree + 1];

        for coeff in coefficients.iter_mut().skip(1) {
            *coeff = gf256::new(rng.gen());
        }

        Polynomial { coefficients }
    }

    /// Evaluates the polynomial at a given point.
    ///
    /// # Arguments
    ///
    /// * `x` - The point at which to evaluate the polynomial.
    ///
    /// # Returns
    ///
    /// The value of the polynomial at point `x`.
    pub fn evaluate(&self, x: gf256) -> gf256 {
        let mut result = gf256::new(0);
        let mut term = gf256::new(1);

        for &coeff in &self.coefficients {
            result += coeff * term;
            term *= x;
        }

        result
    }
}

/// Implements serialization for `Polynomial` as a sequence of bytes,
/// allowing Polynomials to be serialized and sent over the network.
impl Serialize for Polynomial {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize as a sequence of bytes
        let mut seq = serializer.serialize_seq(Some(self.coefficients.len()))?;
        for &gf in &self.coefficients {
            seq.serialize_element(&u8::from(gf))?;
        }
        seq.end()
    }
}

/// Implements deserialization for `Polynomial` from a sequence of bytes,
/// allowing Polynomials to be reconstructed from serialized data.
impl<'de> Deserialize<'de> for Polynomial {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        /// A visitor to handle the deserialization of `Polynomial`.
        struct PolynomialVisitor;

        impl<'de> Visitor<'de> for PolynomialVisitor {
            type Value = Polynomial;

            /// Describes what this visitor expects to find.
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence of bytes representing polynomial coefficients")
            }

            /// Visits a sequence of bytes to construct a `Polynomial`.
            fn visit_seq<V>(self, mut seq: V) -> Result<Polynomial, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mut coefficients = Vec::new();
                while let Some(byte) = seq.next_element()? {
                    coefficients.push(gf256::new(byte));
                }
                Ok(Polynomial { coefficients })
            }
        }

        // Deserialize the sequence using the `PolynomialVisitor`.
        deserializer.deserialize_seq(PolynomialVisitor)
    }
}

/// Splits a secret into a specified number of shares using Shamir's Secret Sharing Scheme.
///
/// # Arguments
/// * `secret` - A byte slice representing the secret to be split.
/// * `threshold` - The minimum number of shares required to reconstruct the secret.
/// * `shares` - The total number of shares to be created.
///
/// # Returns
/// A `Result` containing either a `HashMap` of shares (if successful) or an error message.
///
/// # Errors
/// Returns an error if the threshold is invalid (<= 1) or if the number of shares is less than the threshold.
///
/// # Examples
/// ```rust
/// use shard::sss::split_secret;
///
/// let secret = b"hello world";
/// let threshold = 3;
/// let total_shares = 5;
/// let shares = split_secret(secret, threshold, total_shares);
/// // shares now contains 5 different parts of the secret.
/// ```
pub fn split_secret(
    secret: &[u8],
    threshold: usize,
    shares: usize,
) -> Result<HashMap<u8, Vec<u8>>, String> {
    if threshold <= 1 {
        return Err("Invalid threshold".to_string());
    }

    if shares < threshold {
        return Err("Invalid count".to_string());
    }

    let mut shares_map: HashMap<u8, Vec<u8>> = HashMap::new();

    for &byte in secret {
        let poly = Polynomial::new(threshold - 1, gf256::new(byte));

        for i in 1..=shares as u8 {
            let y = poly.evaluate(gf256::new(i));
            shares_map.entry(i).or_insert_with(Vec::new).push(y.into());
        }
    }

    Ok(shares_map)
}

/// Combines shares to reconstruct a secret using Shamir's Secret Sharing Scheme.
///
/// # Arguments
/// * `shares_map` - A `HashMap` where each key-value pair represents a share of the secret.
///
/// # Returns
/// An `Option` containing the reconstructed secret as a `Vec<u8>` if successful, or `None` if not.
///
/// # Examples
/// ```ignore
/// use shard::sss::{split_secret, combine_shares};
/// // Assuming `shares_map` is a HashMap<u8, Vec<u8>> obtained from `split_secret`
/// let reconstructed_secret = combine_shares(&shares_map).unwrap();
/// ```
pub fn combine_shares(shares_map: &HashMap<u8, Vec<u8>>) -> Option<Vec<u8>> {
    let mut secret_length = 0;
    for v in shares_map.values() {
        secret_length = v.len();
        break;
    }

    let mut secret = vec![0; secret_length];
    let mut points = Vec::new();

    for i in 0..secret_length {
        points.clear();
        for (&k, v) in shares_map {
            if let Some(&y) = v.get(i) {
                points.push((gf256::new(k), gf256::new(y)));
            }
        }
        secret[i] = interpolate(&points, gf256::new(0)).into();
    }

    Some(secret)
}

/// Performs Lagrange interpolation on a set of points to find the value of the polynomial at a specific point.
///
/// This function is a key part of Shamir's Secret Sharing Scheme, enabling the reconstruction of secrets.
///
/// # Arguments
///
/// * `points` - A slice of tuples, each representing a point `(x, y)` on the polynomial.
/// * `x` - The `x` value at which to evaluate the polynomial.
///
/// # Returns
///
/// The interpolated value at `x`.
///
/// # Examples
///
/// Interpolating a value from points of a polynomial:
///
/// ```ignore
/// let points = [(gf256::new(1), gf256::new(5)), (gf256::new(2), gf256::new(10))];
/// let interpolated_value = interpolate(&points, gf256::new(3));
/// // `interpolated_value` is the value of the polynomial at x = 3.
/// ```
fn interpolate(points: &[(gf256, gf256)], x: gf256) -> gf256 {
    let mut value = gf256::new(0);

    for (i, &(a_x, a_y)) in points.iter().enumerate() {
        let mut weight = gf256::new(1);

        for (j, &(b_x, _)) in points.iter().enumerate() {
            if i != j {
                let top = x + b_x; // XOR in GF(2^8) is equivalent to addition
                let bottom = a_x + b_x; // XOR in GF(2^8) is equivalent to addition
                let factor = top / bottom; // Using gf256 division
                weight *= factor;
            }
        }

        value += weight * a_y; // Using gf256 multiplication and addition
    }

    value
}

/// https://en.wikipedia.org/wiki/Proactive_secret_sharing#Mathematics
/// Refreshes the shares of a secret in a proactive secret sharing scheme.
///
/// This method allows the shares to be updated without changing the underlying secret,
/// enhancing security against attacks that compromise share over time.
///
/// # Arguments
///
/// * `shares_map` - A mutable reference to a `HashMap` representing the shares.
/// * `threshold` - The minimum number of shares required to reconstruct the secret.
///
/// # Returns
///
/// `Result<(), String>` indicating successful completion or an error message.
///
/// # Errors
///
/// * Returns `Err` if `threshold` is less than or equal to 1.
/// * Returns `Err` if the `shares_map` is empty.
///
/// # Examples
///
/// Refreshing shares of a secret:
///
/// ```ignore
/// refresh_shares(&mut shares_map, 3).unwrap();
/// // The shares in `shares_map` are now refreshed.
/// ```
pub fn refresh_shares(
    shares_map: &mut HashMap<u8, Vec<u8>>,
    threshold: usize,
) -> Result<(), String> {
    if threshold <= 1 {
        return Err("Invalid threshold".to_string());
    }

    let secret_length = shares_map.values().next().ok_or("Empty shares map")?.len();

    for i in 0..secret_length {
        // Generate a new polynomial with a zero constant term (so it doesn't change the secret)
        let poly: Polynomial = Polynomial::new(threshold - 1, gf256::new(0));

        // Add the new polynomial's values to the existing shares
        for (&key, value) in shares_map.iter_mut() {
            if let Some(y) = value.get_mut(i) {
                let new_y = poly.evaluate(gf256::new(key));
                *y = *y ^ <gf256 as Into<u8>>::into(new_y); // XOR in GF(2^8) is equivalent to addition
            }
        }
    }
    Ok(())
}

/// Refreshes a single share in a proactive secret sharing scheme.
///
/// This function updates the value of a given share by adding new values generated from a set of polynomials.
/// This is useful in scenarios where the security of shares needs to be periodically enhanced without changing the secret.
///
/// # Arguments
///
/// * `share` - A tuple containing the share's identifier and a mutable reference to the share's value.
/// * `polynomials` - A slice of `Polynomial` objects used to generate new values for refreshing the share.
///
/// # Returns
///
/// A `Result<(), String>` indicating successful completion or an error message.
///
/// # Errors
///
/// * Returns `Err` if the share is empty.
/// * Returns `Err` if the length of the share does not match the number of polynomials.
///
/// # Examples
///
/// Refreshing an individual share:
///
/// ```ignore
/// let mut share = (1, vec![5u8, 10]);
/// let polynomials = generate_refresh_key(3, 2).unwrap();
/// refresh_share(&mut share, &polynomials).unwrap();
/// // The share is now updated with new values.
/// ```
pub fn refresh_share(share: (&u8, &mut Vec<u8>), polynomials: &[Polynomial]) -> Result<(), String> {
    if share.1.is_empty() {
        return Err("Empty share".to_string());
    }

    if share.1.len() != polynomials.len() {
        return Err("Share length and polynomials length mismatch".to_string());
    }

    for (i, y) in share.1.iter_mut().enumerate() {
        let poly = &polynomials[i];
        let new_y = poly.evaluate(gf256::new(*share.0)); // Assuming share keys start from 1
        *y = *y ^ <gf256 as Into<u8>>::into(new_y); // XOR in GF(2^8) is equivalent to addition
    }

    Ok(())
}

/// Generates a set of polynomials for refreshing shares in a proactive secret sharing scheme.
///
/// Each polynomial generated has a zero constant term, ensuring that when used for refreshing,
/// they do not alter the underlying secret. This function is critical in scenarios where
/// the security of the shares needs to be periodically enhanced.
///
/// # Arguments
///
/// * `threshold` - The minimum number of shares required to reconstruct the secret.
/// * `secret_length` - The length of the secret in bytes.
///
/// # Returns
///
/// A `Result` containing a vector of `Polynomial` objects if successful, or an error message.
///
/// # Errors
///
/// * Returns `Err` if the `threshold` is less than or equal to 1.
///
/// # Examples
///
/// Generating polynomials for refreshing shares:
///
/// ```ignore
/// let polynomials = generate_refresh_key(3, 5).unwrap();
/// // `polynomials` now contains 5 polynomials for refreshing shares.
/// ```
pub fn generate_refresh_key(
    threshold: usize,
    secret_length: usize,
) -> Result<Vec<Polynomial>, String> {
    if threshold <= 1 {
        return Err("Invalid threshold".to_string());
    }

    let mut polynomials = Vec::with_capacity(secret_length);
    for _ in 0..secret_length {
        // Generate a new polynomial with a zero constant term
        let poly = Polynomial::new(threshold - 1, gf256::new(0));
        polynomials.push(poly);
    }

    Ok(polynomials)
}

#[cfg(test)]
mod tests {
    use rand::seq::IteratorRandom;
    use std::borrow::BorrowMut;

    use super::*;

    #[test]
    fn test_split_and_combine_secret() {
        let secret = "test secret";
        let threshold = 3;
        let total_shares = 5;

        let shares_map = split_secret(secret.as_bytes(), threshold, total_shares).unwrap();
        let recovered = combine_shares(&shares_map).unwrap();

        assert_eq!(secret.as_bytes(), recovered.as_slice());
    }

    #[test]
    fn test_refresh_shares() {
        let secret = "refresh test";
        let threshold = 3;
        let total_shares = 5;

        let mut shares_map = split_secret(secret.as_bytes(), threshold, total_shares).unwrap();
        refresh_shares(&mut shares_map, threshold).unwrap();
        let recovered = combine_shares(&shares_map).unwrap();

        assert_eq!(secret.as_bytes(), recovered.as_slice());
    }

    #[test]
    fn test_refresh_share_end_to_end() -> Result<(), String> {
        let secret = "refresh share end to end";
        let threshold = 3;
        let shares = 5;

        assert!(gf256::new(0x53).to_be_bytes() == [0x53]);
        assert!(gf256::from_be_bytes([0x53]) == gf256::new(0x53));

        // Split the secret into shares
        let mut shares_map = split_secret(secret.as_bytes(), threshold, shares).unwrap();

        let secret_length = shares_map.values().next().ok_or("Empty shares map")?.len();
        let polynomials = generate_refresh_key(threshold, secret_length).unwrap();

        // Refresh each share
        for share in shares_map.borrow_mut() {
            refresh_share(share, &polynomials)?;
        }

        // Combine the shares to recover the secret
        let recovered_secret = combine_shares(&shares_map).unwrap();

        // Check that the recovered secret is the same as the original secret
        assert_eq!(secret.as_bytes(), recovered_secret);

        Ok(())
    }

    #[test]
    fn test_invalid_threshold_and_share_count() {
        let secret = "invalid params";
        assert!(split_secret(secret.as_bytes(), 0, 5).is_err());
        assert!(split_secret(secret.as_bytes(), 6, 5).is_err());
    }

    #[test]
    fn test_share_uniqueness() {
        let secret = "unique shares";
        let threshold = 3;
        let total_shares = 5;

        let shares_map = split_secret(secret.as_bytes(), threshold, total_shares).unwrap();
        let shares: Vec<_> = shares_map.values().collect();
        let all_unique = shares
            .iter()
            .all(|&v| shares.iter().filter(|&&x| x == v).count() == 1);

        assert!(all_unique);
    }

    #[test]
    fn test_share_subset_combination() {
        let secret = "subset test";
        let threshold = 3;
        let total_shares = 5;

        let shares_map = split_secret(secret.as_bytes(), threshold, total_shares).unwrap();
        let mut rng = rand::thread_rng();
        let subset: HashMap<u8, Vec<u8>> = shares_map
            .iter()
            .choose_multiple(&mut rng, threshold)
            .into_iter()
            .map(|(&key, value)| (key, value.clone()))
            .collect();

        let recovered = combine_shares(&subset).unwrap();

        assert_eq!(secret.as_bytes(), recovered.as_slice());
    }

    #[test]
    fn full_test() -> Result<(), String> {
        let secret = b"Remember what the dormouse said.";
        let threshold = 2;
        let total_shares = 5;

        // Split into 30 shares
        let mut shares_map = split_secret(secret, threshold, total_shares)?;
        assert!(shares_map.len() == total_shares);

        // Refresh the shares
        refresh_shares(&mut shares_map, threshold)?;
        refresh_shares(&mut shares_map, threshold)?;
        refresh_shares(&mut shares_map, threshold)?;

        // Select a random subset of shares to combine
        let mut rng = rand::thread_rng();
        let subset: HashMap<u8, Vec<u8>> = shares_map
            .iter()
            .choose_multiple(&mut rng, threshold)
            .into_iter()
            .map(|(&key, value)| (key, value.clone()))
            .collect();

        assert!(subset.len() == threshold);

        let recovered = combine_shares(&subset);
        assert!(recovered.is_some());
        assert!(recovered.unwrap().as_slice() == secret);

        Ok(())
    }

    #[test]
    fn test_should_fail_with_shares_below_threshold() -> Result<(), String> {
        // test should fail if the shares to reassemble are less than the threshold
        let secret = b"Remember what the dormouse said.";
        let threshold = 12;
        let total_shares = 30;

        // Split into 30 shares
        let mut shares_map = split_secret(secret, threshold, total_shares)?;
        assert!(shares_map.len() == total_shares);

        // Refresh the shares
        refresh_shares(&mut shares_map, threshold)?;
        refresh_shares(&mut shares_map, threshold)?;
        refresh_shares(&mut shares_map, threshold)?;

        // Select a random subset of shares to combine
        let mut rng = rand::thread_rng();
        let subset: HashMap<u8, Vec<u8>> = shares_map
            .iter()
            .choose_multiple(&mut rng, threshold-1)
            .into_iter()
            .map(|(&key, value)| (key, value.clone()))
            .collect();

        let recovered = combine_shares(&subset);
        assert!(recovered.is_some());

        println!("actual:    {}", hex::encode(&secret));
        println!("recovered: {}", hex::encode(recovered.clone().unwrap()));

        assert_ne!(recovered.unwrap().as_slice(), secret);

        Ok(())

    }
}
