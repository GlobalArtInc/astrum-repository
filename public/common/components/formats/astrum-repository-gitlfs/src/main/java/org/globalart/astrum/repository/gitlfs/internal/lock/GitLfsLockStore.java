package org.globalart.astrum.repository.gitlfs.internal.lock;

import java.util.List;
import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.repository.content.store.ContentStoreSupport;
import org.globalart.astrum.transaction.Transactional;

import org.springframework.beans.factory.annotation.Autowired;

/**
 * Store of Git LFS locks in a content data store.
 */
public class GitLfsLockStore
    extends ContentStoreSupport<GitLfsLockDAO>
{
  @Autowired
  public GitLfsLockStore(final DataSessionSupplier sessionSupplier, final String contentStoreName) {
    super(sessionSupplier, contentStoreName, GitLfsLockDAO.class);
  }

  @Transactional
  public Optional<GitLfsLockData> read(final int repositoryId, final int lockId) {
    return dao().read(repositoryId, lockId);
  }

  @Transactional
  public Optional<GitLfsLockData> readPath(final int repositoryId, final String path) {
    return dao().readPath(repositoryId, path);
  }

  @Transactional
  public List<GitLfsLockData> browse(
      final int repositoryId,
      @Nullable final String path,
      @Nullable final Integer continuationToken,
      final int limit)
  {
    return dao().browse(repositoryId, path, continuationToken, limit);
  }

  /**
   * @throws org.globalart.astrum.datastore.api.DuplicateKeyException when the path is already locked
   */
  @Transactional
  public GitLfsLockData create(final GitLfsLockData lock) {
    dao().create(lock);
    return lock;
  }

  @Transactional
  public boolean delete(final int repositoryId, final int lockId) {
    return dao().delete(repositoryId, lockId);
  }

  /*
   * Transactional is intentionally omitted so that each batch commits on its own
   */
  public void deleteRepository(final int repositoryId) {
    int count;
    do {
      count = deleteRepositoryBatch(repositoryId);
    }
    while (count > 0);
  }

  @Transactional
  protected int deleteRepositoryBatch(final int repositoryId) {
    return dao().deleteRepository(repositoryId, deleteBatchSize());
  }
}
