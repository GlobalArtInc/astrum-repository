package org.globalart.astrum.repository.gitlfs.internal;

import java.util.Optional;

import org.globalart.astrum.repository.gitlfs.internal.GitLfsPaths.Endpoint;
import org.globalart.astrum.repository.gitlfs.internal.GitLfsPaths.Match;

import org.junit.jupiter.api.Test;

import static org.globalart.astrum.repository.gitlfs.internal.GitLfsObjectsTest.OID;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

class GitLfsPathsTest
{
  @Test
  void resolvesTheApiBelowInfoLfs() {
    assertThat(GitLfsPaths.match("/info/lfs/objects/batch"), is(match(Endpoint.BATCH, "/info/lfs", null)));
    assertThat(GitLfsPaths.match("/info/lfs/objects/verify"), is(match(Endpoint.VERIFY, "/info/lfs", null)));
    assertThat(GitLfsPaths.match("/info/lfs/objects/" + OID), is(match(Endpoint.OBJECT, "/info/lfs", OID)));
    assertThat(GitLfsPaths.match("/info/lfs/locks"), is(match(Endpoint.LOCKS, "/info/lfs", null)));
    assertThat(GitLfsPaths.match("/info/lfs/locks/verify"), is(match(Endpoint.LOCKS_VERIFY, "/info/lfs", null)));
    assertThat(GitLfsPaths.match("/info/lfs/locks/42/unlock"), is(match(Endpoint.UNLOCK, "/info/lfs", "42")));
  }

  @Test
  void resolvesTheApiAtTheRepositoryRoot() {
    assertThat(GitLfsPaths.match("/objects/batch"), is(match(Endpoint.BATCH, "", null)));
    assertThat(GitLfsPaths.match("/objects/" + OID), is(match(Endpoint.OBJECT, "", OID)));
    assertThat(GitLfsPaths.match("/locks/7/unlock"), is(match(Endpoint.UNLOCK, "", "7")));
  }

  @Test
  void resolvesStoredObjectsByAssetPath() {
    assertThat(GitLfsPaths.match("/objects/4d/7a/" + OID), is(match(Endpoint.ASSET, "", OID)));
  }

  @Test
  void ignoresOtherPaths() {
    assertThat(GitLfsPaths.match("/"), is(Optional.empty()));
    assertThat(GitLfsPaths.match("/info/lfs"), is(Optional.empty()));
    assertThat(GitLfsPaths.match("/info/lfs/objects/abc"), is(Optional.empty()));
    assertThat(GitLfsPaths.match("/info/lfs/objects/" + OID.toUpperCase()), is(Optional.empty()));
    assertThat(GitLfsPaths.match("/info/lfs/locks/1/2/unlock"), is(Optional.empty()));
    assertThat(GitLfsPaths.match("/objects/4d/7b/" + OID), is(Optional.empty()));
    assertThat(GitLfsPaths.match(null), is(Optional.empty()));
  }

  private static Optional<Match> match(final Endpoint endpoint, final String base, final String value) {
    return Optional.of(new Match(endpoint, base, value));
  }
}
