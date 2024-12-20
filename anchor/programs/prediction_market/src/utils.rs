use anchor_lang::prelude::*;


use crate::error::CustomError;
use crate::constants::TOKEN_DECIMALS;
use crate::constants::SHARES_DECIMALS;

// Calculates the LMSR cost function
/// `q` is a vector of shares for each outcome.
/// `b` is the liquidity parameter.
pub fn calculate_cost(q: &Vec<u64>, b: u64) -> Result<u64> {
    use std::f64;

    // Convert b to f64 for calculations
    let b_f64 = b as f64;

    // Calculate exponentials of (q[i] / b)
    let exponentials: Vec<f64> = q
        .iter()
        .map(|&qi| (qi as f64 / b_f64).exp()) // e^(qi / b)
        .collect();

    // Sum all exponentials
    let sum_exp: f64 = exponentials.iter().sum();

    // Ensure sum_exp is valid
    if sum_exp <= 0.0 {
        return Err(error!(CustomError::MathError));
    }

    // Take the natural logarithm and scale back
    let cost: f64 = b_f64 * sum_exp.ln();

    let scale_factor = 10u64.pow((TOKEN_DECIMALS - SHARES_DECIMALS) as u32) as f64;
    let scaled_cost = cost * scale_factor;

    // Convert back to u64
    Ok(scaled_cost.round() as u64) // Round to nearest integer
}


/// Calculates the cost based on LMSR formula
// pub fn calculate_cost(q: &Vec<u64>, b: u64) -> Result<u64> {
//     // Use a fixed-point multiplier to simulate exponentiation.
//     let mut sum = 0u64;

//     for &qi in q.iter() {
//         // Calculate e^(q_i / b) using fixed-point arithmetic.
//         let exponent = (qi as f64) / (b as f64); // q_i / b
//         let cost_for_outcome = (exponent.exp() * SCALE as f64).round() as u64; // e^(q_i / b) * SCALE
//         sum = sum.checked_add(cost_for_outcome).ok_or(CustomError::Overflow)?;
//     }

//     // Multiply by b (after scaling).
//     let total_cost = sum.checked_mul(b).ok_or(CustomError::Overflow)?;

//     // Return the total cost, considering the scale.
//     Ok(total_cost / SCALE)
// }


/// Calculates the fee based on cost and fee percent
pub fn calculate_fee(cost: u64, fee_percent: u64) -> Result<u64> {
    // fee_percent is expected to be in basis points (e.g., 500 for 5%)
    let fee = (cost.checked_mul(fee_percent).ok_or(CustomError::Overflow)?)
        .checked_div(10000)
        .ok_or(CustomError::Overflow)?;
    Ok(fee)
}
// pub fn calculate_required_initial_funds(b: u64, num_outcomes: usize) -> Result<u64, &'static str> {
//     if num_outcomes == 0 {
//         return 0; //msg!("Number of outcomes must be greater than 0");
//     }

//     let b_float = b as f64;
//     let n_float = num_outcomes as f64;

//     let required_funds = b_float * n_float.ln();

//     // Safely cast the result back to `u64` if valid
//     if required_funds.is_finite() && required_funds > 0.0 {
//         Ok(required_funds.ceil() as u64) // Round up to ensure adequacy
//     } else {
//         CustomError//Err("Invalid calculation for initial funds")
//     }
// }
