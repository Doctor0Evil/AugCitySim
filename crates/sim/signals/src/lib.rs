pub enum SignalKind {
    AugmentationCaste,
    DiscriminationBias,
    RumorMistrust,
    BacklashUnrest,
    SoftCoercion,
    LossOfAgency,
}

pub struct SignalSample {
    pub t_step: u64,
    pub corridor_id: String,      // EcoCorridorContext id
    pub signal: SignalKind,
    pub value: f64,               // normalized 0..1
    pub exceeds_threshold: bool,  // true if crosses configured limit
}
