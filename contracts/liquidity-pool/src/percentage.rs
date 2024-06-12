use soroban_sdk::{Address, Map};

pub(crate) const ONE_XLM_IN_STROOPS: i64 = 10000000;

fn calculate_percentage(amount: &i128, total_balance: &i128) -> f64 {
    (*amount as f64 * 100.0) / *total_balance as f64
}

fn recalculate_percentage(lender_percentage: f64, old_balance: i128, total_balance: &i128) -> f64 {
    lender_percentage / (*total_balance as f64 / old_balance as f64)
}

pub fn percentage_to_integer(percentage: f64) -> i64 {
    (percentage * ONE_XLM_IN_STROOPS as f64) as i64
}

fn integer_to_percentage(stroop: i64) -> f64 {
    stroop as f64 / ONE_XLM_IN_STROOPS as f64
}

pub fn process_lender_contribution(
    length: u32,
    lender: &Address,
    mut lender_contribution: Map<Address, i64>,
    lender_balance: &i128,
    total_balance: &i128,
    old_balance: &i128,
) -> Map<Address, i64> {
    if lender_contribution.len() == length {
        let percentage = calculate_percentage(&lender_balance, &total_balance);
        lender_contribution.set(lender.clone(), percentage_to_integer(percentage))
    } else {
        for (address, percentage) in lender_contribution.iter() {
            let new_percentage = recalculate_percentage(
                integer_to_percentage(percentage),
                *old_balance,
                &total_balance,
            );
            lender_contribution.set(address.clone(), percentage_to_integer(new_percentage));
        }
        let percentage = calculate_percentage(&lender_balance, &total_balance);
        lender_contribution.set(lender.clone(), percentage_to_integer(percentage))
    }
    lender_contribution
}
