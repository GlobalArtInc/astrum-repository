package org.globalart.astrum.repository.gitlfs.internal.lock;

import java.time.OffsetDateTime;
import java.time.ZoneOffset;
import java.util.List;
import java.util.concurrent.ThreadLocalRandom;

import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.datastore.api.DuplicateKeyException;
import org.globalart.astrum.testdb.DataSessionRule;

import org.junit.Rule;
import org.junit.Test;

import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.empty;
import static org.hamcrest.Matchers.is;
import static org.junit.Assert.assertThrows;

public class GitLfsLockDAOTest
{
  private static final OffsetDateTime LOCKED_AT = OffsetDateTime.of(2026, 10, 12, 9, 30, 0, 0, ZoneOffset.UTC);

  private final int repository = ThreadLocalRandom.current().nextInt(1, Integer.MAX_VALUE / 2);

  private final int otherRepository = repository + 1;

  @Rule
  public DataSessionRule sessionRule = new DataSessionRule(DEFAULT_DATASTORE_NAME).access(GitLfsLockDAO.class);

  @Test
  public void storesAndReadsLocks() {
    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      GitLfsLockDAO dao = session.access(GitLfsLockDAO.class);
      GitLfsLockData lock = lock(repository, "art/cover.psd", "admin");
      dao.create(lock);

      assertThat(lock.getLockId() != null, is(true));
      GitLfsLockData read = dao.read(repository, lock.getLockId()).orElseThrow();
      assertThat(read.getPath(), is("art/cover.psd"));
      assertThat(read.getRefName(), is("refs/heads/main"));
      assertThat(read.getOwner(), is("admin"));
      assertThat(read.getLockedAt().toInstant(), is(LOCKED_AT.toInstant()));
      assertThat(dao.readPath(repository, "art/cover.psd").orElseThrow().getLockId(), is(lock.getLockId()));
      assertThat(dao.read(otherRepository, lock.getLockId()).isPresent(), is(false));
      assertThat(dao.readPath(repository, "art/other.psd").isPresent(), is(false));
    }
  }

  @Test
  public void allowsOneLockPerPathAndRepository() {
    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      GitLfsLockDAO dao = session.access(GitLfsLockDAO.class);
      dao.create(lock(repository, "model.bin", "admin"));
      dao.create(lock(otherRepository, "model.bin", "admin"));
      session.getTransaction().commit();

      assertThrows(DuplicateKeyException.class, () -> dao.create(lock(repository, "model.bin", "dev")));
    }
  }

  @Test
  public void browsesInCreationOrderFromACursor() {
    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      GitLfsLockDAO dao = session.access(GitLfsLockDAO.class);
      for (String path : List.of("a.bin", "b.bin", "c.bin")) {
        dao.create(lock(repository, path, "admin"));
      }
      dao.create(lock(otherRepository, "d.bin", "admin"));

      List<GitLfsLockData> first = dao.browse(repository, null, null, 2);
      assertThat(paths(first), contains("a.bin", "b.bin"));
      List<GitLfsLockData> rest = dao.browse(repository, null, first.get(1).getLockId(), 2);
      assertThat(paths(rest), contains("c.bin"));
      assertThat(paths(dao.browse(repository, "b.bin", null, 10)), contains("b.bin"));
    }
  }

  @Test
  public void deletesLocksOneByOneOrByRepository() {
    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      GitLfsLockDAO dao = session.access(GitLfsLockDAO.class);
      GitLfsLockData lock = lock(repository, "a.bin", "admin");
      dao.create(lock);
      dao.create(lock(repository, "b.bin", "admin"));
      dao.create(lock(repository, "c.bin", "admin"));
      dao.create(lock(otherRepository, "a.bin", "admin"));

      assertThat(dao.delete(otherRepository, lock.getLockId()), is(false));
      assertThat(dao.delete(repository, lock.getLockId()), is(true));
      assertThat(dao.deleteRepository(repository, 1), is(1));
      assertThat(dao.deleteRepository(repository, 10), is(1));
      assertThat(dao.browse(repository, null, null, 10), is(empty()));
      assertThat(dao.browse(otherRepository, null, null, 10).size(), is(1));
    }
  }

  private static List<String> paths(final List<GitLfsLockData> locks) {
    return locks.stream().map(GitLfsLockData::getPath).toList();
  }

  private static GitLfsLockData lock(final int repositoryId, final String path, final String owner) {
    GitLfsLockData lock = new GitLfsLockData();
    lock.setRepositoryId(repositoryId);
    lock.setPath(path);
    lock.setRefName("refs/heads/main");
    lock.setOwner(owner);
    lock.setLockedAt(LOCKED_AT);
    return lock;
  }
}
