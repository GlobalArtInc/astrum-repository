
package org.globalart.astrum.upgrade.datastore.internal;

import org.flywaydb.core.api.MigrationInfo;
import org.flywaydb.core.api.callback.BaseCallback;
import org.flywaydb.core.api.callback.Context;
import org.flywaydb.core.api.callback.Event;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * A Flyway callback which trace logs the actions being performed.
 *
 * @since 3.29
 */
public class TraceLoggingCallback
    extends BaseCallback
{
  private static final Logger log = LoggerFactory.getLogger(TraceLoggingCallback.class);

  @Override
  public void handle(final Event event, final Context context) {
    // Strict NULL checking as otherwise startup/migration can break.
    String eventID = event != null ? event.getId() : "N/A";
    MigrationInfo migrationInfo = context != null ? context.getMigrationInfo() : null;
    String description = migrationInfo != null ? migrationInfo.getDescription() : "N/A";
    String state = migrationInfo != null ? migrationInfo.getState().toString() : "N/A";
    log.trace("{} Migration Description: \"{}\" State: \"{}\"", eventID, description, state);
  }
}
