#[derive(Debug, Clone)]
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
        InfMetadata {
            published_name: "".to_string(),
            original_name: "".to_string(),
            provider_name: "".to_string(),
            class_name: "".to_string(),
            class_guid: "".to_string(),
            driver_version: "".to_string(),
            signer_name: "".to_string(),
            instance_id: "".to_string(),
            device_description: "".to_string(),
            extension_id: "".to_string(),
            parent: "".to_string(),
            children: Vec::new(),
            extension_driver_names: Vec::new(),
        }
    }
}