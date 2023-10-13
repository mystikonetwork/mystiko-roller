use crate::common::{RollerError, RollerResult};
use crate::roller::types::MAX_ROLLUP_SIZE;
use ethers_core::types::U256;
use mystiko_protos::data::v1::Commitment;
use mystiko_types::CircuitType;
use mystiko_utils::convert::bytes_to_u256;
pub fn circuit_type_from_rollup_size(rollup_size: usize) -> RollerResult<CircuitType> {
    match rollup_size {
        1 => Ok(CircuitType::Rollup1),
        2 => Ok(CircuitType::Rollup2),
        4 => Ok(CircuitType::Rollup4),
        8 => Ok(CircuitType::Rollup8),
        16 => Ok(CircuitType::Rollup16),
        x => Err(RollerError::RollupSizeError(x)),
    }
}

fn calc_rollup_size(included: usize, queued: usize) -> usize {
    match () {
        _ if queued >= 16 && included % 16 == 0 => 16,
        _ if queued >= 8 && included % 8 == 0 => 8,
        _ if queued >= 4 && included % 4 == 0 => 4,
        _ if queued >= 2 && included % 2 == 0 => 2,
        _ => 1,
    }
}

pub fn calc_rollup_size_queue(included: usize, queued: usize) -> RollerResult<(usize, Vec<usize>)> {
    if queued == 0 {
        return Err(RollerError::RollupSizeError(0));
    }

    let mut total_rollup_size = 0;
    let mut rollup_array = Vec::new();

    let mut included_count = included;
    let mut queued_count = queued;
    let mut rollup_size = 0;

    loop {
        let new_rollup_size = calc_rollup_size(included_count, queued_count);
        if new_rollup_size < rollup_size || (new_rollup_size == rollup_size && new_rollup_size < MAX_ROLLUP_SIZE) {
            break;
        }

        rollup_size = new_rollup_size;
        rollup_array.push(rollup_size);
        total_rollup_size += rollup_size;

        if queued_count < rollup_size {
            break;
        }

        queued_count -= rollup_size;
        included_count += rollup_size;
    }

    Ok((total_rollup_size, rollup_array))
}
pub fn calc_total_rollup_fee(cms: &[Commitment], total_plan: usize) -> RollerResult<U256> {
    cms.iter().take(total_plan).try_fold(U256::zero(), |acc, cm| {
        cm.rollup_fee
            .as_ref()
            .ok_or(RollerError::RollerInternalError(
                "handler commitment rollup fee is none".to_string(),
            ))
            .map(|fee| acc + bytes_to_u256(fee))
    })
}
