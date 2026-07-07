//! Maestro meta-orchestrator (TASK 039) — Plan → Delegate → Audit → Deliver.
//!
//! Given a user demand and the persona catalog, it drives a [`MicroProject`]
//! through the FSM (TASK 046), routes personas with the deterministic Two-Towers
//! matcher (TASK 047), and delegates to them in a **serial cascade** (TASK 048).
//! It produces an ordered list of [`Signal`]s; the presentation layer maps those
//! to IPC events. Pure and deterministic — no I/O, no LLM.

use crate::domain::models::fsm::{FsmStage, MicroProject};
use crate::domain::models::rollback::{CascadeStep, RollbackPlan};
use crate::domain::models::routing::route;
use crate::domain::models::Persona;

/// Safety rail: the cascade may never exceed this many steps (TASK 033).
pub const MAX_CASCADE_STEPS: usize = 64;

/// An orchestration observation, emitted in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Signal {
    /// The FSM entered a stage.
    Stage(FsmStage),
    /// The plan Maestro proposes.
    Plan(Vec<String>),
    /// A persona changed cognitive state (`observe`/`think`/`act`/`idle`).
    Agent { persona: String, state: String },
    /// Maestro delegated a task to a persona.
    Delegation { persona: String, task: String },
    /// A persona (or Maestro) delivered a result.
    Deliverable { persona: String, summary: String },
    /// Execution is blocked awaiting a user approval.
    ApprovalRequest { id: String, prompt: String },
    /// A rollback inverse action was applied.
    Rollback { action: String },
    /// A diagnostic line.
    Log { level: String, message: String },
}

/// Produces a persona's deliverable summary for `(persona, model, demand)`. The
/// default [`placeholder_deliverable`] is deterministic; the server injects a
/// provider-backed deliverer when a model is available.
pub type Deliverer<'a> = dyn Fn(&str, &str, &str) -> String + 'a;

/// The deterministic fallback deliverable (no LLM call).
pub fn placeholder_deliverable(persona: &str, _model: &str, _demand: &str) -> String {
    format!("{persona} completed its contribution")
}

/// Orchestrate a demand into an ordered signal stream, auto-approving both gates.
/// Deterministic and gate-free view used for headless runs and tests; the gated
/// [`Session`] powers the interactive server path.
pub fn orchestrate(
    demand: &str,
    personas: &[Persona],
    model_for: impl Fn(&str) -> String,
) -> Vec<Signal> {
    let (mut session, mut signals) = Session::start(demand, personas, &model_for);
    signals.extend(session.resume(true, &placeholder_deliverable));
    signals.extend(session.resume(true, &placeholder_deliverable));
    signals
        .into_iter()
        .filter(|s| !matches!(s, Signal::ApprovalRequest { .. }))
        .collect()
}

/// The gate a [`Session`] is currently blocked on, if any.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SessionState {
    AwaitingPlanApproval,
    AwaitingExecApproval,
    Done,
    Aborted,
    RolledBack,
}

/// A resumable, gated orchestration (TASK 046 gates + TASK 048 cascade + TASK 049
/// rollback). Drives a [`MicroProject`] but pauses at the Approval and Execution
/// gates until the user responds — never assuming consent.
pub struct Session {
    project: MicroProject,
    demand: String,
    /// Selected personas paired with their resolved model.
    selected: Vec<(String, String)>,
    state: SessionState,
    rollback: RollbackPlan,
    deliverables: Vec<String>,
}

impl Session {
    /// Id of the plan-approval gate.
    pub const PLAN_APPROVAL_ID: &'static str = "approve-plan";
    /// Id of the execution-approval gate.
    pub const EXEC_APPROVAL_ID: &'static str = "approve-exec";

    /// Begin a run: Ideation → Planning, then block at the Approval gate.
    pub fn start(
        demand: &str,
        personas: &[Persona],
        model_for: impl Fn(&str) -> String,
    ) -> (Self, Vec<Signal>) {
        let mut project = MicroProject::new("mp-1", demand);
        let mut signals = vec![Signal::Stage(project.stage)]; // Ideation

        advance(&mut project, &mut signals); // Planning
        let routing = route(demand, personas);
        let selected: Vec<(String, String)> = routing
            .selected
            .iter()
            .map(|p| (p.clone(), model_for(p)))
            .collect();
        let mut plan = vec![
            format!("understand: {demand}"),
            format!(
                "route {} persona(s): {}",
                selected.len(),
                routing.selected.join(", ")
            ),
            "delegate in serial cascade".to_string(),
            "audit deliverables".to_string(),
            "deliver".to_string(),
        ];
        if routing.used_fallback {
            plan.push("(no strong match — used fallback persona)".to_string());
        }
        signals.push(Signal::Plan(plan));

        advance(&mut project, &mut signals); // Approval
        signals.push(Signal::ApprovalRequest {
            id: Self::PLAN_APPROVAL_ID.to_string(),
            prompt: format!("Approve the plan for '{demand}'?"),
        });

        let session = Self {
            project,
            demand: demand.to_string(),
            selected,
            state: SessionState::AwaitingPlanApproval,
            rollback: RollbackPlan::new(),
            deliverables: Vec::new(),
        };
        (session, signals)
    }

    /// Whether the session is blocked on an approval gate.
    pub fn is_pending(&self) -> bool {
        matches!(
            self.state,
            SessionState::AwaitingPlanApproval | SessionState::AwaitingExecApproval
        )
    }

    /// Whether the session completed successfully.
    pub fn is_done(&self) -> bool {
        self.state == SessionState::Done
    }

    /// Whether the session is blocked specifically on the execution gate.
    pub fn awaiting_execution(&self) -> bool {
        self.state == SessionState::AwaitingExecApproval
    }

    /// The originating demand.
    pub fn demand(&self) -> &str {
        &self.demand
    }

    /// The recorded deliverables (available once execution completes).
    pub fn deliverables(&self) -> &[String] {
        &self.deliverables
    }

    /// Respond to the current gate; advances or rolls back accordingly. `deliver`
    /// produces each persona's deliverable during execution.
    pub fn resume(&mut self, approved: bool, deliver: &Deliverer) -> Vec<Signal> {
        match self.state {
            SessionState::AwaitingPlanApproval => {
                if approved {
                    self.enter_instrumentation()
                } else {
                    self.abort("plan rejected")
                }
            }
            SessionState::AwaitingExecApproval => {
                if approved {
                    self.execute(deliver)
                } else {
                    self.roll_back()
                }
            }
            _ => Vec::new(),
        }
    }

    /// Plan approved → Instrumentation, build the rollback plan, block at Execution gate.
    fn enter_instrumentation(&mut self) -> Vec<Signal> {
        let mut signals = Vec::new();
        advance(&mut self.project, &mut signals); // Instrumentation
        for (persona, _) in &self.selected {
            signals.push(Signal::Agent {
                persona: persona.clone(),
                state: "observe".to_string(),
            });
            self.rollback.record(CascadeStep::new(
                format!("apply {persona} contribution"),
                format!("revert {persona} contribution"),
            ));
        }
        signals.push(Signal::Log {
            level: "info".to_string(),
            message: format!("rollback plan ready ({} step(s))", self.rollback.len()),
        });
        signals.push(Signal::ApprovalRequest {
            id: Self::EXEC_APPROVAL_ID.to_string(),
            prompt: format!(
                "Approve execution of '{}'? A rollback plan is ready.",
                self.demand
            ),
        });
        self.state = SessionState::AwaitingExecApproval;
        signals
    }

    /// Execution approved → serial cascade → Verification → deliver.
    fn execute(&mut self, deliver: &Deliverer) -> Vec<Signal> {
        let mut signals = Vec::new();
        if self.selected.len() > MAX_CASCADE_STEPS {
            self.state = SessionState::Aborted;
            return vec![Signal::Log {
                level: "error".to_string(),
                message: format!(
                    "cascade of {} exceeds the {MAX_CASCADE_STEPS}-step safety limit",
                    self.selected.len()
                ),
            }];
        }
        advance(&mut self.project, &mut signals); // Execution
        for (persona, model) in &self.selected {
            signals.push(Signal::Agent {
                persona: persona.clone(),
                state: "think".to_string(),
            });
            signals.push(Signal::Agent {
                persona: persona.clone(),
                state: "act".to_string(),
            });
            signals.push(Signal::Delegation {
                persona: persona.clone(),
                task: format!("address '{}' [{}]", self.demand, model),
            });
            let summary = deliver(persona, model, &self.demand);
            self.deliverables.push(format!("{persona}: {summary}"));
            signals.push(Signal::Deliverable {
                persona: persona.clone(),
                summary,
            });
            signals.push(Signal::Agent {
                persona: persona.clone(),
                state: "idle".to_string(),
            });
        }
        advance(&mut self.project, &mut signals); // Verification
        let summary = format!(
            "delivered '{}' via {} persona(s)",
            self.demand,
            self.selected.len()
        );
        self.deliverables.push(format!("Maestro: {summary}"));
        signals.push(Signal::Deliverable {
            persona: "Maestro".to_string(),
            summary,
        });
        self.state = SessionState::Done;
        signals
    }

    /// Execution rejected → apply the rollback plan (inverse, reverse order).
    fn roll_back(&mut self) -> Vec<Signal> {
        let mut signals = vec![Signal::Log {
            level: "warn".to_string(),
            message: "execution rejected — rolling back".to_string(),
        }];
        for action in self.rollback.inverse_order() {
            signals.push(Signal::Rollback { action });
        }
        self.state = SessionState::RolledBack;
        signals
    }

    /// Plan rejected before any work → abort with no side effects.
    fn abort(&mut self, reason: &str) -> Vec<Signal> {
        self.state = SessionState::Aborted;
        vec![Signal::Log {
            level: "warn".to_string(),
            message: format!("{reason} — aborting run"),
        }]
    }
}

/// Advance the project one stage and record the transition (terminal-safe).
fn advance(project: &mut MicroProject, signals: &mut Vec<Signal>) {
    if let Ok(stage) = project.advance() {
        signals.push(Signal::Stage(stage));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::default_personas;

    fn stages(signals: &[Signal]) -> Vec<FsmStage> {
        signals
            .iter()
            .filter_map(|s| match s {
                Signal::Stage(stage) => Some(*stage),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn walks_the_full_fsm_in_order() {
        let signals = orchestrate("build a cli", &default_personas(), |_| {
            "mistral".to_string()
        });
        assert_eq!(stages(&signals), FsmStage::ALL.to_vec());
    }

    #[test]
    fn is_deterministic() {
        let a = orchestrate("design quality assurance", &default_personas(), |_| {
            "m".into()
        });
        let b = orchestrate("design quality assurance", &default_personas(), |_| {
            "m".into()
        });
        assert_eq!(a, b);
    }

    #[test]
    fn execution_cascade_is_serial_per_persona() {
        let signals = orchestrate("improve quality assurance", &default_personas(), |_| {
            "mistral".to_string()
        });
        // Every delegated persona must produce a deliverable.
        let delegated: Vec<&String> = signals
            .iter()
            .filter_map(|s| match s {
                Signal::Delegation { persona, .. } => Some(persona),
                _ => None,
            })
            .collect();
        assert!(!delegated.is_empty());
        for persona in delegated {
            assert!(signals
                .iter()
                .any(|s| matches!(s, Signal::Deliverable { persona: p, .. } if p == persona)));
        }
    }

    #[test]
    fn model_router_annotates_delegations() {
        let signals = orchestrate("build a cli", &default_personas(), |_| {
            "codellama".to_string()
        });
        assert!(signals
            .iter()
            .any(|s| matches!(s, Signal::Delegation { task, .. } if task.contains("codellama"))));
    }

    #[test]
    fn maestro_delivers_at_verification() {
        let signals = orchestrate("ship it", &default_personas(), |_| "m".into());
        let last = signals.last().unwrap();
        assert!(matches!(last, Signal::Deliverable { persona, .. } if persona == "Maestro"));
    }

    #[test]
    fn session_blocks_at_the_plan_gate() {
        let (session, signals) = Session::start("build a cli", &default_personas(), |_| "m".into());
        assert!(session.is_pending());
        assert!(!session.is_done());
        assert!(signals.iter().any(|s| matches!(
            s,
            Signal::ApprovalRequest { id, .. } if id == Session::PLAN_APPROVAL_ID
        )));
        // No delegation happens before approval.
        assert!(!signals
            .iter()
            .any(|s| matches!(s, Signal::Delegation { .. })));
    }

    #[test]
    fn session_two_gates_then_completes() {
        let (mut session, _) =
            Session::start("improve quality assurance", &default_personas(), |_| {
                "m".into()
            });
        let after_plan = session.resume(true, &placeholder_deliverable);
        assert!(after_plan.iter().any(|s| matches!(
            s,
            Signal::ApprovalRequest { id, .. } if id == Session::EXEC_APPROVAL_ID
        )));
        assert!(session.is_pending() && !session.is_done());
        let after_exec = session.resume(true, &placeholder_deliverable);
        assert!(session.is_done());
        assert!(after_exec
            .iter()
            .any(|s| matches!(s, Signal::Deliverable { persona, .. } if persona == "Maestro")));
        assert!(!session.deliverables().is_empty());
    }

    #[test]
    fn rejecting_the_plan_aborts_without_work() {
        let (mut session, _) = Session::start("x", &default_personas(), |_| "m".into());
        let signals = session.resume(false, &placeholder_deliverable);
        assert!(!session.is_pending() && !session.is_done());
        assert!(signals
            .iter()
            .any(|s| matches!(s, Signal::Log { message, .. } if message.contains("aborting"))));
    }

    #[test]
    fn rejecting_execution_rolls_back_in_reverse() {
        let (mut session, _) =
            Session::start("improve quality assurance", &default_personas(), |_| {
                "m".into()
            });
        let _ = session.resume(true, &placeholder_deliverable); // approve plan → exec gate
        let signals = session.resume(false, &placeholder_deliverable); // reject execution
        let rollbacks: Vec<&String> = signals
            .iter()
            .filter_map(|s| match s {
                Signal::Rollback { action } => Some(action),
                _ => None,
            })
            .collect();
        assert!(!rollbacks.is_empty());
        assert!(!session.is_done());
    }

    #[test]
    fn injected_deliverer_output_flows_into_deliverables() {
        let (mut session, _) = Session::start("build a cli", &default_personas(), |_| "m".into());
        let _ = session.resume(true, &placeholder_deliverable); // plan → exec gate
        let signals = session.resume(true, &|persona: &str, _model: &str, _demand: &str| {
            format!("REAL:{persona}")
        });
        assert!(signals.iter().any(
            |s| matches!(s, Signal::Deliverable { summary, .. } if summary.starts_with("REAL:"))
        ));
        assert!(session.deliverables().iter().any(|d| d.contains("REAL:")));
    }
}
