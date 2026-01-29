
package org.globalart.astrum.repository.config.internal.datastore;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.common.event.EventConsumer;
import org.globalart.astrum.repository.MissingRepositoryException;
import org.globalart.astrum.repository.config.ConfigurationCreatedEvent;
import org.globalart.astrum.repository.config.ConfigurationDeletedEvent;
import org.globalart.astrum.repository.config.ConfigurationEvent;
import org.globalart.astrum.repository.config.ConfigurationUpdatedEvent;
import org.globalart.astrum.repository.manager.RepositoryManager;

import com.google.common.eventbus.Subscribe;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Repository configuration subscriber on DB events like CREATE/UPDATE/DELETE.
 */
@Component
public class ConfigurationSubscriber
    implements EventAware
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final RepositoryManager repositoryManager;

  @Autowired
  public ConfigurationSubscriber(final RepositoryManager repositoryManager) {
    this.repositoryManager = checkNotNull(repositoryManager);
  }

  @Subscribe
  public void on(final ConfigurationCreatedEvent event) {
    handleReplication(event,
        e -> {
          // Handle missing configuration during event replication race conditions
          var config = repositoryManager.retrieveConfigurationByName(e.getRepositoryName());
          if (config.isEmpty()) {
            log.warn("Configuration '{}' not yet available during event replication, skipping create",
                e.getRepositoryName());
            return;
          }
          repositoryManager.create(config.get());
        });
  }

  @Subscribe
  public void on(final ConfigurationUpdatedEvent event) {
    handleReplication(event,
        e -> {
          // Handle missing configuration during event replication race conditions
          var config = repositoryManager.retrieveConfigurationByName(e.getRepositoryName());
          if (config.isEmpty()) {
            log.warn("Configuration '{}' not yet available during event replication, skipping update",
                e.getRepositoryName());
            return;
          }
          repositoryManager.update(config.get());
        });
  }

  @Subscribe
  public void on(final ConfigurationDeletedEvent event) {
    handleReplication(event, e -> repositoryManager.delete(e.getRepositoryName()));
  }

  private void handleReplication(final ConfigurationEvent event, final EventConsumer<ConfigurationEvent> consumer) {
    if (!event.isLocal()) {
      try {
        consumer.accept(event);
      }
      catch (MissingRepositoryException e) {
        log.debug("Race condition during replication of {}: {}", event, e.getMessage());
      }
      catch (Exception e) {
        log.error("Failed to replicate: {}", event, e);
      }
    }
  }
}
