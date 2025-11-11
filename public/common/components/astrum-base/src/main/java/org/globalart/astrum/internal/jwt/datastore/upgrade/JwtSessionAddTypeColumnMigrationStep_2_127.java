
package org.globalart.astrum.internal.jwt.datastore.upgrade;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.springframework.stereotype.Component;

/**
 * Adds the {@code type} discriminator column to the {@code jwt_session} table.
 *
 * The column splits rows into two kinds: {@code SESSION} (per-session revocations from logout)
 * and {@code USER_INVALIDATION} (per-user cutoffs recorded on password change, consulted by
 * {@code JwtSecurityFilter} against each JWT's {@code iat}). See ASTRUM-52579.
 */
@Component
public class JwtSessionAddTypeColumnMigrationStep_2_127
    implements DatabaseMigrationStep
{
  private static final String TABLE_NAME = "jwt_session";

  private static final String COLUMN_NAME = "type";

  @Override
  public Optional<String> version() {
    return Optional.of("2.127");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    if (!tableExists(connection, TABLE_NAME)) {
      return;
    }

    if (isPostgresql(connection)) {
      runStatement(connection, """
          DO $$
          BEGIN
            BEGIN
              ALTER TABLE jwt_session ADD COLUMN type VARCHAR(32) NOT NULL DEFAULT 'SESSION';
            EXCEPTION
              WHEN duplicate_column THEN NULL;
            END;
          END $$;
          """);
    }
    else if (isH2(connection) && !columnExists(connection, TABLE_NAME, COLUMN_NAME)) {
      runStatement(connection,
          "ALTER TABLE jwt_session ADD COLUMN type VARCHAR(32) NOT NULL DEFAULT 'SESSION'");
    }
  }
}
