
package org.globalart.astrum.repository.content.browse.capability;

import java.util.concurrent.atomic.AtomicBoolean;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.common.db.DatabaseCheck;

import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
public class BrowseTrimService
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final AtomicBoolean postgresqlTrimEnabled = new AtomicBoolean(false);

  private final AtomicBoolean batchTrimEnabled = new AtomicBoolean(false);

  private final DatabaseCheck databaseCheck;

  @Autowired
  public BrowseTrimService(final DatabaseCheck databaseCheck) {
    this.databaseCheck = checkNotNull(databaseCheck);
  }

  public boolean isPostgresqlTrimEnabled() {
    return postgresqlTrimEnabled.get();
  }

  public void setPostgresqlTrimEnabled(final boolean enabled) {
    log.info("PostgreSQL browse trim setting changed to: {}", enabled);
    postgresqlTrimEnabled.set(enabled);
  }

  public boolean isBatchTrimEnabled() {
    return batchTrimEnabled.get();
  }

  public void setBatchTrimEnabled(final boolean enabled) {
    log.info("Batch trim processing setting changed to: {}", enabled);
    batchTrimEnabled.set(enabled);
  }

  /**
   * Determines if trim operations should be allowed based on database type and capability settings.
   *
   * @return true if trim operations should proceed, false otherwise
   */
  public boolean shouldAllowTrim() {
    if (databaseCheck.isPostgresql() && !postgresqlTrimEnabled.get()) {
      log.debug("Trim operations disabled for PostgreSQL (capability not enabled)");
      return false;
    }
    return true;
  }
}
