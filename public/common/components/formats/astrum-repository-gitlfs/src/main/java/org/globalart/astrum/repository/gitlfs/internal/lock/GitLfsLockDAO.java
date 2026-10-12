package org.globalart.astrum.repository.gitlfs.internal.lock;

import java.util.List;
import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.datastore.api.ContentDataAccess;

import org.apache.ibatis.annotations.Param;

/**
 * File locks of Git LFS repositories.
 */
public interface GitLfsLockDAO
    extends ContentDataAccess
{
  Optional<GitLfsLockData> read(@Param("repositoryId") int repositoryId, @Param("lockId") int lockId);

  Optional<GitLfsLockData> readPath(@Param("repositoryId") int repositoryId, @Param("path") String path);

  /**
   * Browses locks in the order they were created, optionally only the one on a path.
   */
  List<GitLfsLockData> browse(
      @Param("repositoryId") int repositoryId,
      @Nullable @Param("path") String path,
      @Nullable @Param("continuationToken") Integer continuationToken,
      @Param("limit") int limit);

  void create(GitLfsLockData lock);

  boolean delete(@Param("repositoryId") int repositoryId, @Param("lockId") int lockId);

  int deleteRepository(@Param("repositoryId") int repositoryId, @Param("limit") int limit);
}
