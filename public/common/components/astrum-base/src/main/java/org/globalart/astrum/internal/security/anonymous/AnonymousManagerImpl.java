
package org.globalart.astrum.internal.security.anonymous;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.inject.Provider;

import org.globalart.astrum.common.concurrent.Mutex;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.common.event.EventConsumer;
import org.globalart.astrum.common.event.EventHelper;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.jmx.reflect.ManagedAttribute;
import org.globalart.astrum.jmx.reflect.ManagedObject;
import org.globalart.astrum.security.anonymous.AnonymousConfiguration;
import org.globalart.astrum.security.anonymous.AnonymousConfigurationChangedEvent;
import org.globalart.astrum.security.anonymous.AnonymousManager;
import org.globalart.astrum.security.anonymous.AnonymousPrincipalCollection;

import com.google.common.eventbus.Subscribe;
import org.apache.shiro.subject.PrincipalCollection;
import org.apache.shiro.subject.Subject;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Default {@link AnonymousManagerImpl}.
 *
 * @since 3.0
 */
@Component
@ManagedObject
public class AnonymousManagerImpl
    implements AnonymousManager, EventAware
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final EventManager eventManager;

  private final AnonymousConfigurationStore store;

  private final Provider<AnonymousConfiguration> defaults;

  private final Mutex lock = new Mutex();

  private AnonymousConfiguration configuration;

  @Autowired
  public AnonymousManagerImpl(
      final EventManager eventManager,
      final AnonymousConfigurationStore store,
      @Qualifier("initial") final Provider<AnonymousConfiguration> defaults)
  {
    this.eventManager = checkNotNull(eventManager);
    this.store = checkNotNull(store);
    log.debug("Store: {}", store);
    this.defaults = checkNotNull(defaults);
    log.debug("Defaults: {}", defaults);
  }

  @Override
  public boolean isConfigured() {
    return store.load() != null;
  }

  //
  // Configuration
  //

  /**
   * Load configuration from store, or use defaults.
   */
  private AnonymousConfiguration loadConfiguration() {
    AnonymousConfiguration model = store.load();

    // use defaults if no configuration was loaded from the store
    if (model == null) {
      AnonymousConfiguration defaultModel = defaults.get();

      // default config must not be null
      checkNotNull(defaultModel);

      model = store.newConfiguration();
      model.setEnabled(defaultModel.isEnabled());
      model.setRealmName(defaultModel.getRealmName());
      model.setUserId(defaultModel.getUserId());

      log.info("Using default configuration: {}", model);
    }
    else {
      log.info("Loaded configuration: {}", model);
    }

    return model;
  }

  /**
   * Return configuration, loading if needed.
   *
   * The result model should be considered _immutable_ unless copied.
   */
  private AnonymousConfiguration getConfigurationInternal() {
    synchronized (lock) {
      if (configuration == null) {
        configuration = loadConfiguration();
      }
      return configuration;
    }
  }

  /**
   * Return _copy_ of configuration.
   */
  @Override
  public AnonymousConfiguration getConfiguration() {
    return getConfigurationInternal().copy();
  }

  @Override
  public AnonymousConfiguration newConfiguration() {
    return store.newConfiguration();
  }

  @Override
  public void setConfiguration(final AnonymousConfiguration configuration) {
    checkNotNull(configuration);

    AnonymousConfiguration model = configuration.copy();

    log.info("Saving configuration: {}", model);

    synchronized (lock) {
      if (!EventHelper.isReplicating()) {
        store.save(model);
      }
      this.configuration = model;
    }

    eventManager.post(new AnonymousConfigurationChangedEvent(model));
  }

  //
  // Helpers
  //

  @Override
  @ManagedAttribute
  public boolean isEnabled() {
    return getConfigurationInternal().isEnabled();
  }

  @Override
  public Subject buildSubject() {
    AnonymousConfiguration model = getConfigurationInternal();

    log.trace("Building anonymous subject with user-id: {}, realm-name: {}", model.getUserId(), model.getRealmName());

    // custom principals to aid with anonymous subject detection
    PrincipalCollection principals = new AnonymousPrincipalCollection(
        model.getUserId(),
        model.getRealmName());

    // FIXME: buildSubject() calls deeply into various shiro dao/save bits which are probably overhead we don't need
    // here at all

    return new Subject.Builder()
        .principals(principals)
        .authenticated(false)
        .sessionCreationEnabled(false)
        .buildSubject();
  }

  /**
   * @since 3.2
   */
  @Subscribe
  public void onStoreChanged(final AnonymousConfigurationEvent event) {
    handleReplication(event, e -> setConfiguration(e.getAnonymousConfiguration()));
  }

  private void handleReplication(
      final AnonymousConfigurationEvent event,
      final EventConsumer<AnonymousConfigurationEvent> consumer)
  {
    if (!event.isLocal()) {
      try {
        consumer.accept(event);
      }
      catch (Exception e) {
        log.error("Failed to replicate: {}", event, e);
      }
    }
  }
}
