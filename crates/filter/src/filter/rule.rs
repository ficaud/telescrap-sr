use parser::core::encounter::Encounter;

use crate::filter::Filter;
use crate::filter::filter_chain::FilterChain;

/// A Rule combines a FilterChain (AND logic between filters) with an action to execute
/// when the chain produces at least one matching encounter.
pub struct Rule {
    pub name: String,
    chain: FilterChain,
    action: Box<dyn Fn(&[Encounter])>,
}

impl Rule {
    pub fn new(
        name: impl Into<String>,
        chain: FilterChain,
        action: impl Fn(&[Encounter]) + 'static,
    ) -> Self {
        Rule {
            name: name.into(),
            chain,
            action: Box::new(action),
        }
    }

    pub fn run(&self, encounters: &Vec<Encounter>) {
        let result = self.chain.apply(encounters);
        if !result.is_empty() {
            (self.action)(&result);
        }
    }
}

/// A RuleSet holds multiple Rules and runs each one independently (OR logic).
/// Each Rule that matches triggers its own action.
pub struct RuleSet {
    rules: Vec<Rule>,
}

impl RuleSet {
    pub fn new() -> Self {
        RuleSet { rules: vec![] }
    }

    pub fn add(mut self, rule: Rule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Runs all rules against the given encounters.
    /// Each matching rule triggers its own action independently.
    pub fn run(&self, encounters: &Vec<Encounter>) {
        for rule in &self.rules {
            rule.run(encounters);
        }
    }
}
