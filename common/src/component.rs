use crate::{ComponentId, NetworkType, UrlType};
use crate::{Deserialize, Serialize};
use anyhow::{anyhow, Error};

use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

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

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq)]
pub enum BlockChainType {
    #[serde(rename = "eth")]
    Eth,
    #[serde(rename = "dot")]
    Dot,
    #[serde(rename = "bsc")]
    Bsc,
    #[serde(rename = "matic")]
    Matic,
}

impl BlockChainType {
    pub fn get_family(&self) -> BlockChainFamily {
        match self {
            BlockChainType::Eth | BlockChainType::Bsc | BlockChainType::Matic => {
                BlockChainFamily::Ethereum
            }
            BlockChainType::Dot => BlockChainFamily::Polkadot,
        }
    }
}

impl Default for BlockChainType {
    fn default() -> Self {
        BlockChainType::Eth
    }
}

impl FromStr for BlockChainType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "eth" => Ok(BlockChainType::Eth),
            "dot" => Ok(BlockChainType::Dot),
            "bsc" => Ok(BlockChainType::Bsc),
            "matic" => Ok(BlockChainType::Bsc),
            _ => Err(anyhow!("Cannot parse {} to BlockChainType", s)),
        }
    }
}

impl Display for BlockChainType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            BlockChainType::Eth => {
                write!(f, "eth")
            }
            BlockChainType::Dot => {
                write!(f, "dot")
            }
            BlockChainType::Bsc => {
                write!(f, "bsc")
            }
            BlockChainType::Matic => {
                write!(f, "bsc")
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq)]
pub enum BlockChainFamily {
    Ethereum,
    Polkadot,
}

impl Default for BlockChainFamily {
    fn default() -> Self {
        BlockChainFamily::Ethereum
    }
}

impl FromStr for BlockChainFamily {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ethereum" => Ok(BlockChainFamily::Ethereum),
            "polkadot" => Ok(BlockChainFamily::Polkadot),
            _ => Err(anyhow!("Cannot parse {} to BlockChainFamily", s)),
        }
    }
}

impl Display for BlockChainFamily {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            BlockChainFamily::Ethereum => {
                write!(f, "ethereum")
            }
            BlockChainFamily::Polkadot => {
                write!(f, "polkadot")
            }
        }
    }
}

impl ComponentInfo {
    pub fn get_url(&self) -> UrlType {
        format!("https://{}", self.ip)
    }

    pub fn get_host_header(&self, domain: &String) -> String {
        match self.component_type {
            ComponentType::Node => {
                format!("{}.node.mbr.{}", self.id, domain)
            }
            ComponentType::Gateway => {
                format!("{}.gw.mbr.{}", self.id, domain)
            }
        }
    }

    pub fn get_chain_id(&self) -> String {
        format!("{}.{}", self.blockchain, self.network)
    }
}

impl fmt::Display for ComponentInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({:?} {:?} {})", self.zone, self.component_type, self.id,)
    }
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

impl FromStr for ComponentType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "gateway" => Ok(ComponentType::Gateway),
            "node" => Ok(ComponentType::Node),
            _ => Err(anyhow!("Invalid value")),
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default, Eq, Hash)]
pub struct ChainInfo {
    pub chain: BlockChainType,
    pub network: NetworkType,
}

impl ToString for ChainInfo {
    fn to_string(&self) -> String {
        format!("{}.{}", self.chain, self.network)
    }
}

impl ChainInfo {
    pub fn new(chain: BlockChainType, network: NetworkType) -> Self {
        ChainInfo { chain, network }
    }
    pub fn chain_id(&self) -> String {
        self.to_string()
    }
}

impl FromStr for ChainInfo {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let arr = s.split(".").collect::<Vec<&str>>();
        if arr.len() != 2 {
            return Err(Error::msg(format!("Cannot parse {} to ChainInfo", s)));
        }
        Ok(ChainInfo {
            chain: BlockChainType::from_str(arr[0])?,
            network: arr[1].to_string(),
        })
    }
}
