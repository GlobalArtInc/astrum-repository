
package org.globalart.astrum.internal.jwt.datastore.upgrade;

import java.sql.Connection;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.springframework.stereotype.Component;

import java.util.Optional;

/**
 * Adds user_source column to jwt_session table for existing databases.
 *
 * The user_source column is needed to uniquely identify users across different
 * authentication realms (default, LDAP, SAML, etc.). Username alone is not
 * sufficient as the same username can exist in multiple realms.
 */
@Component
public class JwtSessionAddUserSourceMigrationStep_2_104
    implements DatabaseMigrationStep
{
  private static final String TABLE_NAME = "jwt_session";

  @Override
  public Optional<String> version() {
    return Optional.of("2.104");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    if (tableExists(connection, TABLE_NAME) && !columnExists(connection, TABLE_NAME, "user_source")) {
      if (isPostgresql(connection)) {
        runStatement(connection,
            "ALTER TABLE jwt_session ADD COLUMN user_source VARCHAR(200) NOT NULL DEFAULT 'default'");
        // Remove default after adding the column
        runStatement(connection, "ALTER TABLE jwt_session ALTER COLUMN user_source DROP DEFAULT");
      }
      else if (isH2(connection)) {
        runStatement(connection,
            "ALTER TABLE jwt_session ADD COLUMN user_source VARCHAR(200) NOT NULL DEFAULT 'default'");
        // H2 doesn't support ALTER COLUMN DROP DEFAULT, but that's okay - the default will only apply to existing rows
      }
    }
  }
}
