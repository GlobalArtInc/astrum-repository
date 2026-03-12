
package org.globalart.astrum.upgrade.datastore.internal.audit;

import java.util.Map;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditorSupport;
import org.globalart.astrum.common.app.ApplicationVersion;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.common.event.EventHelper;
import org.globalart.astrum.common.upgrade.events.UpgradeCompletedEvent;
import org.globalart.astrum.common.upgrade.events.UpgradeEventSupport;
import org.globalart.astrum.common.upgrade.events.UpgradeFailedEvent;
import org.globalart.astrum.common.upgrade.events.UpgradeStartedEvent;

import com.google.common.eventbus.Subscribe;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Captures events for audit logging database migrations.
 */
@Component
public class UpgradeAuditor
    extends AuditorSupport
    implements EventAware, EventAware.Asynchronous
{
  public static final String DOMAIN = "database-migration";

  static final String COMPLETED = "completed";

  static final String FAILED = "failed";

  static final String STARTED = "started";

  static final String MIGRATIONS = "migrations";

  static final String NODE_IDS = "nodeIds";

  static final String MESSAGE = "message";

  static final String SCHEMA_VERSION = "schemaVersion";

  static final String ASTRUM_VERSION = "astrumVersion";

  private final String astrumVersion;

  @Autowired
  public UpgradeAuditor(final ApplicationVersion applicationVersion) {
    astrumVersion = checkNotNull(applicationVersion).getVersion();

    registerType(UpgradeCompletedEvent.class, COMPLETED);
    registerType(UpgradeFailedEvent.class, FAILED);
    registerType(UpgradeStartedEvent.class, STARTED);
  }

  /*
   * Not concurrent as we want the events to be handled in order
   */
  @Subscribe
  public void on(final UpgradeEventSupport event) {
    if (isRecording() && !EventHelper.isReplicating()) {
      AuditData data = new AuditData();
      data.setDomain(DOMAIN);
      data.setType(type(event.getClass()));
      data.setContext(SYSTEM_CONTEXT);
      data.setInitiator(event.getUser().orElse(SYSTEM_CONTEXT));

      Map<String, Object> attributes = data.getAttributes();
      attributes.put(ASTRUM_VERSION, astrumVersion);
      event.getSchemaVersion()
          .ifPresent(schemaVersion -> attributes.put(SCHEMA_VERSION, schemaVersion));

      if (event instanceof UpgradeFailedEvent) {
        attributes.put(MESSAGE, ((UpgradeFailedEvent) event).getErrorMessage());
      }
      else {
        attributes.put(MIGRATIONS, event.getMigrations());
      }

      if (event instanceof UpgradeCompletedEvent) {
        attributes.put(NODE_IDS, ((UpgradeCompletedEvent) event).getNodeIds());
      }

      record(data);
    }
  }
}
