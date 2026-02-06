
package org.globalart.astrum.repository.content.store;

import java.util.Collection;
import java.util.List;
import java.util.Map;
import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.common.entity.EntityId;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.repository.content.AttributeOperation;
import org.globalart.astrum.repository.content.ContentRepository;
import org.globalart.astrum.repository.content.event.repository.ContentRepositoryAttributesDesEvent;
import org.globalart.astrum.repository.content.event.repository.ContentRepositoryAttributesEvent;
import org.globalart.astrum.repository.content.event.repository.ContentRepositoryCreatedEvent;
import org.globalart.astrum.repository.content.event.repository.ContentRepositoryDeletedEvent;
import org.globalart.astrum.repository.content.event.repository.ContentRepositoryPreDeleteEvent;
import org.globalart.astrum.repository.content.facet.ContentFacetFinder;
import org.globalart.astrum.transaction.Transactional;

import org.springframework.beans.factory.annotation.Autowired;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.content.AttributesHelper.applyAttributeChange;

/**
 * {@link ContentRepository} store.
 *
 * @since 3.21
 */
public class ContentRepositoryStore<T extends ContentRepositoryDAO>
    extends ContentStoreEventSupport<T>
{
  private ContentFacetFinder contentFacetFinder;

  @Autowired
  public ContentRepositoryStore(
      final DataSessionSupplier sessionSupplier,
      final ContentFacetFinder contentFacetFinder,
      final String contentStoreName,
      final Class<T> daoClass)
  {
    super(sessionSupplier, contentStoreName, daoClass);
    this.contentFacetFinder = checkNotNull(contentFacetFinder);
  }

  /**
   * Browse all content repositories in the content data store.
   */
  @Transactional
  public Collection<ContentRepository> browseContentRepositories() {
    return dao().browseContentRepositories();
  }

  /**
   * Creates the given content repository in the content data store.
   *
   * @param contentRepository the repository to create
   */
  @Transactional
  public void createContentRepository(final ContentRepositoryData contentRepository) {
    dao().createContentRepository(contentRepository);

    postCommitEvent(() -> new ContentRepositoryCreatedEvent(contentRepository));
  }

  /**
   * Retrieves a content repository from the content data store based on its config identity.
   *
   * @param configRepositoryId the config repository id
   * @return content repository if it was found
   */
  @Transactional
  public Optional<ContentRepository> readContentRepository(final EntityId configRepositoryId) {
    return dao().readContentRepository(configRepositoryId);
  }

  /**
   * Updates the attributes of the given content repository in the content data store.
   *
   * @param contentRepository the content repository to update
   */
  @Transactional
  public void updateContentRepositoryAttributes(
      final ContentRepository contentRepository,
      final AttributeOperation change,
      final String key,
      final @Nullable Object value)
  {
    // reload latest attributes, apply change, then update database if necessary
    dao().readContentRepositoryAttributes(contentRepository).ifPresent(attributes -> {
      ((ContentRepositoryData) contentRepository).setAttributes(attributes);

      if (applyAttributeChange(attributes, change, key, value)) {
        dao().updateContentRepositoryAttributes(contentRepository);

        postCommitEvent(() -> new ContentRepositoryAttributesEvent(contentRepository, change, key, value));
        contentFacetFinder.findRepository(this.format, contentRepository)
            .ifPresent(repository -> {
              String repoName = repository.getName();
              postCommitEvent(() -> new ContentRepositoryAttributesDesEvent(repoName));
            });
      }
    });
  }

  /**
   * Deletes a content repository from the content data store.
   *
   * @param contentRepository the content repository to delete
   * @return {@code true} if the content repository was deleted
   */
  @Transactional
  public boolean deleteContentRepository(final ContentRepository contentRepository) {
    preCommitEvent(() -> new ContentRepositoryPreDeleteEvent(contentRepository));

    boolean deleted = dao().deleteContentRepository(contentRepository);
    if (deleted) {
      postCommitEvent(() -> new ContentRepositoryDeletedEvent(contentRepository, this.format));
    }
    return deleted;
  }

  /**
   * Deletes a content repository from the content data store based on its config identity.
   *
   * @param configRepositoryId the config repository id
   * @return {@code true} if the content repository was deleted
   */
  @Transactional
  public boolean deleteContentRepository(final EntityId configRepositoryId) {
    return dao().readContentRepository(configRepositoryId)
        .map(this::deleteContentRepository)
        .orElse(false);
  }

  /**
   * Retrieves the repository name and content repository id for the given name.
   *
   * @param repositoryFormat the format to retrieve
   * @return repository attributes if found
   */
  @Transactional
  public Optional<Map<String, Object>> readContentRepositoryId(
      final String repositoryFormat,
      final String repositoryName)
  {
    return dao().readContentRepositoryId(repositoryFormat, repositoryName);
  }

  /**
   * Retrieves the repository names and content repository ids.
   *
   * @param repositoryFormats the list formats to retrieve
   * @return repository attributes if found
   */
  @Transactional
  public List<Map<String, Object>> readAllContentRepositoryIds(final List<String> repositoryFormats) {
    return dao().readAllContentRepositoryIds(repositoryFormats);
  }
}
