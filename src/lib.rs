use serde::Deserialize;

pub mod buff;
pub mod openexchange;
pub mod steam;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemKind {
    Case,
    Weapon,
    Knife,
    Glove,
}

impl From<&ItemKind> for &'static str {
    fn from(value: &ItemKind) -> Self {
        match value {
            ItemKind::Case => "case",
            ItemKind::Weapon => "weapon",
            ItemKind::Knife => "knife",
            ItemKind::Glove => "glove",
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
    pub goods_id: u64,
    pub kind: ItemKind,
    pub bought_at: Option<f64>,
    pub condition: Option<Condition>,
}
