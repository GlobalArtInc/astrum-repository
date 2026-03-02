
package org.globalart.astrum.repository.webhooks;

import java.util.Date;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.audit.InitiatorProvider;
import org.globalart.astrum.common.node.NodeAccess;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.manager.RepositoryCreatedEvent;
import org.globalart.astrum.repository.manager.RepositoryDeletedEvent;
import org.globalart.astrum.repository.manager.RepositoryUpdatedEvent;
import org.globalart.astrum.webhooks.GlobalWebhook;
import org.globalart.astrum.webhooks.Webhook;
import org.globalart.astrum.webhooks.WebhookPayload;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Global repository {@link Webhook}.
 *
 * @since 3.1
 */
@Component
public class GlobalRepositoryWebhook
    extends GlobalWebhook
{
  public static final String NAME = "repository";

  private final NodeAccess nodeAccess;

  private final InitiatorProvider initiatorProvider;

  @Autowired
  public GlobalRepositoryWebhook(final NodeAccess nodeAccess, final InitiatorProvider initiatorProvider) {
    this.nodeAccess = checkNotNull(nodeAccess);
    this.initiatorProvider = checkNotNull(initiatorProvider);
  }

  @Override
  public String getName() {
    return NAME;
  }

  @Subscribe
  @AllowConcurrentEvents
  void on(final RepositoryCreatedEvent event) {
    queue(event.getRepository(), EventAction.CREATED);
  }

  @Subscribe
  @AllowConcurrentEvents
  void on(final RepositoryUpdatedEvent event) {
    queue(event.getRepository(), EventAction.UPDATED);
  }

  @Subscribe
  @AllowConcurrentEvents
  void on(final RepositoryDeletedEvent event) {
    queue(event.getRepository(), EventAction.DELETED);
  }

  private void queue(final Repository repository, final EventAction eventAction) {
    RepositoryWebhookPayload.RepositoryPayload repositoryPayload =
        new RepositoryWebhookPayload.RepositoryPayload(repository.getName(), repository.getType(),
            repository.getFormat());

    RepositoryWebhookPayload payload = new RepositoryWebhookPayload(eventAction, repositoryPayload, nodeAccess.getId(),
        new Date(), initiatorProvider.get());

    getSubscriptions().forEach(subscription -> queue(subscription, payload));
  }

  public enum EventAction
  {
    CREATED,
    UPDATED,
    DELETED
  }

  public static class RepositoryWebhookPayload
      extends WebhookPayload
  {
    private final EventAction action;

    private final RepositoryPayload repository;

    public RepositoryWebhookPayload(
        final EventAction action,
        final RepositoryPayload repository,
        final String nodeId,
        final Date timestamp,
        final String initiator)
    {
      this.action = checkNotNull(action);
      this.repository = checkNotNull(repository);
      setNodeId(nodeId);
      setTimestamp(timestamp);
      setInitiator(initiator);
    }

    public EventAction getAction() {
      return this.action;
    }

    public RepositoryPayload getRepository() {
      return this.repository;
    }

    public static class RepositoryPayload
    {
      private final String format;

      private final String name;

      private final String type;

      public RepositoryPayload(final String name, final Type type, final Format format) {
        this.name = checkNotNull(name);
        this.type = checkNotNull(type).getValue();
        this.format = checkNotNull(format).getValue();
      }

      public String getFormat() {
        return this.format;
      }

      public String getName() {
        return this.name;
      }

      public String getType() {
        return this.type;
      }
    }
  }
}
