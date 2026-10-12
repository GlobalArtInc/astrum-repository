package org.globalart.astrum.repository.gitlfs.internal.lock;

import java.time.OffsetDateTime;

/**
 * A row of the {@code gitlfs_lock} table.
 */
public class GitLfsLockData
{
  private Integer lockId;

  private int repositoryId;

  private String path;

  private String refName;

  private String owner;

  private OffsetDateTime lockedAt;

  public Integer getLockId() {
    return lockId;
  }

  public void setLockId(final Integer lockId) {
    this.lockId = lockId;
  }

  public int getRepositoryId() {
    return repositoryId;
  }

  public void setRepositoryId(final int repositoryId) {
    this.repositoryId = repositoryId;
  }

  public String getPath() {
    return path;
  }

  public void setPath(final String path) {
    this.path = path;
  }

  public String getRefName() {
    return refName;
  }

  public void setRefName(final String refName) {
    this.refName = refName;
  }

  public String getOwner() {
    return owner;
  }

  public void setOwner(final String owner) {
    this.owner = owner;
  }

  public OffsetDateTime getLockedAt() {
    return lockedAt;
  }

  public void setLockedAt(final OffsetDateTime lockedAt) {
    this.lockedAt = lockedAt;
  }
}
