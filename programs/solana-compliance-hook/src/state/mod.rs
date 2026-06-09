pub mod compliance_config;
pub mod compliance_list;
pub mod compliance_mode; // Expose the new module tree file

// 🗲 Rule: Flatten the surface area. Handlers shouldn't care how the state directory is partitioned.
pub use compliance_config::ComplianceConfig;
pub use compliance_list::{ComplianceList, ListType};
pub use compliance_mode::ComplianceMode; // Bubble up the enum directly