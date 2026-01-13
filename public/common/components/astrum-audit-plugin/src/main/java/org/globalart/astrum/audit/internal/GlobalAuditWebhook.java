
package org.globalart.astrum.audit.internal;

import java.util.Map;
import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditDataRecordedEvent;
import org.globalart.astrum.audit.internal.GlobalAuditWebhook.AuditWebhookPayload.Audit;
import org.globalart.astrum.webhooks.GlobalWebhook;
import org.globalart.astrum.webhooks.WebhookPayload;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.stereotype.Component;

/**
 * Global audit {@link Webhook}.
 *
 * @since 3.1
 */
@Component
public class GlobalAuditWebhook
    extends GlobalWebhook
{
  public static final String NAME = "audit";

  @Override
  public String getName() {
    return NAME;
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final AuditDataRecordedEvent event) {

    AuditData auditData = event.getData();
    AuditWebhookPayload payload = new AuditWebhookPayload();
    payload.setInitiator(auditData.getInitiator());
    payload.setNodeId(auditData.getNodeId());

    Audit audit = new Audit(auditData.getDomain(), auditData.getType(),
        auditData.getContext(), auditData.getAttributes());
    payload.setAudit(audit);

    getSubscriptions().forEach(s -> queue(s, payload));
  }

  public static class AuditWebhookPayload
      extends WebhookPayload
  {
    public Audit getAudit() {
      return audit;
    }

    public void setAudit(Audit audit) {
      this.audit = audit;
    }

    private Audit audit;

    public static class Audit
    {
      private String domain;

      private String type;

      private String context;

      private Map<String, Object> attributes;

      public Audit(final String domain, final String type, final String context, final Map<String, Object> attributes) {
        this.domain = domain;
        this.type = type;
        this.context = context;
        this.attributes = attributes;
      }

      public String getDomain() {
        return domain;
      }

      public void setDomain(String domain) {
        this.domain = domain;
      }

      public String getType() {
        return type;
      }

      public void setType(String type) {
        this.type = type;
      }

      public String getContext() {
        return context;
      }

      public void setContext(String context) {
        this.context = context;
      }

      public Map<String, Object> getAttributes() {
        return attributes;
      }

      public void setAttributes(Map<String, Object> attributes) {
        this.attributes = attributes;
      }
    }
  }
}
