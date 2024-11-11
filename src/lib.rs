pub mod buff;
pub mod csfloat;
pub mod openexchange;
pub mod skinport;
pub mod steam;

pub mod config;

mod metrics;
pub use metrics::Metrics;

#[derive(Debug, PartialEq)]
pub enum Item<'s> {
    Case {
        name: &'s str,
    },
    Package {
        name: &'s str,
    },
    Capsule,
    Weapon {
        name: &'s str,
        weapon: &'s str,
        skin: &'s str,
        condition: Condition,
        stattrak: bool,
        souvenir: bool,
    },
    Special {
        name: &'s str,
        weapon: &'s str,
        skin: &'s str,
        condition: Condition,
        stattrak: bool,
    },
    Sticker {
        name: &'s str,
    },
    Patch {
        name: &'s str,
    },
    Charm {
        name: &'s str,
    },
    Other {
        name: &'s str,
    },
}

#[derive(Debug, PartialEq)]
pub enum Condition {
    FactorNew,
    MinimalWear,
    FieldTested,
    WellWorn,
    BattleScarred,
}

impl<'s> TryFrom<&'s str> for Item<'s> {
    type Error = ();

    fn try_from(value: &'s str) -> Result<Self, Self::Error> {
        match value.split_once('|').map(|(f, s)| (f.trim(), s.trim())) {
            Some((weapon, _)) if weapon == "Sticker" => Ok(Self::Sticker { name: value }),
            Some((weapon, _)) if weapon == "Patch" => Ok(Self::Patch { name: value }),
            Some((weapon, _)) if weapon == "Charm" => Ok(Self::Charm { name: value }),
            Some((weapon, second)) => {
                let (skin, raw_condition) = second
                    .rsplit_once('(')
                    .and_then(|(l, r)| Some((l.trim(), r.strip_suffix(')')?)))
                    .ok_or(())?;
                let condition = Condition::try_from(raw_condition)?;

                let (weapon, is_special) = match weapon.strip_prefix("★") {
                    Some(w) => (w.trim(), true),
                    None => (weapon, false),
                };

                let (weapon, stattrak) = match weapon.strip_prefix("StatTrak™") {
                    Some(w) => (w.trim(), true),
                    None => (weapon, false),
                };

                let (weapon, souvenir) = match weapon.strip_prefix("Souvenir") {
                    Some(w) => (w.trim(), true),
                    None => (weapon, false),
                };

                if !is_special {
                    Ok(Self::Weapon {
                        name: value,
                        weapon,
                        skin,
                        condition,
                        stattrak,
                        souvenir,
                    })
                } else {
                    Ok(Self::Special {
                        name: value,
                        weapon,
                        skin,
                        condition,
                        stattrak,
                    })
                }
            }
            None if value.starts_with('★') => {
                let weapon = value.strip_prefix('★').unwrap().trim();

                let (weapon, stattrak) = match weapon.strip_prefix("StatTrak™") {
                    Some(w) => (w.trim(), true),
                    None => (weapon, false),
                };

                Ok(Self::Special {
                    name: value,
                    weapon,
                    stattrak,
                    skin: "Vanilla",
                    condition: Condition::FactorNew,
                })
            }
            None => {
                if value.contains("Case") {
                    return Ok(Self::Case { name: value });
                }

                if value.contains("Package") {
                    return Ok(Self::Package { name: value });
                }

                Ok(Self::Other { name: value })
            }
        }
    }
}

impl TryFrom<&str> for Condition {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Factory New" => Ok(Self::FactorNew),
            "Minimal Wear" => Ok(Self::MinimalWear),
            "Field-Tested" => Ok(Self::FieldTested),
            "Well-Worn" => Ok(Self::WellWorn),
            "Battle-Scarred" => Ok(Self::BattleScarred),
            _ => Err(()),
        }
    }
}

impl Condition {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::FactorNew => "factory-new",
            Self::MinimalWear => "minimal-wear",
            Self::FieldTested => "field-tested",
            Self::WellWorn => "well-worn",
            Self::BattleScarred => "battle-scarred",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ak_vulcan() {
        let name_factory_new = "AK-47 | Vulcan (Factory New)";
        let name_minimal_wear = "AK-47 | Vulcan (Minimal Wear)";
        let name_field_tested = "AK-47 | Vulcan (Field-Tested)";
        let name_well_worn = "AK-47 | Vulcan (Well-Worn)";
        let name_battle_scarred = "AK-47 | Vulcan (Battle-Scarred)";

        let expected_fn = Item::Weapon {
            name: name_factory_new,
            weapon: "AK-47",
            skin: "Vulcan",
            condition: Condition::FactorNew,
            souvenir: false,
            stattrak: false,
        };
        let expected_mw = Item::Weapon {
            name: name_minimal_wear,
            weapon: "AK-47",
            skin: "Vulcan",
            condition: Condition::MinimalWear,
            souvenir: false,
            stattrak: false,
        };
        let expected_ft = Item::Weapon {
            name: name_field_tested,
            weapon: "AK-47",
            skin: "Vulcan",
            condition: Condition::FieldTested,
            souvenir: false,
            stattrak: false,
        };
        let expected_ww = Item::Weapon {
            name: name_well_worn,
            weapon: "AK-47",
            skin: "Vulcan",
            condition: Condition::WellWorn,
            souvenir: false,
            stattrak: false,
        };
        let expected_bs = Item::Weapon {
            name: name_battle_scarred,
            weapon: "AK-47",
            skin: "Vulcan",
            condition: Condition::BattleScarred,
            souvenir: false,
            stattrak: false,
        };

        assert_eq!(expected_fn, Item::try_from(name_factory_new).unwrap());
        assert_eq!(expected_mw, Item::try_from(name_minimal_wear).unwrap());
        assert_eq!(expected_ft, Item::try_from(name_field_tested).unwrap());
        assert_eq!(expected_ww, Item::try_from(name_well_worn).unwrap());
        assert_eq!(expected_bs, Item::try_from(name_battle_scarred).unwrap());
    }

    #[test]
    fn stattrak_vulcan() {
        let name = "StatTrak™ AK-47 | Vulcan (Factory New)";

        let expected = Item::Weapon {
            name,
            weapon: "AK-47",
            skin: "Vulcan",
            condition: Condition::FactorNew,
            souvenir: false,
            stattrak: true,
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn souvenir_welcome_to_the_jungle() {
        let name = "Souvenir M4A1-S | Welcome to the Jungle (Factory New)";

        let expected = Item::Weapon {
            name,
            weapon: "M4A1-S",
            skin: "Welcome to the Jungle",
            condition: Condition::FactorNew,
            souvenir: true,
            stattrak: false,
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn karambit_doppler() {
        let name = "★ Karambit | Doppler (Factory New)";

        let expected = Item::Special {
            name,
            weapon: "Karambit",
            skin: "Doppler",
            condition: Condition::FactorNew,
            stattrak: false,
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn karambit_vanilla() {
        let name = "★ Karambit";

        let expected = Item::Special {
            name,
            weapon: "Karambit",
            skin: "Vanilla",
            condition: Condition::FactorNew,
            stattrak: false,
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn statrak_karambit_vanilla() {
        let name = "★ StatTrak™ Karambit";

        let expected = Item::Special {
            name,
            weapon: "Karambit",
            skin: "Vanilla",
            condition: Condition::FactorNew,
            stattrak: true,
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn danger_zone_case() {
        let name = "Danger Zone Case";

        let expected = Item::Case {
            name: "Danger Zone Case",
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn sticker_zywoo_kato2019() {
        let name = "Sticker | ZywOo | Katowice 2019";

        let expected = Item::Sticker {
            name: "Sticker | ZywOo | Katowice 2019",
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn patch_bayonet_frog() {
        let name = "Patch | Bayonet Frog";

        let expected = Item::Patch {
            name: "Patch | Bayonet Frog",
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn charm_diner_dog() {
        let name = "Charm | Diner Dog";

        let expected = Item::Charm {
            name: "Charm | Diner Dog",
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }
}
