use std::fmt::Display;

use serde::Deserialize;

pub mod buff;
pub mod openexchange;
pub mod steam;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase", tag = "kind")]
pub enum ItemKind {
    Case {
        buff_id: u64,
        bought_at: Option<f64>,
    },
    Weapon {
        conditions: Vec<CondtionEntry>,
    },
    Knife {
        conditions: Vec<CondtionEntry>,
    },
    Glove {
        conditions: Vec<CondtionEntry>,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct CondtionEntry {
    condition: Condition,
    buff_id: u64,
}

impl From<&ItemKind> for &'static str {
    fn from(value: &ItemKind) -> Self {
        match value {
            ItemKind::Case { .. } => "case",
            ItemKind::Weapon { .. } => "weapon",
            ItemKind::Knife { .. } => "knife",
            ItemKind::Glove { .. } => "glove",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Condition {
    FactoryNew,
    MinimalWear,
    FieldTested,
    WellWorn,
    BattleScarred,
}

impl Display for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FactoryNew => write!(f, "Factory New"),
            Self::MinimalWear => write!(f, "Minimal Wear"),
            Self::FieldTested => write!(f, "Field-Tested"),
            Self::WellWorn => write!(f, "Well-Worn"),
            Self::BattleScarred => write!(f, "Battle-Scarred"),
        }
    }
}

impl From<&Condition> for &'static str {
    fn from(value: &Condition) -> Self {
        match value {
            Condition::FactoryNew => "factory-new",
            Condition::MinimalWear => "minimal-wear",
            Condition::FieldTested => "field-tested",
            Condition::WellWorn => "well-worn",
            Condition::BattleScarred => "battle-scarred",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigItem {
    pub name: String,
    #[serde(flatten)]
    pub kind: ItemKind,
}

#[derive(Debug)]
pub struct Item<'s> {
    pub name: String,
    pub buff_id: u64,
    pub kind: &'s str,
    pub condition: &'s str,
    pub bought_at: Option<f64>,
}

impl ConfigItem {
    pub fn to_items(&self) -> Vec<Item<'static>> {
        match &self.kind {
            ItemKind::Case { buff_id, bought_at } => {
                vec![Item {
                    name: self.name.clone(),
                    buff_id: *buff_id,
                    kind: "case",
                    condition: "",
                    bought_at: bought_at.clone(),
                }]
            }
            ItemKind::Weapon { conditions } => conditions
                .into_iter()
                .map(|condition| Item {
                    name: format!("{} ({})", self.name, condition.condition),
                    buff_id: condition.buff_id,
                    kind: "weapon",
                    condition: (&condition.condition).into(),
                    bought_at: None,
                })
                .collect(),
            ItemKind::Knife { conditions } => conditions
                .into_iter()
                .map(|condition| Item {
                    name: format!("{} ({})", self.name, condition.condition),
                    buff_id: condition.buff_id,
                    kind: "knife",
                    condition: (&condition.condition).into(),
                    bought_at: None,
                })
                .collect(),
            ItemKind::Glove { conditions } => conditions
                .into_iter()
                .map(|condition| Item {
                    name: format!("{} ({})", self.name, condition.condition),
                    buff_id: condition.buff_id,
                    kind: "glove",
                    condition: (&condition.condition).into(),
                    bought_at: None,
                })
                .collect(),
        }
    }
}
