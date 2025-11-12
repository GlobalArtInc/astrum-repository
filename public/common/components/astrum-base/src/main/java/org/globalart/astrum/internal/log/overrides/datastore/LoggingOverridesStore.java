
package org.globalart.astrum.internal.log.overrides.datastore;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.datastore.ConfigStoreSupport;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.transaction.Transactional;

import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;

import static org.globalart.astrum.common.app.FeatureFlags.DATASTORE_CLUSTERED_ENABLED;
import static org.globalart.astrum.internal.log.overrides.datastore.LoggerOverridesEvent.Action.CHANGE;
import static org.globalart.astrum.internal.log.overrides.datastore.LoggerOverridesEvent.Action.RESET;
import static org.globalart.astrum.internal.log.overrides.datastore.LoggerOverridesEvent.Action.RESET_ALL;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Store for accessing the logging-overrides related data
 */
@Component
@Qualifier("mybatis")
@ConditionalOnProperty(name = DATASTORE_CLUSTERED_ENABLED, havingValue = "true")
public class LoggingOverridesStore
    extends ConfigStoreSupport<LoggingOverridesDAO>
{
  @Autowired
  protected LoggingOverridesStore(final DataSessionSupplier sessionSupplier) {
    super(sessionSupplier);
  }

  @Transactional
  public void create(final LoggingOverridesData data) {
    dao().createRecord(data);
    postCommitEvent(() -> new LoggerOverridesEvent(data.getName(), data.getLevel(), CHANGE));
  }

  @Transactional
  public Continuation<LoggingOverridesData> readRecords() {
    return dao().readRecords(null);
  }

  @Transactional
  public boolean exists(final String name) {
    return readRecords().stream()
        .anyMatch(level -> level.getName().equals(name));
  }

  @Transactional
  public void update(final LoggingOverridesData data) {
    dao().updateRecord(data);
    postCommitEvent(() -> new LoggerOverridesEvent(data.getName(), data.getLevel(), CHANGE));
  }

  @Transactional
  public void deleteByName(final String name) {
    dao().deleteRecord(name);
    postCommitEvent(() -> new LoggerOverridesEvent(name, null, RESET));
  }

  /**
   * Delete all records
   */
  @Transactional
  public void deleteAllRecords() {
    dao().deleteAllRecords();
    postCommitEvent(() -> new LoggerOverridesEvent(null, null, RESET_ALL));
  }
}
