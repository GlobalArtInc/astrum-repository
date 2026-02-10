
package org.globalart.astrum.repository.content.webhooks;

import java.util.Arrays;
import java.util.Date;
import java.util.Optional;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.audit.InitiatorProvider;
import org.globalart.astrum.common.entity.EntityId;
import org.globalart.astrum.common.node.NodeAccess;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.event.asset.AssetCreatedEvent;
import org.globalart.astrum.repository.content.event.asset.AssetDeletedEvent;
import org.globalart.astrum.repository.content.event.asset.AssetEvent;
import org.globalart.astrum.repository.content.event.asset.AssetPurgedEvent;
import org.globalart.astrum.repository.content.event.asset.AssetUpdatedEvent;
import org.globalart.astrum.repository.content.store.InternalIds;
import org.globalart.astrum.repository.rest.api.RepositoryItemIDXO;
import org.globalart.astrum.repository.webhooks.RepositoryWebhook;
import org.globalart.astrum.webhooks.WebhookPayload;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

@Component
public class RepositoryAssetWebhook
    extends RepositoryWebhook
{
  public static final String NAME = "asset";

  private final NodeAccess nodeAccess;

  private final InitiatorProvider initiatorProvider;

  @Autowired
  public RepositoryAssetWebhook(final NodeAccess nodeAccess, final InitiatorProvider initiatorProvider) {
    this.nodeAccess = checkNotNull(nodeAccess);
    this.initiatorProvider = checkNotNull(initiatorProvider);
  }

  @Override
  public String getName() {
    return NAME;
  }

  private enum EventAction
  {
    CREATED,
    UPDATED,
    DELETED,
    PURGED
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final AssetCreatedEvent event) {
    maybeQueue(event, EventAction.CREATED);
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final AssetUpdatedEvent event) {
    maybeQueue(event, EventAction.UPDATED);
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final AssetDeletedEvent event) {
    maybeQueue(event, EventAction.DELETED);
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final AssetPurgedEvent event) {
    maybeQueue(getPayload(event, EventAction.PURGED));
  }

  /**
   * Maybe queue {@link WebhookRequest} for event matching subscriptions.
   */
  private void maybeQueue(final AssetEvent event, final EventAction eventAction) {
    maybeQueue(getPayload(event, eventAction));
  }

  /**
   * Maybe queue {@link WebhookRequest} for event matching subscriptions.
   */
  private void maybeQueue(final RepositoryAssetWebhookPayload payload) {
    subscriptions.forEach(subscription -> {
      RepositoryWebhook.Configuration configuration = (RepositoryWebhook.Configuration) subscription.getConfiguration();
      if (configuration.getRepository().equals(payload.getRepositoryName())) {
        queue(subscription, payload);
      }
    });
  }

  private RepositoryAssetWebhookPayload getPayload(final String repositoryName, final EventAction eventAction) {
    return new RepositoryAssetWebhookPayload(
        nodeAccess.getId(),
        new Date(),
        initiatorProvider.get(),
        repositoryName,
        eventAction);
  }

  private RepositoryAssetWebhookPayload getPayload(final AssetEvent event, final EventAction eventAction) {
    Optional<Repository> repository = event.getRepository();
    String repositoryName = repository.map(Repository::getName).orElse(null);
    String format = repository.map(r -> r.getFormat().getValue()).orElse(null);

    Asset asset = event.getAsset();
    EntityId assetId = InternalIds.toExternalId(InternalIds.internalAssetId(asset));
    RepositoryAssetWebhookPayload payload = getPayload(repositoryName, eventAction);
    payload.setAsset(new RepositoryAssetWebhookPayload.RepositoryAsset(
        assetId.getValue(),
        new RepositoryItemIDXO(repositoryName, assetId.getValue()).getValue(),
        format,
        asset.path()));
    return payload;
  }

  private RepositoryAssetWebhookPayload getPayload(final AssetPurgedEvent event, final EventAction eventAction) {
    Optional<Repository> repository = event.getRepository();
    String repositoryName = repository.map(Repository::getName).orElse(null);
    RepositoryAssetWebhookPayload payload = getPayload(repositoryName, eventAction);
    payload.setAssets(Arrays.stream(event.getAssetIds())
        .mapToObj(id -> InternalIds.toExternalId(id).getValue())
        .toArray(String[]::new));
    return payload;
  }

  static class RepositoryAssetWebhookPayload
      extends WebhookPayload
  {
    private final String repositoryName;

    private final EventAction action;

    private RepositoryAsset asset;

    private String[] assets;

    public RepositoryAssetWebhookPayload(
        final String nodeId,
        final Date timestamp,
        final String initiator,
        final String repositoryName,
        final EventAction action)
    {
      this.repositoryName = repositoryName;
      this.action = action;
      setNodeId(nodeId);
      setTimestamp(timestamp);
      setInitiator(initiator);
    }

    public String getRepositoryName() {
      return repositoryName;
    }

    public EventAction getAction() {
      return action;
    }

    public RepositoryAsset getAsset() {
      return asset;
    }

    public void setAsset(RepositoryAsset asset) {
      this.asset = asset;
    }

    public String[] getAssets() {
      return assets;
    }

    public void setAssets(String[] assets) {
      this.assets = assets;
    }

    public static class RepositoryAsset
    {
      private String id;

      private String assetId;

      private String format;

      private String name;

      public RepositoryAsset(String id, String assetId, String format, String name) {
        this.id = id;
        this.assetId = assetId;
        this.format = format;
        this.name = name;
      }

      public String getId() {
        return id;
      }

      public String getAssetId() {
        return assetId;
      }

      public String getFormat() {
        return format;
      }

      public String getName() {
        return name;
      }
    }
  }
}
