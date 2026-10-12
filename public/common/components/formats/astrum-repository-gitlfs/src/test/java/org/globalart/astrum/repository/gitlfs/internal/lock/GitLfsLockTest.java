package org.globalart.astrum.repository.gitlfs.internal.lock;

import java.time.OffsetDateTime;
import java.time.ZoneOffset;

import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

class GitLfsLockTest
{
  @Test
  void describesTheLockInUtcSeconds() {
    OffsetDateTime lockedAt = OffsetDateTime.of(2026, 10, 12, 12, 30, 15, 987_000_000, ZoneOffset.ofHours(3));

    assertThat(new GitLfsLock("7", "a/b.psd", "refs/heads/main", "dev", lockedAt).toJson().toString(),
        is("{\"id\":\"7\",\"path\":\"a/b.psd\",\"locked_at\":\"2026-10-12T09:30:15Z\",\"owner\":{\"name\":\"dev\"}}"));
  }

  @Test
  void readsTheStoredRow() {
    GitLfsLockData data = new GitLfsLockData();
    data.setLockId(3);
    data.setPath("x.bin");
    data.setRefName("refs/heads/dev");
    data.setOwner("admin");
    data.setLockedAt(OffsetDateTime.of(2026, 1, 2, 3, 4, 5, 0, ZoneOffset.UTC));

    GitLfsLock lock = GitLfsLock.of(data);

    assertThat(lock.id(), is("3"));
    assertThat(lock.path(), is("x.bin"));
    assertThat(lock.ref(), is("refs/heads/dev"));
    assertThat(lock.owner(), is("admin"));
  }
}
