
package org.globalart.astrum.coreui.internal.webhook;

import java.util.List;
import java.util.stream.Collectors;

import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.QueryParam;

import org.globalart.astrum.coreui.ReferenceXO;
import org.globalart.astrum.rest.Resource;
import org.globalart.astrum.webhooks.GlobalWebhook;
import org.globalart.astrum.repository.webhooks.RepositoryWebhook;
import org.globalart.astrum.webhooks.WebhookService;
import org.globalart.astrum.webhooks.WebhookType;

import jakarta.inject.Inject;

import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;

@Component
@Produces(APPLICATION_JSON)
@Path(WebhookInternalResource.RESOURCE_PATH)
public class WebhookInternalResource
    implements Resource
{
  static final String RESOURCE_PATH = "internal/ui/webhooks";

  private final WebhookService webhookService;

  @Inject
  public WebhookInternalResource(final WebhookService webhookService) {
    this.webhookService = checkNotNull(webhookService);
  }

  @GET
  @RequiresAuthentication
  @RequiresPermissions("astrum:settings:read")
  public List<ReferenceXO> getWebhookTypes(@QueryParam("type") final String type) {
    WebhookType webhookType = "repository".equalsIgnoreCase(type)
        ? RepositoryWebhook.TYPE
        : GlobalWebhook.TYPE;

    return webhookService.getWebhooks()
        .stream()
        .filter(webhook -> webhook.getType() == webhookType)
        .map(webhook -> new ReferenceXO(webhook.getName(), webhook.getName()))
        .collect(Collectors.toList());
  }
}
