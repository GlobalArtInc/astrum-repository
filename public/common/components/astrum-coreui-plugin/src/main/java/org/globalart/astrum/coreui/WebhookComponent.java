
package org.globalart.astrum.coreui;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.extdirect.DirectComponent;
import org.globalart.astrum.extdirect.DirectComponentSupport;
import org.globalart.astrum.repository.webhooks.RepositoryWebhook;
import org.globalart.astrum.webhooks.GlobalWebhook;
import org.globalart.astrum.webhooks.WebhookService;
import org.globalart.astrum.webhooks.WebhookType;

import com.codahale.metrics.annotation.ExceptionMetered;
import com.codahale.metrics.annotation.Timed;
import com.softwarementors.extjs.djn.config.annotations.DirectAction;
import com.softwarementors.extjs.djn.config.annotations.DirectMethod;
import org.apache.shiro.authz.annotation.RequiresPermissions;

import java.util.List;
import java.util.stream.Collectors;
import org.springframework.stereotype.Component;

/**
 * Webhook {@link DirectComponent}.
 */
@Component
@DirectAction(action = "coreui_Webhook")
public class WebhookComponent
    extends DirectComponentSupport
{

  private final WebhookService webhookService;

  @Autowired
  public WebhookComponent(final WebhookService webhookService) {
    this.webhookService = webhookService;
  }

  /**
   * Returns all {@link GlobalWebhook} instances.
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresPermissions("astrum:settings:read")
  public List<ReferenceXO> listWithTypeGlobal() {
    return findWebhooksWithType(GlobalWebhook.TYPE);
  }

  /**
   * Returns all {@link RepositoryWebhook} instances.
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresPermissions("astrum:settings:read")
  public List<ReferenceXO> listWithTypeRepository() {
    return findWebhooksWithType(RepositoryWebhook.TYPE);
  }

  private List<ReferenceXO> findWebhooksWithType(final WebhookType type) {
    return webhookService.getWebhooks()
        .stream()
        .filter(webhook -> webhook.getType() == type)
        .map(webhook -> new ReferenceXO(webhook.getName(), webhook.getName()))
        .collect(Collectors.toList()); // NOSONAR
  }
}
