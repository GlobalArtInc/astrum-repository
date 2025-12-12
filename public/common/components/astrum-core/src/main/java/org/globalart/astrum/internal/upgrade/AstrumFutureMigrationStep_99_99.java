
package org.globalart.astrum.internal.upgrade;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static org.globalart.astrum.common.app.FeatureFlags.ZERO_DOWNTIME_FUTURE_MIGRATION_ENABLED;

@Component
@ConditionalOnProperty(name = ZERO_DOWNTIME_FUTURE_MIGRATION_ENABLED, havingValue = "true")
public class AstrumFutureMigrationStep_99_99
    implements DatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String FAIL_MIGRATION_FLAG = "astrum.zdu.baseline.fail";

  private final boolean shouldFail;

  @Autowired
  public AstrumFutureMigrationStep_99_99(
      @Value("${" + FAIL_MIGRATION_FLAG + ":false}") final boolean shouldFail)
  {
    this.shouldFail = shouldFail;
  }

  @Override
  public Optional<String> version() {
    return Optional.of("99.99");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    log.warn("Started step 99.99 test");
    if (shouldFail) {
      if (log.isDebugEnabled()) {
        log.warn("simulating migration failure due to feature flag '{}'", FAIL_MIGRATION_FLAG);
      }
      throw new IllegalStateException("Unable to migrate");
    }
  }
}
