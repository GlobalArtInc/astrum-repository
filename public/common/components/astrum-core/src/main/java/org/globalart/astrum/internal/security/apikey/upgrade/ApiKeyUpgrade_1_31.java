
package org.globalart.astrum.internal.security.apikey.upgrade;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Updates api_key table to add principals to the primary key
 */
@Component
public class ApiKeyUpgrade_1_31
    implements DatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String DROP_CONSTRAINT =
      "ALTER TABLE api_key DROP CONSTRAINT pk_api_key_primaryprincipal_domain;";

  private static final String ADD_CONSTRAINT =
      "ALTER TABLE api_key ADD CONSTRAINT pk_api_key_primaryprincipal_domain_principals PRIMARY KEY "
          + "(primary_principal, domain, principals);";

  @Override
  public Optional<String> version() {
    return Optional.of("1.31");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    boolean tableExists = tableExists(connection, "api_key");
    boolean indexExists = indexExists(connection, "pk_api_key_primaryprincipal_domain");

    log.info("Updating primary key for api_key. table_exists:{} index:{} change_required:{}", tableExists, indexExists,
        tableExists && indexExists);

    if (tableExists && indexExists) {
      runStatement(connection, DROP_CONSTRAINT + ADD_CONSTRAINT);
    }
  }
}
