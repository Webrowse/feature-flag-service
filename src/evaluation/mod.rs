use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use uuid::Uuid;

// User context for evaluation
#[derive(Debug, Deserialize, Clone)]
pub struct UserContext {
    pub user_id: Option<String>,
    pub user_email: Option<String>,
    #[serde(default)]
    pub custom_attributes: std::collections::HashMap<String, String>,
}

// Flag evaluation result
#[derive(Debug, Serialize)]
pub struct FlagEvaluation {
    pub enabled: bool,
    pub reason: String,
}

// Flag data needed for evaluation
#[derive(Debug, Clone)]
pub struct FlagData {
    pub key: String,
    pub enabled: bool,
    pub rollout_percentage: i32,
}

// Rule data for evaluation
#[derive(Debug, Clone)]
pub struct RuleData {
    pub rule_type: String,
    pub rule_value: String,
    pub enabled: bool,
    pub priority: i32,
}

/// Evaluate if a flag should be enabled for a given user
pub fn evaluate_flag(
    flag: &FlagData,
    rules: &[RuleData],
    context: &UserContext,
) -> FlagEvaluation {
    // Step 1: If flag is globally disabled, return false
    if !flag.enabled {
        return FlagEvaluation {
            enabled: false,
            reason: "Flag is globally disabled".to_string(),
        };
    }

    // Step 2: Sort rules by priority (highest first) and check them
    let mut sorted_rules = rules.to_vec();
    sorted_rules.sort_by(|a, b| b.priority.cmp(&a.priority));

    for rule in sorted_rules.iter() {
        if !rule.enabled {
            continue; // Skip disabled rules
        }

        match rule.rule_type.as_str() {
            "user_id" => {
                if let Some(ref user_id) = context.user_id {
                    if user_id == &rule.rule_value {
                        return FlagEvaluation {
                            enabled: true,
                            reason: format!("Matched user_id rule: {}", rule.rule_value),
                        };
                    }
                }
            }
            "user_email" => {
                if let Some(ref email) = context.user_email {
                    if email == &rule.rule_value {
                        return FlagEvaluation {
                            enabled: true,
                            reason: format!("Matched user_email rule: {}", rule.rule_value),
                        };
                    }
                }
            }
            "email_domain" => {
                if let Some(ref email) = context.user_email {
                    if email.ends_with(&rule.rule_value) {
                        return FlagEvaluation {
                            enabled: true,
                            reason: format!("Matched email_domain rule: {}", rule.rule_value),
                        };
                    }
                }
            }
            _ => {} // Unknown rule type, skip
        }
    }

    // Step 3: Check percentage rollout using consistent hashing
    if flag.rollout_percentage > 0 {
        let user_identifier = context.user_id.as_ref()
            .or(context.user_email.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("anonymous");

        if should_enable_for_percentage(&flag.key, user_identifier, flag.rollout_percentage) {
            return FlagEvaluation {
                enabled: true,
                reason: format!("User in {}% rollout", flag.rollout_percentage),
            };
        } else {
            return FlagEvaluation {
                enabled: false,
                reason: format!("User not in {}% rollout", flag.rollout_percentage),
            };
        }
    }

    // Step 4: Default - flag is enabled globally but no rules matched and no rollout
    FlagEvaluation {
        enabled: true,
        reason: "Flag enabled globally, no specific rules applied".to_string(),
    }
}

/// Consistent hashing for percentage rollout
/// Ensures the same user always gets the same result for a given percentage
fn should_enable_for_percentage(flag_key: &str, user_identifier: &str, percentage: i32) -> bool {
    if percentage == 0 {
        return false;
    }
    if percentage >= 100 {
        return true;
    }

    // Create a consistent hash from flag_key + user_identifier
    let mut hasher = DefaultHasher::new();
    format!("{}:{}", flag_key, user_identifier).hash(&mut hasher);
    let hash = hasher.finish();

    // Map hash to 0-99 range
    let bucket = (hash % 100) as i32;

    // User is in the rollout if their bucket is less than the percentage
    bucket < percentage
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_globally_disabled_flag() {
        let flag = FlagData {
            key: "test_flag".to_string(),
            enabled: false,
            rollout_percentage: 100,
        };
        let rules = vec![];
        let context = UserContext {
            user_id: Some("user123".to_string()),
            user_email: None,
            custom_attributes: Default::default(),
        };

        let result = evaluate_flag(&flag, &rules, &context);
        assert!(!result.enabled);
        assert!(result.reason.contains("globally disabled"));
    }

    #[test]
    fn test_user_id_rule_match() {
        let flag = FlagData {
            key: "test_flag".to_string(),
            enabled: true,
            rollout_percentage: 0,
        };
        let rules = vec![RuleData {
            rule_type: "user_id".to_string(),
            rule_value: "user123".to_string(),
            enabled: true,
            priority: 10,
        }];
        let context = UserContext {
            user_id: Some("user123".to_string()),
            user_email: None,
            custom_attributes: Default::default(),
        };

        let result = evaluate_flag(&flag, &rules, &context);
        assert!(result.enabled);
        assert!(result.reason.contains("user_id"));
    }

    #[test]
    fn test_email_domain_match() {
        let flag = FlagData {
            key: "test_flag".to_string(),
            enabled: true,
            rollout_percentage: 0,
        };
        let rules = vec![RuleData {
            rule_type: "email_domain".to_string(),
            rule_value: "@company.com".to_string(),
            enabled: true,
            priority: 5,
        }];
        let context = UserContext {
            user_id: None,
            user_email: Some("john@company.com".to_string()),
            custom_attributes: Default::default(),
        };

        let result = evaluate_flag(&flag, &rules, &context);
        assert!(result.enabled);
        assert!(result.reason.contains("email_domain"));
    }

    #[test]
    fn test_consistent_hashing() {
        // Same user should always get same result
        let result1 = should_enable_for_percentage("test_flag", "user123", 50);
        let result2 = should_enable_for_percentage("test_flag", "user123", 50);
        assert_eq!(result1, result2);

        // 0% should always be false
        assert!(!should_enable_for_percentage("test_flag", "user123", 0));

        // 100% should always be true
        assert!(should_enable_for_percentage("test_flag", "user123", 100));
    }

    #[test]
    fn test_rule_priority() {
        let flag = FlagData {
            key: "test_flag".to_string(),
            enabled: true,
            rollout_percentage: 0,
        };
        // Higher priority rule should be evaluated first
        let rules = vec![
            RuleData {
                rule_type: "user_id".to_string(),
                rule_value: "user123".to_string(),
                enabled: true,
                priority: 10,
            },
            RuleData {
                rule_type: "email_domain".to_string(),
                rule_value: "@company.com".to_string(),
                enabled: true,
                priority: 5,
            },
        ];
        let context = UserContext {
            user_id: Some("user123".to_string()),
            user_email: Some("john@company.com".to_string()),
            custom_attributes: Default::default(),
        };

        let result = evaluate_flag(&flag, &rules, &context);
        assert!(result.enabled);
        // Should match the higher priority user_id rule
        assert!(result.reason.contains("user_id"));
    }
}