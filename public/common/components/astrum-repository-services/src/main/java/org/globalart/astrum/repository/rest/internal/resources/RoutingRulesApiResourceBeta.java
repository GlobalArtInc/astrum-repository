
package org.globalart.astrum.repository.rest.internal.resources;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.repository.routing.RoutingRuleHelper;
import org.globalart.astrum.repository.routing.RoutingRuleStore;

import io.swagger.v3.oas.annotations.Hidden;

import static org.globalart.astrum.repository.rest.internal.resources.RoutingRulesApiResourceBeta.RESOURCE_URI;
import static org.globalart.astrum.rest.APIConstants.BETA_API_PREFIX;
import org.springframework.stereotype.Component;

@Hidden
@Component
@Path(RESOURCE_URI)
@Deprecated
public class RoutingRulesApiResourceBeta
    extends RoutingRulesApiResource
{
  static final String RESOURCE_URI = BETA_API_PREFIX + "/routing-rules";

  @Autowired
  public RoutingRulesApiResourceBeta(
      final RoutingRuleStore routingRuleStore,
      final RoutingRuleHelper routingRuleHelper)
  {
    super(routingRuleStore, routingRuleHelper);
  }
}
