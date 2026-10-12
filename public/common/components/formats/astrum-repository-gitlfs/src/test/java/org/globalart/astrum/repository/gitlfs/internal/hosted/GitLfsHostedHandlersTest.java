package org.globalart.astrum.repository.gitlfs.internal.hosted;

import java.io.IOException;
import java.io.InputStream;
import java.util.Optional;
import java.util.OptionalLong;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.gitlfs.internal.GitLfsContentFacet;
import org.globalart.astrum.repository.gitlfs.internal.GitLfsException;
import org.globalart.astrum.repository.gitlfs.internal.GitLfsJson;
import org.globalart.astrum.repository.gitlfs.internal.GitLfsPaths;
import org.globalart.astrum.repository.gitlfs.internal.GitLfsPaths.Endpoint;
import org.globalart.astrum.repository.gitlfs.internal.GitLfsPaths.Match;
import org.globalart.astrum.repository.gitlfs.internal.security.GitLfsSecurityFacet;
import org.globalart.astrum.repository.view.Content;
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
class GitLfsHostedHandlersTest
{
  private static final String OID = "4d7a214614ab2935c943f9e0ff69d22eadbb8f32b1258daaa5e2ca24d17e2393";

  private static final ObjectMapper MAPPER = new ObjectMapper();

  @Mock
  private Repository repository;

  @Mock
  private GitLfsContentFacet content;

  @Mock
  private GitLfsSecurityFacet security;

  @Mock
  private FluentAsset asset;

  private final GitLfsHostedHandlers underTest = new GitLfsHostedHandlers();

  @BeforeEach
  void setUp() {
    when(repository.getName()).thenReturn("lfs");
    when(repository.getUrl()).thenReturn("http://localhost/repository/lfs");
    when(repository.facet(GitLfsContentFacet.class)).thenReturn(content);
    when(repository.facet(GitLfsSecurityFacet.class)).thenReturn(security);
    when(content.size(any())).thenReturn(OptionalLong.empty());
    when(content.getObject(any())).thenReturn(Optional.empty());
  }

  @Test
  void batchBuildsHrefsOnTheApiTheClientReached() throws Exception {
    when(content.size(OID)).thenReturn(OptionalLong.of(12));
    when(security.isPermitted(any(Request.class), eq(OID), eq(BreadActions.READ))).thenReturn(true);

    Response response = underTest.batch.handle(context("POST", "/info/lfs/objects/batch",
        "{\"operation\":\"download\",\"objects\":[{\"oid\":\"" + OID + "\",\"size\":12}]}",
        new Match(Endpoint.BATCH, "/info/lfs", null)));

    assertThat(response.getStatus().getCode(), is(200));
    assertThat(response.getPayload().getContentType(), is(GitLfsJson.MEDIA_TYPE));
    assertThat(json(response).at("/objects/0/actions/download/href").asText(),
        is("http://localhost/repository/lfs/info/lfs/objects/" + OID));
  }

  @Test
  void batchChecksUploadsAgainstTheAddPrivilege() throws Exception {
    when(security.isPermitted(any(Request.class), eq(OID), eq(BreadActions.ADD))).thenReturn(true);

    Response response = underTest.batch.handle(context("POST", "/objects/batch",
        "{\"operation\":\"upload\",\"objects\":[{\"oid\":\"" + OID + "\",\"size\":12}]}",
        new Match(Endpoint.BATCH, "", null)));

    assertThat(json(response).at("/objects/0/actions/upload/href").asText(),
        is("http://localhost/repository/lfs/objects/" + OID));
    assertThat(json(response).at("/objects/0/actions/verify/href").asText(),
        is("http://localhost/repository/lfs/objects/verify"));
  }

  @Test
  void batchRefusesUploadsToReadOnlyRepositories() {
    when(content.isReadOnly()).thenReturn(true);

    GitLfsException e = assertThrows(GitLfsException.class, () -> underTest.batch.handle(context("POST",
        "/info/lfs/objects/batch", "{\"operation\":\"upload\",\"objects\":[]}",
        new Match(Endpoint.BATCH, "/info/lfs", null))));

    assertThat(e.getStatus(), is(403));
  }

  @Test
  void batchRefusesClientsThatDoNotAcceptJson() {
    Request request = new Request.Builder().action("POST").path("/info/lfs/objects/batch")
        .header("accept", "text/html").build();

    GitLfsException e = assertThrows(GitLfsException.class, () -> GitLfsHostedHandlers.checkAccept(request));

    assertThat(e.getStatus(), is(406));
  }

  @Test
  void uploadStoresNewObjectsOnly() throws Exception {
    Context context = context("PUT", "/info/lfs/objects/" + OID, "content",
        new Match(Endpoint.OBJECT, "/info/lfs", OID));

    assertThat(underTest.upload.handle(context).getStatus().getCode(), is(200));
    when(content.getObject(OID)).thenReturn(Optional.of(asset));
    assertThat(underTest.upload.handle(context).getStatus().getCode(), is(200));

    verify(content).put(eq(OID), any());
  }

  @Test
  void uploadRefusesReadOnlyRepositories() {
    when(content.isReadOnly()).thenReturn(true);

    GitLfsException e = assertThrows(GitLfsException.class, () -> underTest.upload.handle(
        context("PUT", "/info/lfs/objects/" + OID, "content", new Match(Endpoint.OBJECT, "/info/lfs", OID))));

    assertThat(e.getStatus(), is(403));
    verify(content, never()).put(any(), any());
  }

  @Test
  void verifyComparesTheStoredSize() throws Exception {
    when(content.size(OID)).thenReturn(OptionalLong.of(12));

    Response response = underTest.verify.handle(verifyContext("{\"oid\":\"" + OID + "\",\"size\":12}"));
    assertThat(response.getStatus().getCode(), is(200));
    assertThat(json(response).get("size").asLong(), is(12L));

    assertStatus(422, "{\"oid\":\"" + OID + "\",\"size\":13}");
    assertStatus(404, "{\"oid\":\"" + "0".repeat(64) + "\",\"size\":12}");
    assertStatus(422, "{\"oid\":\"" + OID + "\"}");
    assertStatus(400, "{oid");
  }

  @Test
  void downloadAnswersWithTheStoredContentOrNotFound() throws Exception {
    Content stored = new Content(new BytesPayload("data".getBytes(UTF_8), "application/octet-stream"));
    when(asset.download()).thenReturn(stored);
    when(content.getObject(OID)).thenReturn(Optional.of(asset));

    Response found = underTest.download.handle(context("GET", "/objects/4d/7a/" + OID, null,
        new Match(Endpoint.ASSET, "", OID)));
    assertThat(found.getStatus().getCode(), is(200));
    assertThat(found.getPayload(), is(stored));

    String missing = "1".repeat(64);
    Response notFound = underTest.download.handle(context("GET", "/info/lfs/objects/" + missing, null,
        new Match(Endpoint.OBJECT, "/info/lfs", missing)));
    assertThat(notFound.getStatus().getCode(), is(404));
    verify(content, never()).put(any(), any());
  }

  private void assertStatus(final int status, final String body) {
    GitLfsException e = assertThrows(GitLfsException.class, () -> underTest.verify.handle(verifyContext(body)));
    assertThat(body, e.getStatus(), is(status));
  }

  private Context verifyContext(final String body) {
    return context("POST", "/info/lfs/objects/verify", body, new Match(Endpoint.VERIFY, "/info/lfs", null));
  }

  private Context context(final String method, final String path, final String body, final Match match) {
    Request.Builder request = new Request.Builder().action(method).path(path)
        .header("Accept", GitLfsJson.MEDIA_TYPE);
    if (body != null) {
      request.payload(new BytesPayload(body.getBytes(UTF_8), GitLfsJson.MEDIA_TYPE));
    }
    Context context = new Context(repository, request.build());
    context.getAttributes().set(Match.class, match);
    assertThat(GitLfsPaths.matched(context), is(match));
    return context;
  }

  private static JsonNode json(final Response response) throws IOException {
    try (InputStream in = response.getPayload().openInputStream()) {
      return MAPPER.readTree(in);
    }
  }
}
