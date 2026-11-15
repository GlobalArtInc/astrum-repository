use astrum_content::inventory::{Inventory, PackageRef};

pub(super) fn writable(package: &PackageRef) -> bool {
    [
        &package.ecosystem,
        &package.name,
        &package.version,
        &package.purl,
    ]
    .iter()
    .all(|text| !text.chars().any(char::is_control))
}

pub fn sanitise(inventory: &mut Inventory) -> usize {
    let before = inventory.packages.len();
    inventory.packages.retain(writable);
    let dropped = before - inventory.packages.len();
    if dropped > 0 {
        inventory.complete = false;
        tracing::warn!(
            dropped,
            key = %inventory.key,
            "packages with control characters were left out of an inventory, which is now incomplete"
        );
    }
    dropped
}
