
package org.globalart.astrum.repository.helm.internal.content.createindex;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;
import org.springframework.context.annotation.Scope;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;

import java.util.concurrent.atomic.AtomicBoolean;


import org.globalart.astrum.common.event.EventAware.Asynchronous;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.common.stateguard.Guarded;
import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.event.asset.AssetCreatedEvent;
import org.globalart.astrum.repository.content.event.asset.AssetDeletedEvent;
import org.globalart.astrum.repository.content.event.asset.AssetEvent;
import org.globalart.astrum.repository.content.event.asset.AssetPurgedEvent;
import org.globalart.astrum.repository.content.event.asset.AssetUpdatedEvent;
import org.globalart.astrum.repository.content.event.asset.AssetUploadedEvent;
import org.globalart.astrum.repository.content.store.ContentStoreEvent;
import org.globalart.astrum.repository.manager.RepositoryCreatedEvent;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.helm.internal.content.HelmContentFacet;
import org.globalart.astrum.repository.helm.internal.content.recipe.HelmHostedFacet;
import org.globalart.astrum.repository.helm.internal.createindex.CreateIndexFacet;
import org.globalart.astrum.repository.helm.internal.createindex.HelmIndexInvalidationEvent;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport.State.STARTED;
import static org.globalart.astrum.repository.helm.internal.AssetKind.HELM_INDEX;
import static org.globalart.astrum.repository.helm.internal.AssetKind.HELM_PACKAGE;

/**
 * Facet for rebuilding Helm index.yaml files
 *
 * @since 3.28
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class CreateIndexFacetImpl
    extends FacetSupport
    implements CreateIndexFacet, Asynchronous
{
  private static final String INDEX_YAML = "/index.yaml";

  private final EventManager eventManager;

  private final long interval;

  private final AtomicBoolean acceptingEvents = new AtomicBoolean(true);

  private static final String UPDATING_INDEX_LOG = "Updating index.yaml for hosted repository {}";

  //Prevents the same event from being fired multiple times
  private final AtomicBoolean eventFired = new AtomicBoolean(false);

  private CreateIndexService createIndexService;

  @Autowired
  public CreateIndexFacetImpl(
      final EventManager eventManager,
      final CreateIndexService createIndexService,
      @Value("${astrum.helm.createrepo.interval:1000}") final long interval)
  {
    this.eventManager = checkNotNull(eventManager);
    this.interval = interval;
    this.createIndexService = checkNotNull(createIndexService);
  }

  @Subscribe
  @Guarded(by = STARTED)
  @AllowConcurrentEvents
  public void on(final RepositoryCreatedEvent createdEvent) {
    // at repository creation time, create index.yaml with empty entries
    log.debug("Initializing index.yaml for hosted repository {}", getRepository().getName());
    if (getRepository().getName().equals(createdEvent.getRepository().getName())) {
      invalidateIndex();
    }
  }

  @Subscribe
  @Guarded(by = STARTED)
  @AllowConcurrentEvents
  public void on(final AssetCreatedEvent created) {
    log.debug(UPDATING_INDEX_LOG, getRepository().getName());
    maybeInvalidateIndex(created);
  }

  @Subscribe
  @Guarded(by = STARTED)
  @AllowConcurrentEvents
  public void on(final AssetDeletedEvent deleted) {
    log.debug(UPDATING_INDEX_LOG, getRepository().getName());
    maybeInvalidateIndex(deleted);
  }

  @Subscribe
  @Guarded(by = STARTED)
  @AllowConcurrentEvents
  public void on(final AssetUpdatedEvent updated) {
    log.debug(UPDATING_INDEX_LOG, getRepository().getName());
    maybeInvalidateIndex(updated);
  }

  @Subscribe
  @Guarded(by = STARTED)
  @AllowConcurrentEvents
  public void on(final AssetUploadedEvent uploaded) {
    log.debug(UPDATING_INDEX_LOG, getRepository().getName());
    maybeInvalidateIndex(uploaded);
  }

  @Subscribe
  @Guarded(by = STARTED)
  @AllowConcurrentEvents
  public void on(final AssetPurgedEvent purged) {
    log.debug(UPDATING_INDEX_LOG, getRepository().getName());
    if (isRelevantEvent(purged)) {
      invalidateIndex();
    }
  }

  private void maybeInvalidateIndex(final AssetEvent event) {
    if (isRelevantEvent(event) && event.getAsset().kind().equals(HELM_PACKAGE.toString())) {
      invalidateIndex();
    }
  }

  private boolean isRelevantEvent(final ContentStoreEvent event) {
    return event.getRepository()
        .map(Repository::getName)
        .map(name -> name.equals(getRepository().getName()))
        .orElse(false);
  }

  @Subscribe
  public void on(final HelmIndexInvalidationEvent event) {
    if (shouldProcess(event)) {
      acceptingEvents.set(false);
      maybeWait(event);

      log.info("Rebuilding Helm index for repository {}", getRepository().getName());

      try {
        acceptingEvents.set(true);
        eventFired.set(false);

        updateIndexYaml(createIndexService.buildIndexYaml(getRepository()));
      }
      finally {
        log.info("Finished rebuilding Helm index for repository {}", getRepository().getName());
      }
    }
  }

  protected void updateIndexYaml(final Content indexYaml) {
    if (indexYaml == null) {
      deleteIndexYaml();
    }
    else {
      createIndexYaml(indexYaml);
    }
  }

  private void createIndexYaml(final Content indexYaml) {
    Repository repository = getRepository();
    HelmContentFacet helmFacet = repository.facet(HelmContentFacet.class);
    helmFacet.putIndex(INDEX_YAML, indexYaml, HELM_INDEX);
  }

  private void deleteIndexYaml() {
    log.debug("Empty index.yaml returned, proceeding to delete asset");
    HelmHostedFacet hosted = getRepository().facet(HelmHostedFacet.class);
    boolean result = hosted.delete(INDEX_YAML);
    if (result) {
      log.info("Deleted index.yaml because of empty asset list");
    }
    else {
      log.warn("Unable to delete index.yaml asset");
    }
  }

  private boolean shouldProcess(final HelmIndexInvalidationEvent event) {
    return getRepository().getName().equals(event.getRepositoryName());
  }

  private void maybeWait(final HelmIndexInvalidationEvent event) {
    if (event.isWaitBeforeRebuild()) {
      try {
        Thread.sleep(interval);
      }
      catch (InterruptedException e) { // NOSONAR
        log.warn("Helm invalidation thread interrupted, proceeding with invalidation");
      }
    }
  }

  /**
   * We are accepting that the boolean values can change in the middle of this if statement. For example if
   * acceptingEvents is true and eventFired is false but before we check eventFired acceptingEvents is toggled, we will
   * add one additional process request which will be handled synchronously and not cause any issues.
   */
  @Override
  public synchronized void invalidateIndex() {
    if (acceptingEvents.get() && !eventFired.get()) {
      log.info("Scheduling rebuild of Helm metadata to start in {} seconds", interval / 1000);

      // Prevent another event being fired if one is already queued up
      eventFired.set(true);
      eventManager.post(new HelmIndexInvalidationEvent(getRepository().getName(), true));
    }
  }
}
