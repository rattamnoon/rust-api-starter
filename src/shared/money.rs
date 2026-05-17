use crate::errors::app_error::AppError;

pub fn validate_amount(amount: i64) -> Result<(), AppError> {
    if amount < 0 {
        Err(AppError::BadRequest("amount must be zero or greater".into()))
    } else {
        Ok(())
    }
}

pub fn multiply_amount(unit_price_amount: i64, quantity: i32) -> Result<i64, AppError> {
    validate_amount(unit_price_amount)?;
    if quantity <= 0 {
        return Err(AppError::BadRequest(
            "quantity must be at least 1".into(),
        ));
    }

    unit_price_amount
        .checked_mul(i64::from(quantity))
        .ok_or_else(|| AppError::BadRequest("amount overflow".into()))
}
