#[derive(Debug, Clone, Default)]
pub struct InfMetadata {
    pub published_name: String,
    pub original_name: String,
    pub provider_name: String,
    pub class_name: String,
    pub class_guid: String,
    pub driver_version: String,
    pub signer_name: String,
    pub instance_id: String,
    pub device_description: String,
    pub extension_id: String,
    pub parent: String,
    pub children: Vec<String>,
    pub extension_driver_names: Vec<String>,
}

impl InfMetadata {
    pub fn new() -> Self {
        Self { ..Default::default() }
    }
}