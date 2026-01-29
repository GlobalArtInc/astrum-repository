
package org.globalart.astrum.repository.routing;

import java.util.Map;
import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditorSupport;
import org.globalart.astrum.common.event.EventAware;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.stereotype.Component;

/**
 * {@link RoutingRule} auditor.
 *
 */
@Component
public class RoutingRuleAuditor
    extends AuditorSupport
    implements EventAware
{
  public static final String DOMAIN = "RoutingRule";

  public RoutingRuleAuditor() {
    registerType(RoutingRuleCreatedEvent.class, CREATED_TYPE);
    registerType(RoutingRuleUpdatedEvent.class, UPDATED_TYPE);
    registerType(RoutingRuleDeletedEvent.class, DELETED_TYPE);
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final RoutingRuleEvent event) {
    if (isRecording()) {
      RoutingRule routingRule = event.getRoutingRule();

      AuditData data = getRoutingRuleAuditData(event, routingRule);
      record(data);
    }
  }

  private AuditData getRoutingRuleAuditData(final RoutingRuleEvent event, final RoutingRule routingRule) {
    AuditData data = new AuditData();
    data.setDomain(DOMAIN);
    data.setType(type(event.getClass()));
    data.setContext(routingRule.name());

    Map<String, Object> attributes = data.getAttributes();
    attributes.put("description", routingRule.description());
    attributes.put("mode", routingRule.mode());
    attributes.put("matchers", routingRule.matchers());
    return data;
  }
}
