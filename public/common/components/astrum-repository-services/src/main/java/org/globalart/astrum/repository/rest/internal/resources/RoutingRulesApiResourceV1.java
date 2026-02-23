
package org.globalart.astrum.repository.rest.internal.resources;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.repository.routing.RoutingRuleHelper;
import org.globalart.astrum.repository.routing.RoutingRuleStore;

import static org.globalart.astrum.repository.rest.internal.resources.RoutingRulesApiResourceV1.RESOURCE_URI;
import static org.globalart.astrum.rest.APIConstants.V1_API_PREFIX;
import org.springframework.stereotype.Component;

@Component
@Path(RESOURCE_URI)
public class RoutingRulesApiResourceV1
    extends RoutingRulesApiResource
{
  public static final String RESOURCE_URI = V1_API_PREFIX + "/routing-rules";

  @Autowired
  public RoutingRulesApiResourceV1(
      final RoutingRuleStore routingRuleStore,
      final RoutingRuleHelper routingRuleHelper)
  {
    super(routingRuleStore, routingRuleHelper);
  }
}
