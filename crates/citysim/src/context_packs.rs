use serde::{Serialize, Deserialize};

/// Tribal Sovereignty & FPIC context pack.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TribalFpicPack {
    pub id: String,
    pub community_name: String,
    pub requires_fpic: bool,
    pub neurorights_law_ref: String,
    pub community_veto_probability: f32,   // likelihood of veto when unrest risk rises
    pub fpic_minimum_turns: u32,           // min “consultation ticks” before any rollout
}

/// Municipal Smart-City Regulations pack.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SmartCityPack {
    pub id: String,
    pub city_name: String,
    pub has_decidim_like_platform: bool,
    pub participatory_budgeting_strength: f32, // 0–1
    pub surveillance_intensity: f32,           // 0–1
    pub ai_transparency_policy_strength: f32,  // 0–1
}

/// Local Inequality Metrics pack.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InequalityPack {
    pub id: String,
    pub region_name: String,
    pub gini_income: f32,
    pub gini_healthcare_access: f32,
    pub spatial_segregation_index: f32,
    pub stigma_disability: f32,
    pub stigma_mental_health: f32,
}
