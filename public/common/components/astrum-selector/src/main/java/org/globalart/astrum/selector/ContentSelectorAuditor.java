
package org.globalart.astrum.selector;

import java.util.Map;
import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditorSupport;
import org.globalart.astrum.common.event.EventAware;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.stereotype.Component;

/**
 * ContentSelector auditor.
 *
 */
@Component
public class ContentSelectorAuditor
    extends AuditorSupport
    implements EventAware
{
  public static final String DOMAIN = "ContentSelector";

  public ContentSelectorAuditor() {
    registerType(ContentSelectorCreatedEvent.class, CREATED_TYPE);
    registerType(ContentSelectorUpdatedEvent.class, UPDATED_TYPE);
    registerType(ContentSelectorDeletedEvent.class, DELETED_TYPE);
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final ContentSelectorEvent event) {
    if (isRecording()) {
      SelectorConfiguration selectorConfiguration = event.getSelectorConfiguration();

      AuditData data = getSelectorAuditData(event, selectorConfiguration);
      record(data);
    }
  }

  private AuditData getSelectorAuditData(
      final ContentSelectorEvent event,
      final SelectorConfiguration selectorConfiguration)
  {
    AuditData data = new AuditData();
    data.setDomain(DOMAIN);
    data.setType(type(event.getClass()));
    data.setContext(selectorConfiguration.getName());

    Map<String, Object> attributes = data.getAttributes();
    attributes.put("description", selectorConfiguration.getDescription());
    attributes.put("type", selectorConfiguration.getType());
    attributes.put("expression", selectorConfiguration.getAttributes());
    return data;
  }
}
