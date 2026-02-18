
package org.globalart.astrum.repository.move;

import java.util.List;

import javax.annotation.Nullable;

/**
 * @since 3.28
 */
public interface ChangeRepositoryBlobStoreStore
{
  /**
   * @return all {@link ChangeRepositoryBlobStoreConfiguration} objects.
   */
  List<ChangeRepositoryBlobStoreConfiguration> list();

  /**
   * Persist a new {@link ChangeRepositoryBlobStoreConfiguration} object.
   */
  void create(final ChangeRepositoryBlobStoreConfiguration configuration);

  @Nullable
  ChangeRepositoryBlobStoreConfiguration findByRepositoryName(String repository);

  /**
   * Finds a ChangeRepository Blob Store task by blob store name, it could be source, or target blob store
   * 
   * @param blobStoreName
   * @return
   */
  @Nullable
  List<ChangeRepositoryBlobStoreConfiguration> findByBlobStoreName(final String blobStoreName);

  void add(final String repository, final String sourceBlobStoreName, final String targetBlobStoreName);

  void update(final String repository, final String sourceBlobStoreName, final String targetBlobStoreName);

  void remove(final String repository);
}
