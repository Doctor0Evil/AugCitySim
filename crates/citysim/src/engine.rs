use crate::types::*;
use crate::context_packs::*;
use serde::{Serialize, Deserialize};

/// Per-stage policy knobs for simulation (not hardware control).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StagePolicy {
    pub stage: RolloutStageKind,
    pub max_new_augmentations_per_tick: u32,
    pub prioritize_therapeutic: bool,
    pub allow_invasive: bool,
    pub require_fpic_on_tribal_land: bool,
}

/// One simulation tick output.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TickResult {
    pub tick: u32,
    pub stage: RolloutStageKind,
    pub social_metrics: SocialMetrics,
    pub fpic_blocked_projects: u32,
    pub community_vetoes: u32,
    pub errority_events: u32,
}

/// Global simulation state (outer domain only).
pub struct SimulationState {
    pub cfg: CityConfig,
    pub tribal_packs: Vec<TribalFpicPack>,
    pub smart_packs: Vec<SmartCityPack>,
    pub inequality_packs: Vec<InequalityPack>,
    pub stage_policies: Vec<StagePolicy>,
    pub current_stage_idx: usize,
    pub tick: u32,
}

impl SimulationState {
    pub fn new(
        cfg: CityConfig,
        tribal_packs: Vec<TribalFpicPack>,
        smart_packs: Vec<SmartCityPack>,
        inequality_packs: Vec<InequalityPack>,
        stage_policies: Vec<StagePolicy>,
    ) -> Self {
        Self {
            cfg,
            tribal_packs,
            smart_packs,
            inequality_packs,
            stage_policies,
            current_stage_idx: 0,
            tick: 0,
        }
    }

    fn current_stage(&self) -> RolloutStageKind {
        self.cfg.rollout_plan[self.current_stage_idx].clone()
    }

    /// Advance simulation by one tick.
    pub fn step(&mut self) -> TickResult {
        let stage = self.current_stage();
        let policy = self.stage_policies
            .iter()
            .find(|p| p.stage == stage)
            .expect("missing policy for stage");

        self.tick += 1;

        // 1) Compute which modalities are “deployable” this tick in this stage.
        let deployable: Vec<&AugmentationModality> = self.cfg
            .modalities
            .iter()
            .filter(|m| m.enabled_in_stage.contains(&stage))
            .filter(|m| match m.quadrant {
                AugmentationQuadrant::TherapeuticInvasive
                | AugmentationQuadrant::EnhancingInvasive => policy.allow_invasive,
                _ => true,
            })
            .collect();

        // 2) For simplicity, attempt new augmentations up to policy bound.
        let mut new_aug_count = 0u32;
        let mut fpic_blocked = 0u32;
        let mut vetoes = 0u32;
        let mut errority = 0u32;

        for agent in &mut self.cfg.agents {
            if new_aug_count >= policy.max_new_augmentations_per_tick {
                break;
            }

            if agent.augmentation.has_any {
                continue; // focus on first-time uptake for clarity
            }

            // Look up neighborhood context and packs.
            let n_ctx = self.cfg.neighborhoods
                .iter()
                .find(|n| n.id == agent.neighborhood_id)
                .expect("neighborhood missing");

            // FPIC gate on tribal land.
            if n_ctx.is_tribal_land && policy.require_fpic_on_tribal_land {
                fpic_blocked += 1;
                // Tribal FPIC: treat as blocked unless separate FPIC sim passed.
                continue;
            }

            // Simple, outer-domain “desire” condition: higher income → more enhancement demand.
            let wants_enhancement = agent.income_band >= 3 &&
                agent.fear_of_augmentation < 0.5 &&
                agent.trust_in_institutions > 0.4;

            let candidate = if policy.prioritize_therapeutic {
                deployable.iter()
                    .find(|m| matches!(m.purpose, PurposeKind::Therapeutic))
                    .or_else(|| deployable.first())
            } else if wants_enhancement {
                deployable.iter()
                    .find(|m| matches!(m.purpose, PurposeKind::Enhancing))
                    .or_else(|| deployable.first())
            } else {
                deployable.first()
            };

            let modality = match candidate {
                Some(m) => *m,
                None => continue,
            };

            // 3) Call into your inner RoH / NEVL guard for this *proposed* augmentation.
            //    Here we assume a host-local enclave API:
            //
            //    let verdict = nevl_guard::evaluate_social_simulation_proposal(
            //         agent.id.clone(), modality.id.clone(), modality.roh_risk_hint);
            //
            //    For this crate, we just require that roh_risk_hint <= principle_core.hard_bci_roh_ceiling.

            if modality.roh_risk_hint > self.cfg.principle_core.hard_bci_roh_ceiling {
                // Treat this as an Errority event in the sim: unsafe offering.
                errority += 1;
                continue;
            }

            // 4) Apply social side-effects (outer domain only).
            agent.augmentation.has_any = true;
            agent.augmentation.modalities.push(modality.id.clone());
            agent.perceived_benefit = (agent.perceived_benefit + 0.1).min(1.0);
            // Non-augmented peers in same neighborhood feel more coerced.
            for peer in &mut self.cfg.agents {
                if peer.neighborhood_id == agent.neighborhood_id && !peer.augmentation.has_any {
                    peer.perceived_coercion =
                        (peer.perceived_coercion + 0.02).min(1.0);
                }
            }

            new_aug_count += 1;
        }

        // 5) Compute aggregate social metrics from current agent state and inequality packs.
        let metrics = self.compute_social_metrics();

        // 6) Tribal veto logic: if unrest exceeds threshold on any tribal neighborhood, trigger veto.
        for n_ctx in &self.cfg.neighborhoods {
            if !n_ctx.is_tribal_land {
                continue;
            }
            let unrest = metrics.civil_unrest_probability;
            // Find matching TribalFpicPack, if any.
            if let Some(pack) = self.tribal_packs.iter()
                .find(|p| p.id == n_ctx.inequality_pack_id.clone().unwrap_or_default())
            {
                if unrest > 0.5 && pack.requires_fpic {
                    vetoes += 1;
                    // Effect: next stage cannot advance until governance process modeled elsewhere.
                }
            }
        }

        TickResult {
            tick: self.tick,
            stage,
            social_metrics: metrics,
            fpic_blocked_projects: fpic_blocked,
            community_vetoes: vetoes,
            errority_events: errority,
        }
    }

    fn compute_social_metrics(&self) -> SocialMetrics {
        // Very simplified; in practice you’d compute Gini, segregation, etc.
        let mut augmented = 0u32;
        let mut non_augmented = 0u32;
        let mut coercion_sum = 0.0f32;
        let mut stigma_sum = 0.0f32;

        for a in &self.cfg.agents {
            if a.augmentation.has_any {
                augmented += 1;
            } else {
                non_augmented += 1;
            }
            coercion_sum += a.perceived_coercion;
            if a.is_disabled {
                stigma_sum += 0.1; // placeholder; link to inequality packs in a full build
            }
        }

        let total = (augmented + non_augmented).max(1) as f32;
        let share_aug = augmented as f32 / total;
        let gini_aug_gap = (share_aug - 0.5).abs(); // crude proxy: 0.0 = balanced, 0.5 = extreme.
        let avg_coercion = coercion_sum / total;
        let avg_stigma = stigma_sum / total;

        SocialMetrics {
            gini_income: 0.0,                // imported from inequality pack at init
            gini_augmented_vs_non: gini_aug_gap,
            segregation_index: 0.0,          // from neighborhood patterns
            stigma_index: avg_stigma,
            perceived_coercion_index: avg_coercion,
            civil_unrest_probability: (gini_aug_gap + avg_coercion).min(1.0),
        }
    }
}
