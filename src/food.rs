use std::collections::VecDeque;
use std::{fmt, fmt::Display};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CookingStep {
    Cut,
    Spice,
    Bake,
    Grill,
    Peel,
}

pub static COOKING_STEPS: [CookingStep; 5] = [
    CookingStep::Cut,
    CookingStep::Spice,
    CookingStep::Bake,
    CookingStep::Grill,
    CookingStep::Peel,
];

impl Display for CookingStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CookingStep::Cut => "cutting",
                CookingStep::Spice => "spicing",
                CookingStep::Bake => "baking",
                CookingStep::Grill => "grilling",
                CookingStep::Peel => "peeling",
            }
        )
    }
}

pub struct Food {
    pub name: String,
    pub cooking_steps: VecDeque<CookingStep>,
    pub conditions: Vec<&'static str>,
    pub borked: bool,
}

impl Food {
    pub fn new(name: impl Into<String>, cooking_steps: Vec<CookingStep>) -> Self {
        Self {
            name: name.into(),
            cooking_steps: cooking_steps.into_iter().collect(),
            conditions: vec![],
            borked: false,
        }
    }
    pub fn add_step_result(&mut self, step: CookingStep) {
        self.conditions.push(match step {
            CookingStep::Cut => "cut",
            CookingStep::Spice => "spiced",
            CookingStep::Bake => "baked",
            CookingStep::Grill => "grilled",
            CookingStep::Peel => "peeled",
        });
    }
    pub fn add_mishap(&mut self, step: CookingStep) {
        self.borked = true;
        self.conditions.push(match step {
            CookingStep::Cut => "crooked",
            CookingStep::Spice => "pepper-covered",
            CookingStep::Bake => "melted",
            CookingStep::Grill => "burned",
            CookingStep::Peel => "blood-covered",
        });
    }
}

impl Display for Food {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let conditions = capitalize(&self.conditions.join(" and "));
        write!(f, "{} {}", conditions, &capitalize(&self.name))
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
