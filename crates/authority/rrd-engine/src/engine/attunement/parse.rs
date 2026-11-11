//! Durable, resumable Parse. Each Step will parse a bounded batch of the
//! Inventory files that have no facts for their current content and commit those
//! facts with the job transition, so a restart resumes where the last commit left
//! off and an edit re-parses only the edited file.

use super::*;

// The executor that writes and reads these records lands in a later pull request.
#[allow(dead_code)]
mod records;
// The executor drives the worker in a later pull request.
#[allow(dead_code)]
mod worker;
