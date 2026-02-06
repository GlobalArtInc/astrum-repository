
package org.globalart.astrum.repository.content.store;

import java.util.AbstractMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.OptionalInt;
import java.util.concurrent.ConcurrentHashMap;
import java.util.stream.Collectors;

import org.globalart.astrum.common.db.DatabaseCheck;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.manager.RepositoryCreatedEvent;
import org.globalart.astrum.repository.manager.RepositoryDeletedEvent;

import com.google.common.collect.Iterables;
import com.google.common.eventbus.Subscribe;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.FeatureFlags.REPOSITORY_SIZE_ENABLED;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

/**
 * Maintain mapping of repository name and id
 */
@Component
@ConditionalOnProperty(name = REPOSITORY_SIZE_ENABLED, havingValue = "true", matchIfMissing = true)
public class RepositoryNameIdMappingCache
    implements EventAware
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private volatile Map<String, Integer> nameRepositoryIdMap;

  private final ContentRepositoryStore<?> contentRepositoryStore;

  private final List<String> formatNames;

  private final DatabaseCheck databaseCheck;

  private Boolean available;

  @Autowired
  public RepositoryNameIdMappingCache(
      final List<FormatStoreManager> formatStoreManagers,
      final List<Format> formats,
      final DatabaseCheck databaseCheck)
  {
    this.contentRepositoryStore =
        Iterables.get(checkNotNull(formatStoreManagers), 0).contentRepositoryStore(DEFAULT_DATASTORE_NAME);
    this.formatNames = checkNotNull(formats).stream().map(Format::getValue).collect(Collectors.toList());
    this.databaseCheck = checkNotNull(databaseCheck);
  }

  @Subscribe
  public void on(final RepositoryCreatedEvent event) {
    if (!isAvailable()) {
      return;
    }
    Repository repository = event.getRepository();
    log.debug("Handling repository create event for {}", repository.getName());
    fetchRepositoryId(repository.getName(), repository.getFormat().getValue());
  }

  @Subscribe
  public void on(final RepositoryDeletedEvent event) {
    if (!isAvailable()) {
      return;
    }
    log.debug("Handling repository deleted event for {}", event.getRepository().getName());
    if (nameRepositoryIdMap != null) {
      nameRepositoryIdMap.remove(event.getRepository().getName());
    }
  }

  private boolean isAvailable() {
    if (available == null) {
      available = databaseCheck.isPostgresql();
    }
    return available;
  }

  public Map<Integer, String> getRepositoryNameIds(final List<String> repositoryNames, final String format) {
    return repositoryNames.stream()
        .map(name -> new AbstractMap.SimpleEntry<>(fetchRepositoryId(name, format), name))
        .filter(entry -> entry.getKey().isPresent())
        .collect(Collectors.toMap(entry -> entry.getKey().getAsInt(), AbstractMap.SimpleEntry::getValue));
  }

  private Map<String, Integer> getCache() {
    Map<String, Integer> cache = nameRepositoryIdMap;
    if (cache == null) {
      synchronized (this) {
        if (nameRepositoryIdMap == null) {
          nameRepositoryIdMap = populateCache();
        }
        cache = nameRepositoryIdMap;
      }
    }
    return cache;
  }

  private OptionalInt fetchRepositoryId(final String name, final String format) {
    if (getCache().containsKey(name)) {
      return OptionalInt.of(getCache().get(name));
    }
    Optional<Map<String, Object>> repositoryNameId = contentRepositoryStore.readContentRepositoryId(format, name);
    if (repositoryNameId.isPresent() && repositoryNameId.get().containsValue(name)) {
      int repositoryId = (Integer) repositoryNameId.get().get("repository_id");
      nameRepositoryIdMap.put(name, repositoryId);
      return OptionalInt.of(repositoryId);
    }
    else {
      return OptionalInt.empty();
    }
  }

  private Map<String, Integer> populateCache() {
    Map<String, Integer> cache = new ConcurrentHashMap<>();
    List<Map<String, Object>> ids = contentRepositoryStore.readAllContentRepositoryIds(this.formatNames);
    ids.stream().forEach(id -> cache.put((String) id.get("name"), (Integer) id.get("repository_id")));
    return cache;
  }
}
