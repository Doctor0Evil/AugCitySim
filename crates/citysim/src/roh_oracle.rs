use crate::types::AugmentationModality;

/// Thin interface to your existing NEVL / RoH guard crate.
/// This crate must be host-local, enclave-only, and non-actuating.
pub trait RohOracle {
    /// Returns (admissible, roh_after_hint).
    fn check_modality(&self, host_id: &str, modality: &AugmentationModality) -> (bool, f32);
}

// Example adapter: in tests, we just enforce roh_risk_hint <= ceiling.
// In real deployments, this calls nevl_guard::evaluate(...) inside a TEE.
pub struct BoundedHintOracle {
    pub roh_ceiling: f32,
}

impl RohOracle for BoundedHintOracle {
    fn check_modality(&self, _host_id: &str, modality: &AugmentationModality) -> (bool, f32) {
        let admissible = modality.roh_risk_hint <= self.roh_ceiling;
        (admissible, modality.roh_risk_hint)
    }
}
