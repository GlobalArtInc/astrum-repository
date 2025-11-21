
package org.globalart.astrum.datastore;

import java.util.HashMap;
import java.util.Map;

import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditorSupport;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.datastore.api.DataStoreConfiguration;
import org.globalart.astrum.distributed.event.service.api.common.DataStoreConfigurationEvent;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.stereotype.Component;

@Component
public class DataStoreAuditor
    extends AuditorSupport
    implements EventAware
{
  public static final String DOMAIN = "DataStore";

  public DataStoreAuditor() {
    registerType(DataStoreConfigurationEvent.class, UPDATED_TYPE);
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final DataStoreConfigurationEvent event) {
    if (isRecording()) {
      AuditData data = new AuditData();
      data.setDomain(DOMAIN);
      data.setType(type(event.getClass()));
      data.setContext(event.getConfigurationName());

      Map<String, Object> attributes = data.getAttributes();
      attributes.put("type", event.getType());
      attributes.put("source", event.getSource());

      Map<String, String> eventAttributes = new HashMap<>(event.getAttributes());
      eventAttributes.replace("password", DataStoreConfiguration.REDACTED);
      attributes.put("attributes", eventAttributes);

      record(data);
    }
  }
}
