
package org.globalart.astrum.repository.content.store;

import java.util.Collection;
import java.util.List;
import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.common.entity.EntityId;
import org.globalart.astrum.datastore.api.ContentDataAccess;
import org.globalart.astrum.datastore.api.SchemaTemplate;
import org.globalart.astrum.repository.content.ContentRepository;

import org.apache.ibatis.annotations.Param;

/**
 * Content repository {@link ContentDataAccess}.
 *
 * @since 3.20
 */
@SchemaTemplate("format")
public interface ContentRepositoryDAO
    extends ContentDataAccess
{
  /**
   * Browse all content repositories in the content data store.
   */
  Collection<ContentRepository> browseContentRepositories();

  /**
   * Creates the given content repository in the content data store.
   *
   * @param contentRepository the repository to create
   */
  void createContentRepository(ContentRepositoryData contentRepository);

  /**
   * Retrieves a content repository from the content data store based on its config identity.
   *
   * @param configRepositoryId the config repository id
   * @return content repository if it was found
   */
  Optional<ContentRepository> readContentRepository(@Param("configRepositoryId") EntityId configRepositoryId);

  /**
   * Retrieves the latest attributes of the given content repository in the content data store.
   *
   * @param contentRepository the content repository to read
   * @return repository attributes if found
   */
  Optional<NestedAttributesMap> readContentRepositoryAttributes(ContentRepository contentRepository);

  /**
   * Updates the attributes of the given content repository in the content data store.
   *
   * @param contentRepository the content repository to update
   */
  void updateContentRepositoryAttributes(ContentRepository contentRepository);

  /**
   * Deletes a content repository from the content data store.
   *
   * @param contentRepository the content repository to delete
   * @return {@code true} if the content repository was deleted
   */
  boolean deleteContentRepository(ContentRepository contentRepository);

  /**
   * Retrieves the repository name and content repository id for the given name.
   *
   * @param repositoryFormat the format to retrieve
   * @return repository attributes if found
   */
  Optional<Map<String, Object>> readContentRepositoryId(
      @Param("repositoryFormat") String repositoryFormat,
      @Param("repositoryName") String repositoryName);

  /**
   * Retrieves the repository names and content repository ids.
   *
   * @param repositoryFormats the list formats to retrieve
   * @return repository attributes if found
   */
  List<Map<String, Object>> readAllContentRepositoryIds(@Param("repositoryFormats") List<String> repositoryFormats);
}
