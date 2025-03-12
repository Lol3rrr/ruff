pub mod bitskins;
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
    Capsule {
        name: &'s str,
    },
    PinsCapsule {
        name: &'s str,
    },
    PatchPack {
        name: &'s str,
    },
    MusicKitBox {
        name: &'s str,
    },
    MusicKit {
        name: &'s str,
        stattrak: bool,
    },
    GraffitiBox {
        name: &'s str,
    },
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
    Agent {
        name: &'s str,
    },
    SealedGraffiti {
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
            Some(("Sticker", _)) => Ok(Self::Sticker { name: value }),
            Some(("Patch", _)) => Ok(Self::Patch { name: value }),
            Some(("Charm", _)) => Ok(Self::Charm { name: value }),
            Some(("Autograph Capsule", _)) => Ok(Self::Capsule { name: value }),
            Some(("Sealed Graffiti", _)) => Ok(Self::SealedGraffiti { name: value }),
            Some((_, "The Professionals"))
            | Some((_, "Guerrilla Warfare"))
            | Some((_, "SWAT"))
            | Some((_, "KSK"))
            | Some((_, "Sabre"))
            | Some((_, "SAS"))
            | Some((_, "Phoenix"))
            | Some((_, "Sabre Footsoldier"))
            | Some((_, "NSWC SEAL"))
            | Some((_, "NZSAS"))
            | Some((_, "Elite Crew"))
            | Some((_, "SEAL Frogman"))
            | Some((_, "FBI"))
            | Some((_, "FBI SWAT"))
            | Some((_, "FBI HRT"))
            | Some((_, "FBI Sniper"))
            | Some((_, "Brazilian 1st Battalion"))
            | Some((_, "TACP Cavalry"))
            | Some((_, "USAF TACP")) => Ok(Self::Agent { name: value }),
            Some((_, "Gendarmerie Nationale")) => Ok(Self::Agent { name: value }),
            Some(("Music Kit", _)) => Ok(Self::MusicKit {
                name: value,
                stattrak: false,
            }),
            Some(("StatTrak™ Music Kit", _)) => Ok(Self::MusicKit {
                name: value.strip_prefix("StatTrak™ ").unwrap(),
                stattrak: true,
            }),
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

                if value.contains("Pins Capsule") {
                    return Ok(Self::PinsCapsule { name: value });
                }

                if value.contains("Capsule") {
                    return Ok(Self::Capsule { name: value });
                }

                if value.contains("Patch Pack") {
                    return Ok(Self::PatchPack { name: value });
                }

                if value.contains("Music Kit Box") {
                    return Ok(Self::MusicKitBox { name: value });
                }

                if value.contains("Graffiti Box") {
                    return Ok(Self::GraffitiBox { name: value });
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

    #[test]
    fn autograph_capsule() {
        let name = "Autograph Capsule | Gambit Gaming | Atlanta 2017";

        let expected = Item::Capsule {
            name: "Autograph Capsule | Gambit Gaming | Atlanta 2017",
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn agents_the_professionals() {
        let name = "Sir Bloody Loudmouth Darryl | The Professionals";

        let expected = Item::Agent {
            name: "Sir Bloody Loudmouth Darryl | The Professionals",
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn agents_guerilla_warfare() {
        let name = "Trapper | Guerrilla Warfare";

        let expected = Item::Agent {
            name: "Trapper | Guerrilla Warfare",
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn agents_swat() {
        let name = "1st Lieutenant Farlow | SWAT";

        let expected = Item::Agent {
            name: "1st Lieutenant Farlow | SWAT",
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn agents_ksk() {
        let name = "3rd Commando Company | KSK";

        let expected = Item::Agent {
            name: "3rd Commando Company | KSK",
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn agents_sabre() {
        let name = "Blackwolf | Sabre";

        let expected = Item::Agent {
            name: "Blackwolf | Sabre",
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn agents_gendarmerie_nationale() {
        let name = "Aspirant | Gendarmerie Nationale";

        let expected = Item::Agent {
            name: "Aspirant | Gendarmerie Nationale",
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn agents_nswc_seal() {
        let name = "'Blueberries' Buckshot | NSWC SEAL";

        let expected = Item::Agent {
            name: "'Blueberries' Buckshot | NSWC SEAL",
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn agents_sas() {
        let name = "B Squadron Officer | SAS";

        let expected = Item::Agent {
            name: "B Squadron Officer | SAS",
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn agents_seal_frogman() {
        let name = "Cmdr. Davida 'Goggles' Fernandez | SEAL Frogman";

        let expected = Item::Agent {
            name: "Cmdr. Davida 'Goggles' Fernandez | SEAL Frogman",
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn agents_sabre_footsoldier() {
        let name = "Dragomir | Sabre Footsoldier";

        let expected = Item::Agent {
            name: "Dragomir | Sabre Footsoldier",
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn agents_nzsas() {
        let name = "D Squadron Officer | NZSAS";

        let expected = Item::Agent {
            name: "D Squadron Officer | NZSAS",
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn agents_phoenix() {
        let name = "Enforcer | Phoenix";

        let expected = Item::Agent {
            name: "Enforcer | Phoenix",
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn agents_elite_crew() {
        let name = "Ground Rebel  | Elite Crew";

        let expected = Item::Agent {
            name: "Ground Rebel  | Elite Crew",
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn agents_fbi_hrt() {
        let name = "Markus Delrow | FBI HRT";

        let expected = Item::Agent {
            name: "Markus Delrow | FBI HRT",
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn agents_fbi_sniper() {
        let name = "Michael Syfers  | FBI Sniper";

        let expected = Item::Agent {
            name: "Michael Syfers  | FBI Sniper",
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn agents_fbi_swat() {
        let name = "Operator | FBI SWAT";

        let expected = Item::Agent {
            name: "Operator | FBI SWAT",
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn agents_brazilian_1st_battalion() {
        let name = "Primeiro Tenente | Brazilian 1st Battalion";

        let expected = Item::Agent {
            name: "Primeiro Tenente | Brazilian 1st Battalion",
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn agents_fbi() {
        let name = "Special Agent Ava | FBI";

        let expected = Item::Agent {
            name: "Special Agent Ava | FBI",
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn agents_tacp_cavalry() {
        let name = "'Two Times' McCoy | TACP Cavalry";

        let expected = Item::Agent {
            name: "'Two Times' McCoy | TACP Cavalry",
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn agents_usaf_tacp() {
        let name = "'Two Times' McCoy | USAF TACP";

        let expected = Item::Agent {
            name: "'Two Times' McCoy | USAF TACP",
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn warhammer_sticker_capsule() {
        let name = "Warhammer 40,000 Sticker Capsule";

        let expected = Item::Capsule {
            name: "Warhammer 40,000 Sticker Capsule",
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn stockholm_legends_patch_pack() {
        let name = "Stockholm 2021 Legends Patch Pack";

        let expected = Item::PatchPack {
            name: "Stockholm 2021 Legends Patch Pack",
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn collectible_pins_capsule() {
        let name = "Half-Life: Alyx Collectible Pins Capsule";

        let expected = Item::PinsCapsule {
            name: "Half-Life: Alyx Collectible Pins Capsule",
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn nightmode_music_kit_box() {
        let name = "NIGHTMODE Music Kit Box";

        let expected = Item::MusicKitBox {
            name: "NIGHTMODE Music Kit Box",
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn perfect_world_graffiti_box() {
        let name = "Perfect World Graffiti Box";

        let expected = Item::GraffitiBox {
            name: "Perfect World Graffiti Box",
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn sealed_graffiti_sherrif_tiger_orange() {
        let name = "Sealed Graffiti | Sheriff (Tiger Orange)";

        let expected = Item::SealedGraffiti {
            name: "Sealed Graffiti | Sheriff (Tiger Orange)",
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn music_kit() {
        let name = "Music Kit | Noisia, Sharpened";

        let expected = Item::MusicKit {
            name: "Music Kit | Noisia, Sharpened",
            stattrak: false,
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }

    #[test]
    fn stattrak_music_kit() {
        let name = "StatTrak™ Music Kit | Sam Marshall, Bodacious";

        let expected = Item::MusicKit {
            name: "Music Kit | Sam Marshall, Bodacious",
            stattrak: true,
        };

        assert_eq!(expected, Item::try_from(name).unwrap());
    }
}
