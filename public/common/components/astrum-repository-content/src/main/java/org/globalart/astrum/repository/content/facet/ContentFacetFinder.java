
package org.globalart.astrum.repository.content.facet;

import java.util.Map;
import java.util.Optional;
import java.util.concurrent.ConcurrentHashMap;
import java.util.stream.Stream;

import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.RepositoryStartedEvent;
import org.globalart.astrum.repository.RepositoryStoppedEvent;
import org.globalart.astrum.repository.content.RepositoryContent;
import org.globalart.astrum.repository.group.GroupFacet;

import com.google.common.collect.ImmutableList;
import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static java.util.Optional.ofNullable;
import static org.globalart.astrum.repository.content.store.InternalIds.contentRepositoryId;
import org.springframework.stereotype.Component;

/**
 * Provides various methods for finding {@link ContentFacet}s and their repositories.
 *
 * @since 3.26
 */
@Component
public class ContentFacetFinder
    implements EventAware
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final Map<String, Repository> repositoriesByContentId = new ConcurrentHashMap<>();

  /**
   * Finds the repository the given content was uploaded to.
   */
  public Optional<Repository> findRepository(final String format, final RepositoryContent content) {
    return findRepository(format, contentRepositoryId(content));
  }

  /**
   * Finds the {@link ContentFacet} of the repository the given content was uploaded to.
   */
  public Optional<ContentFacet> findContentFacet(final String format, final RepositoryContent content) {
    return findContentFacet(format, contentRepositoryId(content));
  }

  /**
   * Finds the repository with the given contentRepositoryId.
   */
  public Optional<Repository> findRepository(final String format, final int contentRepositoryId) {
    return ofNullable(repositoriesByContentId.get(cacheKey(format, contentRepositoryId)));
  }

  /**
   * Finds the {@link ContentFacet} of the repository with the given contentRepositoryId.
   */
  public Optional<ContentFacet> findContentFacet(final String format, final int contentRepositoryId) {
    return findRepository(format, contentRepositoryId).map(r -> r.facet(ContentFacet.class));
  }

  /**
   * Finds the upper-most {@link ContentFacet}s contained within the given repository
   */
  public static Stream<ContentFacet> findContentFacets(final Repository repository) {

    // repository at this level has a content facet, use that
    Optional<ContentFacet> contentFacet = repository.optionalFacet(ContentFacet.class);
    if (contentFacet.isPresent()) {
      return Stream.of(contentFacet.get());
    }

    // otherwise repeat this search for any next-level repositories and merge the results
    return repository.optionalFacet(GroupFacet.class)
        .map(GroupFacet::members)
        .orElse(ImmutableList.of())
        .stream()
        .flatMap(ContentFacetFinder::findContentFacets);
  }

  @AllowConcurrentEvents
  @Subscribe
  public void on(final RepositoryStartedEvent event) {
    Repository repository = event.getRepository();
    String format = repository.getFormat().getValue();

    repository.optionalFacet(ContentFacet.class)
        .ifPresent(
            facet -> repositoriesByContentId.put(cacheKey(format, facet.contentRepositoryId()), repository));
  }

  @AllowConcurrentEvents
  @Subscribe
  public void on(final RepositoryStoppedEvent event) {
    Repository repository = event.getRepository();
    String format = repository.getFormat().getValue();

    repository.optionalFacet(ContentFacet.class)
        .ifPresent(
            facet -> repositoriesByContentId.remove(cacheKey(format, facet.contentRepositoryId())));
  }

  /**
   * Bind format and repository id to get a unique key across all formats.
   */
  private String cacheKey(final String format, final int contentRepositoryId) {
    return format + ':' + contentRepositoryId;
  }
}
