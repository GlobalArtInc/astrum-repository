package org.globalart.astrum.repository.gitlfs.internal.lock;

import java.io.IOException;
import java.io.InputStream;
import java.time.OffsetDateTime;
import java.time.ZoneOffset;
import java.util.List;
import java.util.Optional;
import java.util.concurrent.atomic.AtomicReference;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.gitlfs.internal.GitLfsException;
import org.globalart.astrum.repository.gitlfs.internal.GitLfsPaths.Endpoint;
import org.globalart.astrum.repository.gitlfs.internal.GitLfsPaths.Match;
import org.globalart.astrum.repository.gitlfs.internal.lock.GitLfsLockFacet.Page;
import org.globalart.astrum.repository.gitlfs.internal.lock.GitLfsLockFacet.Result;
import org.globalart.astrum.repository.gitlfs.internal.security.GitLfsSecurityFacet;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.payloads.BytesPayload;
import org.globalart.astrum.security.BreadActions;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import static java.nio.charset.StandardCharsets.UTF_8;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class GitLfsLockHandlersTest
{
  private static final ObjectMapper MAPPER = new ObjectMapper();

  private static final OffsetDateTime LOCKED_AT = OffsetDateTime.of(2026, 10, 12, 9, 30, 0, 0, ZoneOffset.UTC);

  private static final GitLfsLock ADMIN_LOCK = new GitLfsLock("1", "model.bin", null, "admin", LOCKED_AT);

  private static final GitLfsLock DEV_LOCK = new GitLfsLock("2", "data/set.bin", "refs/heads/main", "dev", LOCKED_AT);

  @Mock
  private Repository repository;

  @Mock
  private GitLfsLockFacet locks;

  @Mock
  private GitLfsSecurityFacet security;

  private final AtomicReference<String> user = new AtomicReference<>("dev");

  private final GitLfsLockHandlers underTest = new GitLfsLockHandlers(user::get);

  @BeforeEach
  void setUp() {
    when(repository.getName()).thenReturn("lfs");
    when(repository.facet(GitLfsLockFacet.class)).thenReturn(locks);
    when(repository.facet(GitLfsSecurityFacet.class)).thenReturn(security);
    when(locks.find("1")).thenReturn(Optional.of(ADMIN_LOCK));
    when(locks.find("2")).thenReturn(Optional.of(DEV_LOCK));
  }

  @Test
  void createsALockForTheCurrentUser() throws Exception {
    when(locks.lock("data/set.bin", "refs/heads/main", "dev")).thenReturn(new Result(DEV_LOCK, true));

    Response response = underTest.locks.handle(context("POST", "/info/lfs/locks",
        "{\"path\":\"data/set.bin\",\"ref\":{\"name\":\"refs/heads/main\"}}", Endpoint.LOCKS, null));

    assertThat(response.getStatus().getCode(), is(201));
    assertThat(json(response).toString(), is(MAPPER.readTree("""
        {"lock":{"id":"2","path":"data/set.bin","locked_at":"2026-10-12T09:30:00Z","owner":{"name":"dev"}}}
        """).toString()));
  }

  @Test
  void answersAConflictWithTheExistingLock() {
    when(locks.lock("model.bin", null, "dev")).thenReturn(new Result(ADMIN_LOCK, false));

    GitLfsException e = assertThrows(GitLfsException.class, () -> underTest.locks.handle(
        context("POST", "/info/lfs/locks", "{\"path\":\"model.bin\"}", Endpoint.LOCKS, null)));

    assertThat(e.getStatus(), is(409));
    JsonNode body = MAPPER.valueToTree(e.getBody());
    assertThat(body.at("/lock/id").asText(), is("1"));
    assertThat(body.get("message").asText(), is("already created lock"));
  }

  @Test
  void refusesLocksWithoutAPath() {
    for (String body : List.of("{}", "{\"path\":\"  \"}", "{\"path\":42}")) {
      GitLfsException e = assertThrows(GitLfsException.class,
          () -> underTest.locks.handle(context("POST", "/info/lfs/locks", body, Endpoint.LOCKS, null)));
      assertThat(body, e.getStatus(), is(422));
    }
  }

  @Test
  void listsLocksByPathIdOrPage() throws Exception {
    when(locks.browse(null, null, GitLfsLockFacet.DEFAULT_LIMIT)).thenReturn(new Page(List.of(ADMIN_LOCK), "1"));
    when(locks.browse("model.bin", "1", 5)).thenReturn(new Page(List.of(), null));

    JsonNode page = json(underTest.locks.handle(context("GET", "/info/lfs/locks", null, Endpoint.LOCKS, null)));
    assertThat(page.at("/locks/0/path").asText(), is("model.bin"));
    assertThat(page.get("next_cursor").asText(), is("1"));

    JsonNode byPath = json(underTest.locks.handle(context("GET", "/info/lfs/locks", null, Endpoint.LOCKS, null,
        "path", "model.bin", "cursor", "1", "limit", "5")));
    assertThat(byPath.get("locks").size(), is(0));
    assertThat(byPath.has("next_cursor"), is(false));

    JsonNode byId = json(underTest.locks.handle(context("GET", "/info/lfs/locks", null, Endpoint.LOCKS, null,
        "id", "2")));
    assertThat(byId.at("/locks/0/owner/name").asText(), is("dev"));

    JsonNode byIdAndOtherPath = json(underTest.locks.handle(context("GET", "/info/lfs/locks", null, Endpoint.LOCKS,
        null, "id", "2", "path", "model.bin")));
    assertThat(byIdAndOtherPath.get("locks").size(), is(0));
  }

  @Test
  void verifySplitsLocksIntoOursAndTheirs() throws Exception {
    when(locks.browse(null, null, 10)).thenReturn(new Page(List.of(ADMIN_LOCK, DEV_LOCK), null));

    JsonNode body = json(underTest.verify.handle(context("POST", "/info/lfs/locks/verify",
        "{\"limit\":10,\"ref\":{\"name\":\"refs/heads/main\"}}", Endpoint.LOCKS_VERIFY, null)));

    assertThat(body.at("/ours/0/id").asText(), is("2"));
    assertThat(body.get("ours").size(), is(1));
    assertThat(body.at("/theirs/0/id").asText(), is("1"));
    assertThat(body.get("theirs").size(), is(1));
  }

  @Test
  void unlocksOwnLocks() throws Exception {
    Response response = underTest.unlock.handle(
        context("POST", "/info/lfs/locks/2/unlock", "{\"force\":false}", Endpoint.UNLOCK, "2"));

    assertThat(response.getStatus().getCode(), is(200));
    assertThat(json(response).at("/lock/path").asText(), is("data/set.bin"));
    verify(locks).unlock(DEV_LOCK);
  }

  @Test
  void unlocksLocksOfOthersOnlyByForceWithTheDeletePrivilege() throws Exception {
    GitLfsException withoutForce = assertThrows(GitLfsException.class, () -> underTest.unlock.handle(
        context("POST", "/info/lfs/locks/1/unlock", "{\"force\":false}", Endpoint.UNLOCK, "1")));
    assertThat(withoutForce.getStatus(), is(403));

    GitLfsException withoutPrivilege = assertThrows(GitLfsException.class, () -> underTest.unlock.handle(
        context("POST", "/info/lfs/locks/1/unlock", "{\"force\":true}", Endpoint.UNLOCK, "1")));
    assertThat(withoutPrivilege.getStatus(), is(403));
    verify(locks, never()).unlock(any());

    when(security.isPermitted(any(Request.class), eq(BreadActions.DELETE))).thenReturn(true);
    Response response = underTest.unlock.handle(
        context("POST", "/info/lfs/locks/1/unlock", "{\"force\":true}", Endpoint.UNLOCK, "1"));
    assertThat(response.getStatus().getCode(), is(200));
    verify(locks).unlock(ADMIN_LOCK);
  }

  @Test
  void unlockOfAnUnknownLockIsNotFound() {
    when(locks.find("9")).thenReturn(Optional.empty());

    GitLfsException e = assertThrows(GitLfsException.class, () -> underTest.unlock.handle(
        context("POST", "/info/lfs/locks/9/unlock", null, Endpoint.UNLOCK, "9")));

    assertThat(e.getStatus(), is(404));
  }

  @Test
  void capsTheLimit() {
    assertThat(GitLfsLockHandlers.limit(null), is(GitLfsLockFacet.DEFAULT_LIMIT));
    assertThat(GitLfsLockHandlers.limit("20"), is(20));
    assertThat(GitLfsLockHandlers.limit("100000"), is(GitLfsLockFacet.MAX_LIMIT));
    assertThat(assertThrows(GitLfsException.class, () -> GitLfsLockHandlers.limit("0")).getStatus(), is(422));
    assertThat(assertThrows(GitLfsException.class, () -> GitLfsLockHandlers.limit("ten")).getStatus(), is(422));
  }

  private Context context(
      final String method,
      final String path,
      final String body,
      final Endpoint endpoint,
      final String value,
      final String... parameters)
  {
    Request.Builder request = new Request.Builder().action(method).path(path);
    for (int i = 0; i < parameters.length; i += 2) {
      request.parameter(parameters[i], parameters[i + 1]);
    }
    if (body != null) {
      request.payload(new BytesPayload(body.getBytes(UTF_8), "application/vnd.git-lfs+json"));
    }
    Context context = new Context(repository, request.build());
    context.getAttributes().set(Match.class, new Match(endpoint, "/info/lfs", value));
    return context;
  }

  private static JsonNode json(final Response response) throws IOException {
    try (InputStream in = response.getPayload().openInputStream()) {
      return MAPPER.readTree(in);
    }
  }
}
