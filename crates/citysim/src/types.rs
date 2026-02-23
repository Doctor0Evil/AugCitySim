use serde::{Serialize, Deserialize};

/// Immutable ethical core flags baked into every run.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrincipleCore {
    pub no_ads: bool,
    pub no_financialization: bool,
    pub neurorights_inner_domain_excluded: bool,
    pub symmetric_civil_rights: bool,
    pub hard_bci_roh_ceiling: f32,   // must be <= 0.30
    pub errority_tightening_only: bool,
}

impl PrincipleCore {
    pub fn phoenix_west_defaults() -> Self {
        Self {
            no_ads: true,
            no_financialization: true,
            neurorights_inner_domain_excluded: true,
            symmetric_civil_rights: true,
            hard_bci_roh_ceiling: 0.30,
            errority_tightening_only: true,
        }
    }
}

/// Purpose axis: Therapy vs Enhancement.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PurposeKind {
    Therapeutic,
    Enhancing,
}

/// Invasiveness axis: Non-invasive vs Invasive.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InvasivenessKind {
    NonInvasive,
    Invasive,
}

/// Quadrant label for quick filtering and metrics.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AugmentationQuadrant {
    TherapeuticNonInvasive,
    TherapeuticInvasive,
    EnhancingNonInvasive,
    EnhancingInvasive,
}

impl AugmentationQuadrant {
    pub fn from_axes(p: &PurposeKind, i: &InvasivenessKind) -> Self {
        match (p, i) {
            (PurposeKind::Therapeutic, InvasivenessKind::NonInvasive) =>
                Self::TherapeuticNonInvasive,
            (PurposeKind::Therapeutic, InvasivenessKind::Invasive) =>
                Self::TherapeuticInvasive,
            (PurposeKind::Enhancing, InvasivenessKind::NonInvasive) =>
                Self::EnhancingNonInvasive,
            (PurposeKind::Enhancing, InvasivenessKind::Invasive) =>
                Self::EnhancingInvasive,
        }
    }
}

/// One augmentation modality available in the simulation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AugmentationModality {
    pub id: String,
    pub purpose: PurposeKind,
    pub invasiveness: InvasivenessKind,
    pub quadrant: AugmentationQuadrant,
    /// Scalar risk index already bounded by RoH 0.3 / NEVL in inner stack.
    pub roh_risk_hint: f32,
    /// Whether this modality is available in a given rollout stage.
    pub enabled_in_stage: Vec<RolloutStageKind>,
}

/// Simulation rollout stages (0–4).
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum RolloutStageKind {
    Stage0Workshops,
    Stage1EnvNonInvasive,
    Stage2NonInvasiveNeuro,
    Stage3LimitedInvasive,
    Stage4CityScale,
}

/// Agent’s augmentation status (non-clonable, no consciousness copying).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AugmentationStatus {
    pub has_any: bool,
    pub modalities: Vec<String>, // AugmentationModality.id
}

/// Social-cohesion & inequality metrics recorded per tick.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SocialMetrics {
    pub gini_income: f32,
    pub gini_augmented_vs_non: f32,
    pub segregation_index: f32,
    pub stigma_index: f32,
    pub perceived_coercion_index: f32,
    pub civil_unrest_probability: f32,
}

/// Core agent (citizen) in the simulation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub neighborhood_id: String,
    pub income_band: u8,
    pub is_indigenous: bool,
    pub is_disabled: bool,
    pub augmentation: AugmentationStatus,
    pub trust_in_institutions: f32,    // 0–1
    pub fear_of_augmentation: f32,     // 0–1
    pub perceived_coercion: f32,       // 0–1
    pub perceived_benefit: f32,        // 0–1
}

/// Neighborhood-level social context.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeighborhoodContext {
    pub id: String,
    pub is_tribal_land: bool,
    pub has_fpic_required: bool,
    pub base_gini_income: f32,
    pub base_stigma_index: f32,
    pub base_segregation_index: f32,
    pub inequality_pack_id: Option<String>,
    pub smart_city_pack_id: Option<String>,
}

/// Entire city configuration for a simulation run.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CityConfig {
    pub principle_core: PrincipleCore,
    pub rollout_plan: Vec<RolloutStageKind>,
    pub modalities: Vec<AugmentationModality>,
    pub neighborhoods: Vec<NeighborhoodContext>,
    pub agents: Vec<Agent>,
}
