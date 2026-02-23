pub enum RolloutStage { Stage0, Stage1, Stage2, Stage3, Stage4 }

pub struct RolloutProfile {
    pub stage: RolloutStage,
    pub allowed_modalities: Vec<(Invasiveness, Purpose)>, // e.g., (NonInvasive, Therapeutic)
}
