
package org.globalart.astrum.internal.capability;

import java.util.HashMap;
import java.util.Map;
import java.util.Map.Entry;

import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditorSupport;
import org.globalart.astrum.capability.Capability;
import org.globalart.astrum.capability.CapabilityContext;
import org.globalart.astrum.capability.CapabilityDescriptor;
import org.globalart.astrum.capability.CapabilityEvent;
import org.globalart.astrum.capability.CapabilityReference;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.formfields.Encrypted;
import org.globalart.astrum.formfields.FormField;
import org.globalart.astrum.security.UserIdHelper;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.stereotype.Component;

/**
 * {@link Capability} auditor.
 *
 * @since 3.1
 */
@Component
public class CapabilityAuditor
    extends AuditorSupport
    implements EventAware
{
  public static final String DOMAIN = "capability";

  public CapabilityAuditor() {
    registerType(CapabilityEvent.Created.class, CREATED_TYPE);
    registerType(CapabilityEvent.AfterActivated.class, "activated");
    registerType(CapabilityEvent.BeforePassivated.class, "passivated");
    registerType(CapabilityEvent.AfterRemove.class, DELETED_TYPE);
    registerType(CapabilityEvent.AfterUpdate.class, UPDATED_TYPE);
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final CapabilityEvent event) {
    // Skip system-initiated events (startup/shutdown → *UNKNOWN; privileged
    // background execution → *SYSTEM). Only user-initiated changes belong in
    // the audit log per ASTRUM-53769.
    if (isRecording() && !UserIdHelper.isUnknown() && !UserIdHelper.isSystem()) {
      CapabilityReference reference = event.getReference();
      CapabilityContext context = reference.context();
      CapabilityDescriptor descriptor = context.descriptor();

      AuditData data = new AuditData();
      data.setDomain(DOMAIN);
      data.setType(type(event.getClass()));
      data.setContext(context.type().toString());

      Map<String, Object> attributes = data.getAttributes();
      attributes.put("id", context.id().toString());
      attributes.put("type", context.type().toString());
      attributes.put("enabled", string(context.isEnabled()));
      attributes.put("active", string(context.isActive()));
      attributes.put("failed", string(context.hasFailure()));

      // include all non-secure properties
      Map<String, FormField> fields = fields(descriptor);
      for (Entry<String, String> entry : context.properties().entrySet()) {
        FormField field = fields.get(entry.getKey());
        // skip secure fields
        if (field instanceof Encrypted) {
          continue;
        }
        attributes.put("property." + entry.getKey(), entry.getValue());
      }

      record(data);
    }
  }

  private static Map<String, FormField> fields(final CapabilityDescriptor descriptor) {
    Map<String, FormField> result = new HashMap<>();
    for (FormField field : descriptor.formFields()) {
      result.put(field.getId(), field);
    }
    return result;
  }
}
