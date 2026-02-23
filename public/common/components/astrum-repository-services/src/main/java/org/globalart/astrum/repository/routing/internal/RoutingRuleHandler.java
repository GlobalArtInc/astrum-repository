
package org.globalart.astrum.repository.routing.internal;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.routing.RoutingRuleHelper;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * A handler which validates that the request is allowed by the RoutingRule assigned to the repository.
 *
 * @since 3.16
 */
@Component
public class RoutingRuleHandler
    implements Handler, org.globalart.astrum.repository.routing.RoutingRuleHandler
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  static final String PATH_IS_BLOCKED = "Routing rules block the requested item from this repository";

  private final RoutingRuleHelper routingRuleHelper;

  @Autowired
  public RoutingRuleHandler(final RoutingRuleHelper routingRuleHelper) {
    this.routingRuleHelper = checkNotNull(routingRuleHelper);
  }

  @Override
  public Response handle(final Context context) throws Exception {
    Repository repository = context.getRepository();
    String path = path(context.getRequest());

    boolean isRoutingRuleAllowed = routingRuleHelper.isAllowed(repository, path);

    if (!isRoutingRuleAllowed) {
      String repositoryType = repository.getType().getValue();
      String repositoryName = repository.getName();

      log.debug("Routing rules block the requested item for Repository{name='{}', type={}}",
          repositoryName,
          repositoryType);

      return HttpResponses.forbidden(PATH_IS_BLOCKED);
    }

    return context.proceed();
  }

  private String path(final Request request) {
    if (request.getParameters().size() == 0) {
      return request.getPath();
    }
    StringBuilder sb = new StringBuilder();
    request.getParameters()
        .forEach(entry -> sb.append('&').append(entry.getKey()).append('=').append(entry.getValue()));
    sb.replace(0, 1, "?");
    return request.getPath() + sb;
  }
}
