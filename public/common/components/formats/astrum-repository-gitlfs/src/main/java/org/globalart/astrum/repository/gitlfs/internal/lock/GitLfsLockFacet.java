package org.globalart.astrum.repository.gitlfs.internal.lock;

import java.util.List;
import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.Facet;

/**
 * File locks of a Git LFS repository. A path holds at most one lock.
 */
@Facet.Exposed
public interface GitLfsLockFacet
    extends Facet
{
  int DEFAULT_LIMIT = 100;

  int MAX_LIMIT = 1000;

  /**
   * The outcome of {@link #lock}: the new lock, or the lock that already held the path.
   */
  record Result(GitLfsLock lock, boolean created)
  {
  }

  /**
   * A page of locks and the cursor of the next page, {@code null} on the last page.
   */
  record Page(List<GitLfsLock> locks, @Nullable String nextCursor)
  {
  }

  Result lock(String path, @Nullable String ref, String owner);

  Optional<GitLfsLock> find(String id);

  /**
   * @param path   only the lock on this path when given
   * @param cursor the cursor of a previous page
   * @param limit  the page size, capped to {@link #MAX_LIMIT}
   */
  Page browse(@Nullable String path, @Nullable String cursor, int limit);

  boolean unlock(GitLfsLock lock);
}
