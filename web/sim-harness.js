// Browser/Node harness for configuring and running city-scale augmentation sims.

import initCitySimWasm, {
  SimulationState,
  load_city_config_from_json,
  run_until_stage,
} from "../pkg/citysim_wasm.js";

/**
 * Build a default set of augmentation axes (therapeutic vs enhancing, invasive vs non-invasive).
 */
export function buildDefaultAxesConfig() {
  return {
    modalities: [
      {
        id: "dbs_parkinsons",
        purpose: "Therapeutic",
        invasiveness: "Invasive",
        roh_risk_hint: 0.22,
        enabled_in_stage: ["Stage3LimitedInvasive", "Stage4CityScale"]
      },
      {
        id: "bci_rehab",
        purpose: "Therapeutic",
        invasiveness: "NonInvasive",
        roh_risk_hint: 0.18,
        enabled_in_stage: ["Stage2NonInvasiveNeuro", "Stage3LimitedInvasive", "Stage4CityScale"]
      },
      {
        id: "ar_focus_glasses",
        purpose: "Enhancing",
        invasiveness: "NonInvasive",
        roh_risk_hint: 0.15,
        enabled_in_stage: ["Stage1EnvNonInvasive", "Stage2NonInvasiveNeuro", "Stage4CityScale"]
      },
      {
        id: "cortical_memory_boost",
        purpose: "Enhancing",
        invasiveness: "Invasive",
        roh_risk_hint: 0.28,
        enabled_in_stage: ["Stage3LimitedInvasive", "Stage4CityScale"]
      }
    ]
  };
}

/**
 * Run a full “journey” 0–4 and collect stage metrics.
 */
export async function simulateJourney({ cityConfigJson, maxTicksPerStage = 365 }) {
  await initCitySimWasm();

  const cityConfig = load_city_config_from_json(cityConfigJson);
  const sim = new SimulationState(cityConfig);

  const resultsPerStage = [];

  const rolloutPlan = cityConfig.rollout_plan; // exposed from Rust via serde/wasm-bindgen
  for (let stageIdx = 0; stageIdx < rolloutPlan.length; stageIdx++) {
    const stageKind = rolloutPlan[stageIdx];
    const stageResults = run_until_stage(sim, stageKind, maxTicksPerStage);
    resultsPerStage.push({
      stage: stageKind,
      ticks: stageResults.map(r => ({
        tick: r.tick,
        gini_augmented_vs_non: r.social_metrics.gini_augmented_vs_non,
        perceived_coercion_index: r.social_metrics.perceived_coercion_index,
        civil_unrest_probability: r.social_metrics.civil_unrest_probability,
        fpic_blocked_projects: r.fpic_blocked_projects,
        community_vetoes: r.community_vetoes,
        errority_events: r.errority_events
      }))
    });
  }

  return resultsPerStage;
}
