pub struct TriggerRule {
    pub signal: SignalKind,
    pub threshold: f64,
    pub min_steps_above: u32,
    pub suggested_interventions: Vec<&'static str>, // e.g. ["community_veto", "symmetry"]
}

pub struct TriggerOutcome {
    pub fired: bool,
    pub at_step: Option<u64>,
    pub applied_policies: Vec<&'static str>,
}
