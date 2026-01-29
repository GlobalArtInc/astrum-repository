
package org.globalart.astrum.repository.internal.blobstore;

import java.util.Map;

import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditorSupport;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.blobstore.api.BlobStoreCreatedEvent;
import org.globalart.astrum.blobstore.api.BlobStoreDeletedEvent;
import org.globalart.astrum.blobstore.api.BlobStoreEvent;
import org.globalart.astrum.blobstore.api.BlobStoreStartedEvent;
import org.globalart.astrum.blobstore.api.BlobStoreStoppedEvent;
import org.globalart.astrum.blobstore.api.BlobStoreUpdatedEvent;
import org.globalart.astrum.common.event.EventAware;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.stereotype.Component;

/**
 * {@link BlobStore} auditor.
 *
 * @since 3.1
 */
@Component
public class BlobStoreAuditor
    extends AuditorSupport
    implements EventAware
{
  public static final String DOMAIN = "blobstore";

  public BlobStoreAuditor() {
    registerType(BlobStoreCreatedEvent.class, CREATED_TYPE);
    registerType(BlobStoreUpdatedEvent.class, UPDATED_TYPE);
    registerType(BlobStoreDeletedEvent.class, DELETED_TYPE);
    registerType(BlobStoreStartedEvent.class, "started");
    registerType(BlobStoreStoppedEvent.class, "stopped");
  }

  private static final String PRE_SIGNED_URL_ENABLED = "preSignedUrlEnabled";

  @Subscribe
  @AllowConcurrentEvents
  public void on(final BlobStoreEvent event) {
    if (isRecording()) {
      BlobStore blobStore = event.getBlobStore();
      BlobStoreConfiguration configuration = blobStore.getBlobStoreConfiguration();

      AuditData data = new AuditData();
      data.setDomain(DOMAIN);
      data.setType(type(event.getClass()));
      data.setContext(configuration.getName());

      Map<String, Object> attributes = data.getAttributes();
      attributes.put("name", configuration.getName());
      attributes.put("type", configuration.getType());

      Boolean preSignedUrlEnabled = detectPreSignedUrlEnabled(configuration);
      if (preSignedUrlEnabled != null) {
        attributes.put(PRE_SIGNED_URL_ENABLED, preSignedUrlEnabled);
      }

      record(data);
    }
  }

  private static Boolean detectPreSignedUrlEnabled(final BlobStoreConfiguration configuration) {
    Map<String, Map<String, Object>> outer = configuration.getAttributes();
    if (outer == null) {
      return null;
    }
    for (Map<String, Object> sub : outer.values()) {
      if (sub != null && sub.containsKey(PRE_SIGNED_URL_ENABLED)) {
        Object value = sub.get(PRE_SIGNED_URL_ENABLED);
        if (value instanceof Boolean bool) {
          return bool;
        }
        if (value != null) {
          return Boolean.parseBoolean(value.toString());
        }
      }
    }
    return null;
  }
}
