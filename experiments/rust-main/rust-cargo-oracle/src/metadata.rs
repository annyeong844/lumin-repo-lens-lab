mod model;
mod selection;

pub(crate) use model::{CargoMetadata, CargoPackage};
pub(crate) use selection::{package_root, packages_by_name_or_id, selected_packages};
