
package org.globalart.astrum.repository.content.store.internal;

import java.util.Arrays;
import java.util.Map;
import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditorSupport;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.event.component.ComponentAttributesEvent;
import org.globalart.astrum.repository.content.event.component.ComponentCreatedEvent;
import org.globalart.astrum.repository.content.event.component.ComponentDeletedEvent;
import org.globalart.astrum.repository.content.event.component.ComponentEvent;
import org.globalart.astrum.repository.content.event.component.ComponentKindEvent;
import org.globalart.astrum.repository.content.event.component.ComponentsPurgedAuditEvent;
import org.globalart.astrum.repository.content.event.component.ComponentPurgedEvent;
import org.globalart.astrum.repository.content.event.component.ComponentUpdatedEvent;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;

/**
 * Repository component auditor.
 *
 * @since 3.27
 */
@org.springframework.stereotype.Component
public class ComponentAuditor
    extends AuditorSupport
    implements EventAware
{
  public static final String DOMAIN = "repository.component";

  public ComponentAuditor() {
    registerType(ComponentCreatedEvent.class, CREATED_TYPE);
    registerType(ComponentDeletedEvent.class, DELETED_TYPE);
    registerType(ComponentPurgedEvent.class, PURGE_TYPE);
    registerType(ComponentsPurgedAuditEvent.class, PURGE_TYPE);
    registerType(ComponentUpdatedEvent.class, UPDATED_TYPE);
    registerType(ComponentKindEvent.class, UPDATED_TYPE + "-kind");
    registerType(ComponentAttributesEvent.class, UPDATED_TYPE + "-attribute");
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final ComponentPurgedEvent event) {
    if (isRecording()) {
      String repositoryName = event.getRepository().map(Repository::getName).orElse("Unknown");

      AuditData data = new AuditData();
      data.setDomain(DOMAIN);
      data.setType(type(event.getClass()));
      data.setContext(repositoryName);

      Map<String, Object> attributes = data.getAttributes();
      attributes.put("repository.name", repositoryName);
      attributes.put("componentIds", Arrays.toString(event.getComponentIds()));

      record(data);
    }
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final ComponentsPurgedAuditEvent event) {
    if (isRecording()) {
      event.getComponents().forEach(component -> {
        String eventType = type(event.getClass());
        String repoName = event.getRepository().map(Repository::getName).orElse("Unknown");
        AuditData data = prepareAuditData(component, eventType, repoName, null);
        record(data);
      });
    }
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final ComponentEvent event) {
    if (isRecording()) {
      Component component = event.getComponent();
      String eventType = type(event.getClass());
      String repoName = event.getRepository().map(Repository::getName).orElse("Unknown");
      ComponentAttributesEvent attributesEvent = null;
      if (event instanceof ComponentAttributesEvent) {
        attributesEvent = (ComponentAttributesEvent) event;
      }

      AuditData data = prepareAuditData(component, eventType, repoName, attributesEvent);

      record(data);
    }
  }

  private static AuditData prepareAuditData(
      final Component component,
      final String eventType,
      final String repoName,
      final ComponentAttributesEvent attributesEvent)
  {
    AuditData data = new AuditData();
    data.setDomain(DOMAIN);
    data.setType(eventType);
    data.setContext(component.name());

    Map<String, Object> attributes = data.getAttributes();
    attributes.put("repository.name", repoName);
    attributes.put("name", component.name());
    attributes.put("kind", component.kind());
    attributes.put("namespace", component.namespace());
    attributes.put("version", component.version());

    if (attributesEvent != null) {
      attributes.put("attribute.change", attributesEvent.getChange());
      attributes.put("attribute.key", attributesEvent.getKey());
      attributesEvent.getValue().ifPresent(v -> attributes.put("attribute.value", v));
    }
    return data;
  }
}
