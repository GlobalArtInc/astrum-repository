package org.globalart.astrum.repository.gitlfs.internal;

import java.io.IOException;

import org.globalart.astrum.repository.gitlfs.internal.GitLfsBatchRequest.ObjectSpec;
import org.globalart.astrum.repository.gitlfs.internal.GitLfsBatchRequest.Operation;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.junit.jupiter.api.Test;

import static org.globalart.astrum.repository.gitlfs.internal.GitLfsObjectsTest.OID;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.is;
import static org.junit.jupiter.api.Assertions.assertThrows;

class GitLfsBatchRequestTest
{
  private static final ObjectMapper MAPPER = new ObjectMapper();

  @Test
  void readsTheRequestGitLfsSends() throws IOException {
    GitLfsBatchRequest request = GitLfsBatchRequest.parse(json("""
        {"operation":"upload","transfers":["lfs-standalone-file","basic","ssh"],
         "ref":{"name":"refs/heads/main"},"objects":[{"oid":"%s","size":12}],"hash_algo":"sha256"}
        """.formatted(OID)));

    assertThat(request.getOperation(), is(Operation.UPLOAD));
    assertThat(request.getObjects(), contains(new ObjectSpec(OID, 12)));
  }

  @Test
  void defaultsToTheBasicTransferAndKeepsInvalidObjectsForPerObjectErrors() throws IOException {
    GitLfsBatchRequest request = GitLfsBatchRequest.parse(json("""
        {"operation":"download","objects":[{"oid":"nope","size":"12"},{"size":3}]}
        """));

    assertThat(request.getOperation(), is(Operation.DOWNLOAD));
    assertThat(request.getObjects(), contains(new ObjectSpec("nope", -1), new ObjectSpec(null, 3)));
  }

  @Test
  void refusesRequestsItCannotAnswer() {
    assertStatus(422, "{\"objects\":[]}");
    assertStatus(422, "{\"operation\":\"delete\",\"objects\":[]}");
    assertStatus(422, "{\"operation\":\"download\"}");
    assertStatus(409, "{\"operation\":\"download\",\"hash_algo\":\"sha512\",\"objects\":[]}");
    assertStatus(422, "{\"operation\":\"download\",\"transfers\":[\"tus\"],\"objects\":[]}");

    StringBuilder objects = new StringBuilder();
    for (int i = 0; i <= GitLfsBatchRequest.MAX_OBJECTS; i++) {
      objects.append(i == 0 ? "" : ",").append("{\"oid\":\"").append(OID).append("\",\"size\":1}");
    }
    assertStatus(413, "{\"operation\":\"download\",\"objects\":[" + objects + "]}");
  }

  private static void assertStatus(final int status, final String body) {
    GitLfsException e = assertThrows(GitLfsException.class, () -> GitLfsBatchRequest.parse(json(body)));
    assertThat(body, e.getStatus(), is(status));
  }

  private static JsonNode json(final String body) throws IOException {
    return MAPPER.readTree(body);
  }
}
