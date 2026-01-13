
package org.globalart.astrum.audit.internal.store;

import java.time.ZoneOffset;
import java.util.Date;
import java.util.Map;
import java.util.Set;

import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditDataRecordedEvent;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.common.event.EventHelper;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.autoconfigure.condition.ConditionalOnBooleanProperty;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.audit.AuditorSupport.CREATED_TYPE;
import static org.globalart.astrum.audit.AuditorSupport.DELETED_TYPE;
import static org.globalart.astrum.audit.AuditorSupport.UPDATED_TYPE;
import static org.globalart.astrum.common.app.FeatureFlags.PREVIEW_UI_AUDIT_ENABLED;

/**
 * Persists audit events to the database. For policed domains only listed types are persisted;
 * everything else (e.g. task lifecycle noise) is skipped. Webhooks and file appenders are unaffected.
 */
@Component
@ConditionalOnBooleanProperty(name = PREVIEW_UI_AUDIT_ENABLED, matchIfMissing = true)
public class AuditEventPersistingSubscriber
    implements EventAware
{
  private static final Logger log = LoggerFactory.getLogger(AuditEventPersistingSubscriber.class);

  private static final Map<String, Set<String>> PERSISTED_TYPES_BY_DOMAIN = Map.of(
      // Everything else TaskAuditor emits (started, running, finished, blocked, ...) is
      // per-run lifecycle noise and would flood the audit table.
      "tasks", Set.of("scheduled", "deleted"),
      // Repository events that are either user initiated or may be of relevance
      "repository", Set.of(CREATED_TYPE, UPDATED_TYPE, DELETED_TYPE, "autoBlockStatus", "cacheInvalidated"),
      // Blobstore Create-Update-Delete
      "blobstore", Set.of(CREATED_TYPE, UPDATED_TYPE, DELETED_TYPE));

  private final AuditEventStore auditEventStore;

  @Autowired
  public AuditEventPersistingSubscriber(final AuditEventStore auditEventStore) {
    this.auditEventStore = auditEventStore;
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final AuditDataRecordedEvent event) {
    if (EventHelper.isReplicating()) {
      return;
    }

    AuditData data = event.getData();

    if (isFiltered(data)) {
      return;
    }

    try {
      AuditEventData entity = new AuditEventData();
      entity.setDomain(data.getDomain());
      entity.setType(data.getType());
      entity.setContext(data.getContext());
      Date ts = data.getTimestamp();
      if (ts != null) {
        entity.setTimestamp(ts.toInstant().atOffset(ZoneOffset.UTC));
      }
      entity.setInitiator(data.getInitiator());
      entity.setNodeId(data.getNodeId());
      entity.setAttributes(data.getAttributes());
      auditEventStore.insert(entity);
    }
    catch (Exception e) {
      log.warn("Failed to persist audit event", e);
    }
  }

  private boolean isFiltered(final AuditData data) {
    Set<String> allowedTypes = PERSISTED_TYPES_BY_DOMAIN.get(data.getDomain());
    if (allowedTypes == null) {
      return false;
    }
    return data.getType() == null || !allowedTypes.contains(data.getType());
  }
}
