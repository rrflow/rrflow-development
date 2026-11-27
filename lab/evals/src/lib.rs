//! RRFlow Lab's reasoning evaluations. `tasks` defines planning tasks with known
//! answers and scores what an agent produced, `study` checks a study's
//! configuration against cited requirements and audits what was captured, and
//! `provider` runs an agent through a coding-agent CLI under a budget.
pub mod provider;
pub mod study;
pub mod tasks;
