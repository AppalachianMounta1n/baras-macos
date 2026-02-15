//! Condition editors
//!
//! Counter conditions for timer/phase guards.

use dioxus::prelude::*;

use crate::types::{ComparisonOp, CounterCondition};

/// Editor for counter conditions
/// Shows empty by default, selecting a counter enables the condition
#[component]
pub fn CounterConditionEditor(
    condition: Option<CounterCondition>,
    counters: Vec<String>, // Available counter IDs
    on_change: EventHandler<Option<CounterCondition>>,
) -> Element {
    // Normalize: treat Some with empty counter_id as None
    let effective_condition = condition.clone().filter(|c| !c.counter_id.is_empty());

    let cond = effective_condition.clone().unwrap_or(CounterCondition {
        counter_id: String::new(),
        operator: ComparisonOp::Eq,
        value: 1,
    });

    let op_value = match cond.operator {
        ComparisonOp::Eq => "eq",
        ComparisonOp::Lt => "lt",
        ComparisonOp::Gt => "gt",
        ComparisonOp::Lte => "lte",
        ComparisonOp::Gte => "gte",
        ComparisonOp::Ne => "ne",
    };

    // Use a sentinel for the "none" option so that Dioxus value matching works reliably
    let selected_value = if cond.counter_id.is_empty() {
        "__none__".to_string()
    } else {
        cond.counter_id.clone()
    };

    rsx! {
        div { class: "flex items-center gap-xs",
            // Counter ID selector (__none__ = no condition)
            select {
                class: "select",
                style: "width: 140px;",
                value: "{selected_value}",
                onchange: {
                    let cond_clone = cond.clone();
                    move |e| {
                        if e.value() == "__none__" {
                            on_change.call(None);
                        } else {
                            on_change.call(Some(CounterCondition {
                                counter_id: e.value(),
                                operator: cond_clone.operator,
                                value: cond_clone.value,
                            }));
                        }
                    }
                },
                option { value: "__none__", "(none)" }
                if counters.is_empty() {
                    option { value: "__none__", disabled: true, "No counters defined" }
                }
                for counter_id in &counters {
                    option {
                        value: "{counter_id}",
                        "{counter_id}"
                    }
                }
            }

            // Only show operator/value if a counter is selected
            if effective_condition.is_some() {
                // Operator
                select {
                    class: "select",
                    style: "width: 55px;",
                    value: "{op_value}",
                    onchange: {
                        let cond_clone = cond.clone();
                        move |e| {
                            let op = match e.value().as_str() {
                                "eq" => ComparisonOp::Eq,
                                "lt" => ComparisonOp::Lt,
                                "gt" => ComparisonOp::Gt,
                                "lte" => ComparisonOp::Lte,
                                "gte" => ComparisonOp::Gte,
                                "ne" => ComparisonOp::Ne,
                                _ => ComparisonOp::Eq,
                            };
                            on_change.call(Some(CounterCondition {
                                counter_id: cond_clone.counter_id.clone(),
                                operator: op,
                                value: cond_clone.value,
                            }));
                        }
                    },
                    option { value: "eq", "=" }
                    option { value: "lt", "<" }
                    option { value: "gt", ">" }
                    option { value: "lte", "≤" }
                    option { value: "gte", "≥" }
                    option { value: "ne", "≠" }
                }

                // Value
                input {
                    r#type: "number",
                    class: "input-inline",
                    style: "width: 55px;",
                    min: "0",
                    value: "{cond.value}",
                    oninput: {
                        let cond_clone = cond.clone();
                        move |e| {
                            if let Ok(val) = e.value().parse::<u32>() {
                                on_change.call(Some(CounterCondition {
                                    counter_id: cond_clone.counter_id.clone(),
                                    operator: cond_clone.operator,
                                    value: val,
                                }));
                            }
                        }
                    }
                }
            }
        }
    }
}
