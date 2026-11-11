use sea_orm::{ConnectionTrait, DbBackend, DbErr, Statement};

const GRACE: &str = "1 hour";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Purged {
    pub inventories: u64,
    pub packages: u64,
    pub layer_facts: u64,
}

async fn delete(db: &impl ConnectionTrait, sql: &str) -> Result<u64, DbErr> {
    Ok(db
        .execute(Statement::from_string(DbBackend::Postgres, sql))
        .await?
        .rows_affected())
}

pub async fn run(db: &impl ConnectionTrait) -> Result<Purged, DbErr> {
    let inventories = delete(
        db,
        &format!(
            "DELETE FROM sca_inventory i WHERE i.extracted_at < now() - interval '{GRACE}' \
             AND NOT EXISTS (SELECT 1 FROM component_scan s WHERE s.inventory_id = i.id)"
        ),
    )
    .await?;
    let packages = delete(
        db,
        "DELETE FROM sca_package p WHERE NOT EXISTS \
         (SELECT 1 FROM sca_inventory_package ip WHERE ip.package_id = p.id)",
    )
    .await?;
    let layer_facts = delete(
        db,
        &format!(
            "DELETE FROM oci_layer_facts f WHERE f.extracted_at < now() - interval '{GRACE}' \
             AND NOT EXISTS (SELECT 1 FROM asset a WHERE a.sha256 = split_part(f.digest, ':', 2) \
             AND a.format = 'docker' AND a.kind = 'blob')"
        ),
    )
    .await?;
    Ok(Purged {
        inventories,
        packages,
        layer_facts,
    })
}
