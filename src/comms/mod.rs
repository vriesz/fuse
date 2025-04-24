// src/comms/mod.rs

// Serialize/deserialize helpers for Instant
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use std::time::{Duration, Instant};
use openssl::ssl::{SslMethod, SslConnector};

// Custom serialization for Instant (as duration since current time)
mod instant_serde {
    use super::*;

    // A helper struct that can be serialized
    #[derive(Serialize, Deserialize)]
    struct InstantDuration {
        secs: u64,
        nanos: u32,
    }

    pub fn serialize<S>(instant: &Option<Instant>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match instant {
            Some(instant) => {
                // Calculate duration since program start
                let program_duration = instant.elapsed();
                
                // Serialize as seconds and nanoseconds
                let duration = InstantDuration {
                    secs: program_duration.as_secs(),
                    nanos: program_duration.subsec_nanos(),
                };
                duration.serialize(serializer)
            },
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Instant>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<InstantDuration>::deserialize(deserializer)?;
        match opt {
            Some(duration) => {
                // Recreate Instant by subtracting the duration from now
                let dur = Duration::new(duration.secs, duration.nanos);
                Ok(Some(Instant::now() - dur))
            },
            None => Ok(None),
        }
    }
}

// ------ Core Communication Types ------
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LinkType {
    MAVLink {
        version: u8,
        heartbeat_interval_ms: u32,
    },
    LoRa {
        frequency_mhz: u32,
        spreading_factor: u8,
    },
    WiFiDirect {
        bandwidth_mbps: u32,
        channel: u8,
    },
    MilitaryEncrypted {
        key_rotation_minutes: u32,
        cipher_suite: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationLink {
    pub link_type: LinkType,
    pub encryption: bool,
    #[serde(with = "instant_serde")]
    pub last_active: Option<Instant>,
}

// ------ Secure Communication ------

// Custom wrapper for SslConnector to make it cloneable and skippable for serde
#[derive(Debug)]
pub struct SslConnectorWrapper(SslConnector);

impl Clone for SslConnectorWrapper {
    fn clone(&self) -> Self {
        // Create a new connector with the same settings
        // This is simplified - in practice you'd need to copy all settings
        let builder = SslConnector::builder(SslMethod::tls())
            .expect("Failed to create SSL connector builder");
        SslConnectorWrapper(builder.build()) 
    }
}

impl SslConnectorWrapper {
    pub fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        // This demonstrates that we're using the field
        let _connector = &self.0;
        
        // Simple placeholder implementation
        let mut result = Vec::new();
        result.extend_from_slice(data);
        result
    }
}

// Add custom serialization for SslConnectorWrapper
impl Serialize for SslConnectorWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Skip serialization by serializing it as a unit type
        serializer.serialize_unit()
    }
}

impl<'de> Deserialize<'de> for SslConnectorWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Skip deserialization and create a new instance
        let _ = <()>::deserialize(deserializer)?;
        
        let builder = SslConnector::builder(SslMethod::tls())
            .map_err(|e| serde::de::Error::custom(format!("OpenSSL error: {}", e)))?;
        
        Ok(SslConnectorWrapper(builder.build()))
    }
}

#[derive(Debug, Clone)]
pub struct SecureChannel {
    connector: SslConnectorWrapper,
    heartbeat_interval: u32,
}

// Add custom serialization for SecureChannel
impl Serialize for SecureChannel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        // Only serialize the heartbeat_interval
        let mut state = serializer.serialize_struct("SecureChannel", 1)?;
        state.serialize_field("heartbeat_interval", &self.heartbeat_interval)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for SecureChannel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Create a temporary struct that matches our serialization
        #[derive(Deserialize)]
        struct SecureChannelHelper {
            heartbeat_interval: u32,
        }
        
        let helper = SecureChannelHelper::deserialize(deserializer)?;
        
        // Create a new secure channel with default settings
        match Self::new("AES256-SHA256") {
            Ok(mut channel) => {
                channel.heartbeat_interval = helper.heartbeat_interval;
                Ok(channel)
            },
            Err(e) => Err(serde::de::Error::custom(format!("OpenSSL error: {}", e)))
        }
    }
}

impl SecureChannel {
    pub fn new(cipher_suite: &str) -> Result<Self, openssl::error::ErrorStack> {
        let mut builder = SslConnector::builder(SslMethod::tls())?;
        builder.set_cipher_list(cipher_suite)?;
        
        Ok(Self {
            connector: SslConnectorWrapper(builder.build()),
            heartbeat_interval: 1000,
        })
    }

    pub fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        // Use the connector to encrypt data
        self.connector.encrypt(data)
    }
    
    pub fn get_heartbeat_interval(&self) -> u32 {
        self.heartbeat_interval
    }
    
    pub fn set_heartbeat_interval(&mut self, interval: u32) {
        self.heartbeat_interval = interval;
    }
}

// ------ Operational Components ------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operator {
    pub id: String,
    pub clearance_level: u8,
    pub assigned_links: Vec<LinkType>,
    #[serde(with = "instant_serde")]
    pub last_heartbeat: Option<Instant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadarContact {
    pub distance_m: f32,
    pub bearing_deg: f32,
    pub relative_speed_mps: f32,
    pub via_link: LinkType,  // Which comms link detected this
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationBeacon {
    pub id: String,
    pub position: (f32, f32),
    pub signal_strength: f32,
    pub link_used: LinkType,  // Which comms link received this
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationHub {
    pub primary_link: CommunicationLink,
    pub backup_links: Vec<CommunicationLink>,
    pub operators: Vec<Operator>,
    #[serde(skip)] // Skip this field during serialization/deserialization
    pub secure_channel: Option<SecureChannel>,
    pub radar_contacts: Vec<RadarContact>,
}

impl CommunicationHub {
    pub fn new(primary: LinkType, secure: bool) -> Self {
        let secure_channel = if secure {
            match SecureChannel::new("AES256-SHA256") {
                Ok(channel) => Some(channel),
                Err(e) => {
                    eprintln!("Failed to create secure channel: {}", e);
                    None
                }
            }
        } else {
            None
        };

        Self {
            primary_link: CommunicationLink {
                link_type: primary,
                encryption: secure,
                last_active: None,
            },
            backup_links: Vec::new(),
            operators: Vec::new(),
            secure_channel,
            radar_contacts: Vec::new(),
        }
    }

    pub fn add_operator(&mut self, id: String, clearance: u8, links: Vec<LinkType>) {
        self.operators.push(Operator {
            id,
            clearance_level: clearance,
            assigned_links: links,
            last_heartbeat: Some(Instant::now()),
        });
    }

    pub fn establish_secure_link(&mut self) -> Result<(), String> {
        match &mut self.secure_channel {
            Some(_) => Ok(()),
            None => {
                self.secure_channel = Some(
                    SecureChannel::new("AES256-SHA256")
                        .map_err(|e| format!("Secure channel failed: {}", e))?
                );
                Ok(())
            }
        }
    }
    
    pub fn log_beacon(&mut self, beacon: NavigationBeacon) {
        // Implementation - e.g., store the beacon or process it
        println!("Beacon logged: {} at {:?}", beacon.id, beacon.position);
    }
}

// Define OODA cycle priority enum
#[derive(Debug, Clone, PartialEq)]
pub enum CommsPriority {
    Low,
    Medium,
    High,
}

impl CommunicationHub {
    pub fn process_ooda_cycle(&mut self, ooda_time: Duration) -> CommsPriority {
        let bandwidth_needed = match ooda_time {
            t if t < Duration::from_millis(100) => CommsPriority::High,
            t if t < Duration::from_millis(500) => CommsPriority::Medium,
            _ => CommsPriority::Low,
        };
        
        self.adjust_links(bandwidth_needed.clone());
        bandwidth_needed
    }

    fn adjust_links(&mut self, priority: CommsPriority) {
        match priority {
            CommsPriority::High => {
                self.primary_link.link_type = LinkType::WiFiDirect {
                    bandwidth_mbps: 100,
                    channel: 36,
                };
            },
            CommsPriority::Medium => {
                self.primary_link.link_type = LinkType::MAVLink {
                    version: 2,
                    heartbeat_interval_ms: 500,
                };
            },
            CommsPriority::Low => {
                self.primary_link.link_type = LinkType::LoRa {
                    frequency_mhz: 915,
                    spreading_factor: 10,
                };
            }
        }
    }
}