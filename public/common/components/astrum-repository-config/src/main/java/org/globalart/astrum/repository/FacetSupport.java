
package org.globalart.astrum.repository;

import java.util.Optional;
import java.util.concurrent.locks.Lock;
import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.common.event.EventBus;
import org.globalart.astrum.common.event.EventHelper;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.common.stateguard.Guarded;
import org.globalart.astrum.common.stateguard.StateGuard;
import org.globalart.astrum.common.stateguard.StateGuardAware;
import org.globalart.astrum.common.stateguard.Transitions;
import org.globalart.astrum.distributed.event.service.api.common.RepositoryCacheSyncTokenEvent;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.manager.RepositoryAttributeService;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.FacetSupport.State.ATTACHED;
import static org.globalart.astrum.repository.FacetSupport.State.DELETED;
import static org.globalart.astrum.repository.FacetSupport.State.DESTROYED;
import static org.globalart.astrum.repository.FacetSupport.State.FAILED;
import static org.globalart.astrum.repository.FacetSupport.State.INITIALISED;
import static org.globalart.astrum.repository.FacetSupport.State.NEW;
import static org.globalart.astrum.repository.FacetSupport.State.STARTED;
import static org.globalart.astrum.repository.FacetSupport.State.STOPPED;

/**
 * Support for {@link Facet} implementations.
 *
 * @since 3.0
 */
public abstract class FacetSupport
    implements Facet, StateGuardAware
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  protected static final String CACHE_TOKEN_ATTRIBUTE = "cacheToken";

  private EventManager eventManager;

  protected RepositoryAttributeService repositoryAttributeService;

  @Autowired
  public void installDependencies(final EventManager eventManager) {
    this.eventManager = checkNotNull(eventManager);
  }

  @Autowired
  public void setRepositoryAttributeService(@Nullable final RepositoryAttributeService repositoryAttributeService) {
    this.repositoryAttributeService = repositoryAttributeService;
  }

  protected EventManager getEventManager() {
    return checkNotNull(eventManager);
  }

  protected RepositoryAttributeService repositoryAttributeService() {
    return repositoryAttributeService;
  }

  private Repository repository;

  protected Repository getRepository() {
    return checkNotNull(repository);
  }

  //
  // State
  //

  public static final class State
  {
    public static final String NEW = "NEW";

    public static final String ATTACHED = "ATTACHED";

    public static final String INITIALISED = "INITIALISED";

    public static final String STARTED = "STARTED";

    public static final String STOPPED = "STOPPED";

    public static final String DELETED = "DELETED";

    public static final String DESTROYED = "DESTROYED";

    public static final String FAILED = "FAILED";
  }

  protected final StateGuard states = new StateGuard.Builder()
      .logger(log)
      .initial(NEW)
      .failure(FAILED)
      .create();

  @Override
  @Nonnull
  public StateGuard getStateGuard() {
    return states;
  }

  //
  // Lifecycle
  //

  @Override
  @Transitions(from = NEW, to = ATTACHED)
  public void attach(final Repository repository) throws Exception {
    this.repository = checkNotNull(repository);
  }

  @Override
  @Guarded(by = {ATTACHED, STARTED, STOPPED})
  public void validate(final Configuration configuration) throws Exception {
    doValidate(configuration);
  }

  /**
   * Sub-class should override to provide configuration validation logic.
   */
  protected void doValidate(final Configuration configuration) throws Exception {
    // nop
  }

  /**
   * Common init/update configuration extension-point.
   *
   * By default this is called on {@link #init} and {@link #update}
   * unless sub-class overrides {@link #doInit} or {@link #doUpdate}.
   */
  protected void doConfigure(final Configuration configuration) throws Exception {
    // nop
  }

  @Override
  @Transitions(from = ATTACHED, to = INITIALISED)
  public void init() throws Exception {
    doInit(getRepository().getConfiguration());
  }

  protected void doInit(final Configuration configuration) throws Exception {
    doConfigure(configuration);
  }

  @Override
  @Guarded(by = STOPPED)
  public void update() throws Exception {
    doUpdate(getRepository().getConfiguration());
  }

  protected void doUpdate(final Configuration configuration) throws Exception {
    doConfigure(configuration);
  }

  @Override
  @Transitions(from = {INITIALISED, STOPPED}, to = STARTED)
  public void start() throws Exception {
    doStart();
    eventManager.register(this);
  }

  protected void doStart() throws Exception {
    // nop
  }

  @Override
  public Lock getWriteLock() {
    return states.getWriteLock();
  }

  /**
   * Stop the repository.
   *
   * Repository must have been previously started. Repository and all it's facets must have been locked
   * because transition configuration disables write lock. Repository is stopped before applying {@link #update}.
   */
  @Override
  @Transitions(from = STARTED, to = STOPPED, requiresWriteLock = false)
  public void stop() throws Exception {
    eventManager.unregister(this);
    doStop();
  }

  protected void doStop() throws Exception {
    // nop
  }

  @Override
  @Transitions(from = STOPPED, to = DELETED)
  public void delete() throws Exception {
    doDelete();
  }

  protected void doDelete() throws Exception {
    // nop
  }

  @Override
  @Transitions(to = DESTROYED)
  public void destroy() throws Exception {
    if (states.is(STARTED)) {
      stop();
    }

    doDestroy();
    this.repository = null;
  }

  protected void doDestroy() throws Exception {
    // nop
  }

  //
  // Helpers
  //

  /**
   * Lookup a facet on attached repository.
   *
   * Reduce some verbosity for commonly used repository operation.
   *
   * @see Repository#facet(Class)
   */
  @Nonnull
  protected <T extends Facet> T facet(final Class<T> type) throws MissingFacetException {
    return getRepository().facet(type);
  }

  /**
   * Lookup an {@link Optional} facet on attached repository.
   *
   * Reduce some verbosity for commonly used repository operation.
   *
   * @see Repository#optionalFacet(Class)
   */
  @Nonnull
  protected <T extends Facet> Optional<T> optionalFacet(final Class<T> type) {
    return getRepository().optionalFacet(type);
  }

  /**
   * @deprecated use {@link EventManager} instead
   */
  @Deprecated
  protected EventBus getEventBus() {
    return eventManager;
  }

  /**
   * Posts a cache synchronization event to coordinate cache tokens across cluster nodes.
   *
   * @param repository the repository for which to sync the cache token
   * @param cacheToken the cache token to synchronize
   */
  public void postCacheTokenEvent(final Repository repository, final String cacheToken) {
    checkNotNull(repository);
    if (!EventHelper.isReplicating()) {
      // Store cache token as repository attribute if the attribute storage service is available
      if (repositoryAttributeService() != null) {
        repositoryAttributeService().setRepositoryAttribute(repository, CACHE_TOKEN_ATTRIBUTE, cacheToken);
      }
      eventManager.post(new RepositoryCacheSyncTokenEvent(repository.getName(), cacheToken));
    }
  }
}
