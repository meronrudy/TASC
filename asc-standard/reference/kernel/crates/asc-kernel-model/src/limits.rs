use crate::generated_thresholds as t;

#[derive(Debug, Clone)]
pub struct KernelLimits {
    pub frame: String,
    pub max_speed_mps: f64,
    pub max_bank_deg: f64,
    pub attitude_limit_deg: f64,
    pub min_altitude_m: f64,
    pub position_min_m: [f64; 3],
    pub position_max_m: [f64; 3],
    pub min_soc_percent: f64,
    pub max_input_age_ms: u64,
    pub max_tick_interval_ms: u64,
    pub deadline_ms: u64,
    pub max_roll_rate_dps: f64,
    pub max_pitch_rate_dps: f64,
    pub max_yaw_rate_dps: f64,
    pub max_climb_rate_mps: f64,
}

impl Default for KernelLimits {
    fn default() -> Self {
        Self {
            frame: t::FRAME.to_string(),
            max_speed_mps: t::MAX_SPEED_MPS,
            max_bank_deg: t::MAX_BANK_DEG,
            attitude_limit_deg: t::ATTITUDE_LIMIT_DEG,
            min_altitude_m: t::MIN_ALTITUDE_M,
            position_min_m: [t::POSITION_MIN_X_M, t::POSITION_MIN_Y_M, t::POSITION_MIN_Z_M],
            position_max_m: [t::POSITION_MAX_X_M, t::POSITION_MAX_Y_M, t::POSITION_MAX_Z_M],
            min_soc_percent: t::MIN_SOC_PERCENT,
            max_input_age_ms: t::MAX_INPUT_AGE_MS,
            max_tick_interval_ms: t::MAX_TICK_INTERVAL_MS,
            deadline_ms: t::DEADLINE_MS,
            max_roll_rate_dps: t::MAX_ROLL_RATE_DPS,
            max_pitch_rate_dps: t::MAX_PITCH_RATE_DPS,
            max_yaw_rate_dps: t::MAX_YAW_RATE_DPS,
            max_climb_rate_mps: t::MAX_CLIMB_RATE_MPS,
        }
    }
}
