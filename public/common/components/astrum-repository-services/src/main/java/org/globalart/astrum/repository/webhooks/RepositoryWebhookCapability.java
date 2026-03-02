
package org.globalart.astrum.repository.webhooks;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Set;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.common.i18n.I18N;
import org.globalart.astrum.common.i18n.MessageBundle;
import org.globalart.astrum.capability.CapabilitySupport;
import org.globalart.astrum.capability.CapabilityType;
import org.globalart.astrum.capability.Condition;
import org.globalart.astrum.capability.Tag;
import org.globalart.astrum.capability.Taggable;
import org.globalart.astrum.repository.capability.RepositoryConditions;
import org.globalart.astrum.webhooks.WebhookService;
import org.globalart.astrum.webhooks.WebhookSubscription;

import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.capability.CapabilityType.capabilityType;

@Component(RepositoryWebhookCapability.TYPE_ID)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class RepositoryWebhookCapability
    extends CapabilitySupport<RepositoryWebhookCapabilityConfiguration>
    implements Taggable
{
  public static final String TYPE_ID = "webhook.repository";

  public static final CapabilityType TYPE = capabilityType(TYPE_ID);

  interface Messages
      extends MessageBundle
  {
    @DefaultMessage("Webhook: Repository")
    String name();

    @DefaultMessage("Webhook")
    String category();

    @DefaultMessage("Repository")
    String repositoryLabel();

    @DefaultMessage("Repository to discriminate events from")
    String repositoryHelp();

    @DefaultMessage("Event Types")
    String namesLabel();

    @DefaultMessage("Event types which trigger this Webhook")
    String namesHelp();

    @DefaultMessage("URL")
    String urlLabel();

    @DefaultMessage("Send a HTTP POST request to this URL")
    String urlHelp();

    @DefaultMessage("Secret Key")
    String secretLabel();

    @DefaultMessage("Key to use for HMAC payload digest")
    String secretHelp();

    @DefaultMessage("%s")
    String description(String names);
  }

  static final Messages messages = I18N.create(Messages.class);

  private final WebhookService webhookService;

  private final RepositoryConditions repositoryConditions;

  private final List<WebhookSubscription> subscriptions = new ArrayList<>();

  @Autowired
  public RepositoryWebhookCapability(
      final WebhookService webhookService,
      final RepositoryConditions repositoryConditions)
  {
    this.webhookService = checkNotNull(webhookService);
    this.repositoryConditions = checkNotNull(repositoryConditions);
  }

  @Override
  protected RepositoryWebhookCapabilityConfiguration createConfig(final Map<String, String> properties) {
    return new RepositoryWebhookCapabilityConfiguration(properties, secretsService());
  }

  @Override
  protected String renderDescription() {
    return messages.description(String.join(", ", getConfig().names));
  }

  @Override
  public Condition activationCondition() {
    return conditions().logical()
        .and(
            conditions().capabilities().passivateCapabilityDuringUpdate(),
            repositoryConditions.repositoryExists(() -> getConfig().repository));
  }

  @Override
  protected void onActivate(final RepositoryWebhookCapabilityConfiguration config) {
    webhookService.getWebhooks()
        .stream()
        .filter(webhook -> webhook.getType() == RepositoryWebhook.TYPE && config.names.contains(webhook.getName()))
        .forEach(webhook -> subscriptions.add(webhook.subscribe(config)));
  }

  @Override
  protected void onPassivate(final RepositoryWebhookCapabilityConfiguration config) {
    subscriptions.forEach(WebhookSubscription::cancel);
    subscriptions.clear();
  }

  @Override
  public Set<Tag> getTags() {
    return Set.of(
        Tag.categoryTag(messages.category()),
        Tag.repositoryTag(getConfig().repository));
  }

}
