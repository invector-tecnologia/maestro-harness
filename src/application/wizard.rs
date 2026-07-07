//! Creation wizards (TASK 013).
//!
//! Guided creation of personas/scopes/skills. This module holds the wizard's
//! required-field logic (pure and testable); the interactive TUI presentation
//! composes it with Niobium panels in the frontend.

use std::collections::BTreeMap;

use thiserror::Error;

/// A single field in a wizard form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldSpec {
    /// Field key.
    pub name: String,
    /// Whether the field must be filled before completion.
    pub required: bool,
}

impl FieldSpec {
    /// A required field.
    pub fn required(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            required: true,
        }
    }
    /// An optional field.
    pub fn optional(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            required: false,
        }
    }
}

/// A wizard form that blocks completion until required fields are filled.
#[derive(Debug)]
pub struct WizardForm {
    fields: Vec<FieldSpec>,
    values: BTreeMap<String, String>,
}

/// Wizard completion failures.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum WizardError {
    /// One or more required fields are still empty.
    #[error("required field(s) missing: {0}")]
    MissingRequired(String),
}

impl WizardForm {
    /// Build a form from its field specification.
    pub fn new(fields: Vec<FieldSpec>) -> Self {
        Self {
            fields,
            values: BTreeMap::new(),
        }
    }

    /// Set a known field's value (unknown fields are ignored). Blank values clear.
    pub fn set(&mut self, name: &str, value: impl Into<String>) {
        if self.fields.iter().any(|f| f.name == name) {
            let value = value.into();
            if value.trim().is_empty() {
                self.values.remove(name);
            } else {
                self.values.insert(name.to_string(), value);
            }
        }
    }

    /// Required fields that are still empty.
    pub fn missing_required(&self) -> Vec<String> {
        self.fields
            .iter()
            .filter(|f| f.required && !self.values.contains_key(&f.name))
            .map(|f| f.name.clone())
            .collect()
    }

    /// Whether the form may be completed.
    pub fn can_complete(&self) -> bool {
        self.missing_required().is_empty()
    }

    /// Complete the wizard, returning the collected values or the blocking error.
    pub fn complete(self) -> Result<BTreeMap<String, String>, WizardError> {
        let missing = self.missing_required();
        if missing.is_empty() {
            Ok(self.values)
        } else {
            Err(WizardError::MissingRequired(missing.join(", ")))
        }
    }
}

/// The persona-creation wizard: name + responsibility are required.
pub fn persona_wizard() -> WizardForm {
    WizardForm::new(vec![
        FieldSpec::required("name"),
        FieldSpec::required("responsibility"),
        FieldSpec::optional("skills"),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_until_required_filled() {
        let mut form = persona_wizard();
        assert!(!form.can_complete());
        assert_eq!(form.missing_required(), vec!["name", "responsibility"]);

        form.set("name", "Reviewer");
        assert!(!form.can_complete());

        form.set("responsibility", "review code");
        assert!(form.can_complete());
    }

    #[test]
    fn complete_returns_values() {
        let mut form = persona_wizard();
        form.set("name", "Reviewer");
        form.set("responsibility", "review code");
        form.set("skills", "rust");
        let values = form.complete().unwrap();
        assert_eq!(values.get("name").unwrap(), "Reviewer");
        assert_eq!(values.get("skills").unwrap(), "rust");
    }

    #[test]
    fn blank_required_is_rejected() {
        let mut form = persona_wizard();
        form.set("name", "   ");
        form.set("responsibility", "x");
        let err = form.complete().unwrap_err();
        assert_eq!(err, WizardError::MissingRequired("name".to_string()));
    }
}
