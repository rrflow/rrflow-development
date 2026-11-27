//! Native typed selection wire; all effects use the shared bounded transport.
//! Protocol reference: https://docs.typesafe.ai/api/system-one (2026-11-20).
use super::{completed_failure, failure, Result};
use rrd_contract::{
    CanonicalId, EndpointAbstentionReason, EndpointEffortSupport, EndpointSelectionAllowance,
    EndpointSelectionChoice, EndpointTokenUsage,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone)]
pub(super) struct SelectionQuestion {
    choices: BTreeMap<String, EndpointSelectionChoice>,
}

impl SelectionQuestion {
    pub(super) fn new(allowance: &EndpointSelectionAllowance) -> Result<Self> {
        let mut choices = BTreeMap::new();
        for reason in [
            EndpointAbstentionReason::InsufficientEvidence,
            EndpointAbstentionReason::CannotDecide,
        ] {
            insert_choice(&mut choices, EndpointSelectionChoice::Abstain { reason })?;
        }
        if allowance.candidates.is_empty() {
            insert_choice(
                &mut choices,
                EndpointSelectionChoice::Abstain {
                    reason: EndpointAbstentionReason::NoEligibleCandidate,
                },
            )?;
        }
        for candidate in &allowance.candidates {
            let EndpointEffortSupport::Observed {
                allow_default,
                values,
                ..
            } = &candidate.effort
            else {
                continue;
            };
            for effort in allow_default
                .then_some(None)
                .into_iter()
                .chain(values.iter().cloned().map(Some))
            {
                if choices.len() == 255 {
                    return Err(failure("System One choice limit exceeded", false));
                }
                insert_choice(
                    &mut choices,
                    EndpointSelectionChoice::Select {
                        binding_id: candidate.binding_id.clone(),
                        effort,
                    },
                )?;
            }
        }
        Ok(Self { choices })
    }

    pub(super) fn request<'a>(
        &'a self,
        model: &'a CanonicalId,
        instructions: &'a str,
        state: &'a str,
    ) -> impl Serialize + 'a {
        #[derive(Serialize)]
        struct Question<'a> {
            r#type: &'static str,
            instructions: &'a str,
            criteria: &'a BTreeMap<String, EndpointSelectionChoice>,
        }
        #[derive(Serialize)]
        struct Request<'a> {
            model: &'a CanonicalId,
            state: &'a str,
            questions: BTreeMap<&'static str, Question<'a>>,
        }
        Request {
            model,
            state,
            questions: [(
                "endpoint",
                Question {
                    r#type: "choice",
                    instructions,
                    criteria: &self.choices,
                },
            )]
            .into_iter()
            .collect(),
        }
    }

    pub(super) fn decode(
        &self,
        bytes: &[u8],
        model: &CanonicalId,
    ) -> Result<(String, EndpointTokenUsage)> {
        // An undecodable or mismatched envelope cannot establish terminality.
        let response: Response = serde_json::from_slice(bytes)
            .map_err(|_| failure("invalid System One response", true))?;
        if response.model != model.as_str() {
            return Err(failure("System One response model differs", true));
        }
        let answer = response.answers.endpoint;
        let invalid = || completed_failure("invalid System One choice distribution");
        if answer.r#type != "choice"
            || !unit(answer.confidence)
            || answer.probabilities.len() != self.choices.len()
            || answer.probabilities.keys().ne(self.choices.keys())
            || answer.probabilities.values().any(|p| !unit(*p))
            || (answer.probabilities.values().sum::<f64>() - 1.0).abs() > 1e-6
        {
            return Err(invalid());
        }
        let selected = answer
            .probabilities
            .get(&answer.choice)
            .ok_or_else(invalid)?;
        if answer.probabilities.values().any(|p| p > selected) {
            return Err(invalid());
        }
        let choice = self.choices.get(&answer.choice).ok_or_else(invalid)?;
        let usage = EndpointTokenUsage {
            prompt_tokens: Some(response.usage.input_tokens),
            completion_tokens: Some(response.usage.output_tokens),
            total_tokens: Some(
                response
                    .usage
                    .input_tokens
                    .checked_add(response.usage.output_tokens)
                    .ok_or_else(|| completed_failure("inconsistent reported usage"))?,
            ),
        };
        usage
            .validate()
            .map_err(|_| completed_failure("inconsistent reported usage"))?;
        let content = serde_json::to_string(choice)
            .map_err(|_| completed_failure("invalid endpoint selection choice"))?;
        Ok((content, usage))
    }
}

// Bind option names to meaning. Positional names would reinterpret a retained
// answer when semantic narrowing removes or reorders a candidate.
fn insert_choice(
    choices: &mut BTreeMap<String, EndpointSelectionChoice>,
    choice: EndpointSelectionChoice,
) -> Result<()> {
    let bytes =
        serde_json::to_vec(&choice).map_err(|_| failure("invalid System One criterion", false))?;
    choices.insert(rrflow_core::digest::sha256_hex(&bytes), choice);
    Ok(())
}

fn unit(value: f64) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Response {
    model: String,
    answers: Answers,
    usage: Usage,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Answers {
    endpoint: Answer,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Answer {
    r#type: String,
    choice: String,
    confidence: f64,
    #[serde(deserialize_with = "unique_probabilities")]
    probabilities: BTreeMap<String, f64>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Usage {
    input_tokens: u64,
    output_tokens: u64,
}

fn unique_probabilities<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<BTreeMap<String, f64>, D::Error> {
    struct Visitor;
    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = BTreeMap<String, f64>;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("unique bounded choice probabilities")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut map: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut result = BTreeMap::new();
            while let Some((key, value)) = map.next_entry()? {
                if result.len() == 255 || result.insert(key, value).is_some() {
                    return Err(serde::de::Error::custom(
                        "duplicate or excessive probabilities",
                    ));
                }
            }
            Ok(result)
        }
    }
    deserializer.deserialize_map(Visitor)
}
