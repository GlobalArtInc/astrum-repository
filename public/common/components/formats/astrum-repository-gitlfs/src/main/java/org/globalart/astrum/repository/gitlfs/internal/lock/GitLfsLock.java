package org.globalart.astrum.repository.gitlfs.internal.lock;

import java.time.OffsetDateTime;
import java.time.ZoneOffset;
import java.time.format.DateTimeFormatter;
import java.time.temporal.ChronoUnit;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.gitlfs.internal.GitLfsJson;

import com.fasterxml.jackson.databind.node.ObjectNode;

/**
 * A lock on a file path of a git repository, held by a user.
 */
public record GitLfsLock(String id, String path, @Nullable String ref, String owner, OffsetDateTime lockedAt)
{
  static GitLfsLock of(final GitLfsLockData data) {
    return new GitLfsLock(String.valueOf(data.getLockId()), data.getPath(), data.getRefName(), data.getOwner(),
        data.getLockedAt());
  }

  /**
   * @return the lock as the lock API describes it
   */
  public ObjectNode toJson() {
    ObjectNode node = GitLfsJson.object();
    node.put("id", id);
    node.put("path", path);
    node.put("locked_at", DateTimeFormatter.ISO_OFFSET_DATE_TIME.format(
        lockedAt.withOffsetSameInstant(ZoneOffset.UTC).truncatedTo(ChronoUnit.SECONDS)));
    node.putObject("owner").put("name", owner);
    return node;
  }
}
