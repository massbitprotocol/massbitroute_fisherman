use std::str::FromStr;
use crate::{BlockChainType, ComponentId, NetworkType};
use crate::{Deserialize,Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Default, Hash, PartialEq, Eq)]
pub struct ComponentInfo {
    pub blockchain: BlockChainType,
    pub network: NetworkType,
    pub id: ComponentId,
    #[serde(rename = "userId", default)]
    pub user_id: String,
    pub ip: String,
    #[serde(default)]
    pub zone: Zone,
    #[serde(rename = "countryCode", default)]
    pub country_code: String,
    #[serde(rename = "appKey", default)]
    pub token: String,
    #[serde(rename = "componentType", default)]
    pub component_type: ComponentType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(default)]
    pub status: String,
}


#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Hash, Eq)]
pub enum ComponentType {
    Node,
    Gateway,
}

impl ToString for ComponentType {
    fn to_string(&self) -> String {
        match self {
            ComponentType::Node => "node".to_string(),
            ComponentType::Gateway => "gateway".to_string(),
        }
    }
}

impl std::default::Default for ComponentType {
    fn default() -> Self {
        ComponentType::Node
    }
}


#[derive(Debug, PartialEq, Clone, Deserialize, Serialize, Hash, Eq)]
pub enum Zone {
    // Asia
    AS,
    // Europe
    EU,
    // North America
    NA,
    // South america
    SA,
    // Africa
    AF,
    // Oceania
    OC,
    // Global
    GB,
}

impl FromStr for Zone {
    type Err = ();

    fn from_str(input: &str) -> Result<Zone, Self::Err> {
        match input {
            "AS" => Ok(Zone::AS),
            "EU" => Ok(Zone::EU),
            "NA" => Ok(Zone::NA),
            "SA" => Ok(Zone::SA),
            "AF" => Ok(Zone::AF),
            "OC" => Ok(Zone::OC),
            "GB" => Ok(Zone::GB),
            _ => Err(()),
        }
    }
}

impl Default for Zone {
    fn default() -> Self {
        Zone::GB
    }
}
