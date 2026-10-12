package org.globalart.astrum.repository.gitlfs.internal.lock;

import java.time.OffsetDateTime;
import java.time.ZoneOffset;
import java.time.temporal.ChronoUnit;
import java.util.List;
import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.common.stateguard.Guarded;
import org.globalart.astrum.datastore.api.DuplicateKeyException;
import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.store.FormatStoreManager;
import org.globalart.astrum.repository.content.store.InternalIds;
import org.globalart.astrum.repository.gitlfs.GitLfsFormat;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.FacetSupport.State.STARTED;

@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class GitLfsLockFacetImpl
    extends FacetSupport
    implements GitLfsLockFacet
{
  private final FormatStoreManager formatStoreManager;

  private GitLfsLockStore store;

  @Autowired
  public GitLfsLockFacetImpl(@Qualifier(GitLfsFormat.NAME) final FormatStoreManager formatStoreManager) {
    this.formatStoreManager = checkNotNull(formatStoreManager);
  }

  @Override
  protected void doStart() throws Exception {
    ContentFacetSupport contentFacet = (ContentFacetSupport) facet(ContentFacet.class);
    store = formatStoreManager.formatStore(contentFacet.stores().contentStoreName, GitLfsLockDAO.class);
  }

  @Override
  protected void doDelete() throws Exception {
    if (store != null) {
      store.deleteRepository(repositoryId());
    }
  }

  @Override
  @Guarded(by = STARTED)
  public Result lock(final String path, @Nullable final String ref, final String owner) {
    Optional<GitLfsLockData> existing = store.readPath(repositoryId(), path);
    if (existing.isPresent()) {
      return new Result(GitLfsLock.of(existing.get()), false);
    }
    GitLfsLockData data = new GitLfsLockData();
    data.setRepositoryId(repositoryId());
    data.setPath(path);
    data.setRefName(ref);
    data.setOwner(owner);
    data.setLockedAt(OffsetDateTime.now(ZoneOffset.UTC).truncatedTo(ChronoUnit.SECONDS));
    try {
      return new Result(GitLfsLock.of(store.create(data)), true);
    }
    catch (DuplicateKeyException e) {
      return store.readPath(repositoryId(), path)
          .map(held -> new Result(GitLfsLock.of(held), false))
          .orElseThrow(() -> e);
    }
  }

  @Override
  @Guarded(by = STARTED)
  public Optional<GitLfsLock> find(final String id) {
    return lockId(id).flatMap(lockId -> store.read(repositoryId(), lockId)).map(GitLfsLock::of);
  }

  @Override
  @Guarded(by = STARTED)
  public Page browse(@Nullable final String path, @Nullable final String cursor, final int limit) {
    int pageSize = Math.max(1, Math.min(limit, MAX_LIMIT));
    Integer after = cursor == null || cursor.isEmpty() ? null : lockId(cursor).orElse(Integer.MAX_VALUE);
    List<GitLfsLockData> rows = store.browse(repositoryId(), path, after, pageSize + 1);
    List<GitLfsLock> locks = rows.stream().limit(pageSize).map(GitLfsLock::of).toList();
    String next = rows.size() > pageSize ? locks.get(locks.size() - 1).id() : null;
    return new Page(locks, next);
  }

  @Override
  @Guarded(by = STARTED)
  public boolean unlock(final GitLfsLock lock) {
    return lockId(lock.id()).map(lockId -> store.delete(repositoryId(), lockId)).orElse(false);
  }

  private int repositoryId() {
    return InternalIds.contentRepositoryId(getRepository()).orElseThrow(IllegalStateException::new);
  }

  private static Optional<Integer> lockId(final String id) {
    try {
      return Optional.of(Integer.parseInt(id));
    }
    catch (NumberFormatException e) {
      return Optional.empty();
    }
  }
}
