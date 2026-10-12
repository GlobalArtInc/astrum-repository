package org.globalart.astrum.repository.gitlfs.internal;

import java.io.IOException;
import java.util.Map;
import java.util.OptionalLong;
import java.util.function.Predicate;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.junit.jupiter.api.Test;

import static org.globalart.astrum.repository.gitlfs.internal.GitLfsObjectsTest.OID;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.junit.jupiter.api.Assertions.assertThrows;

class GitLfsBatchTest
{
  private static final ObjectMapper MAPPER = new ObjectMapper();

  private static final String API = "http://localhost:8081/repository/lfs/info/lfs";

  private static final String MISSING = "0".repeat(64);

  private static final Map<String, Long> STORED = Map.of(OID, 12L);

  @Test
  void downloadsStoredObjectsAndReportsMissingOnes() throws IOException {
    JsonNode response = respond("download", all -> true, OID, 99, MISSING, 5);

    assertThat(response.toString(), is(MAPPER.readTree("""
        {"transfer":"basic","objects":[
          {"oid":"%s","size":12,"actions":{"download":{"href":"%s/objects/%s"}}},
          {"oid":"%s","size":5,"error":{"code":404,"message":"Object does not exist"}}
        ],"hash_algo":"sha256"}
        """.formatted(OID, API, OID, MISSING)).toString()));
  }

  @Test
  void hidesObjectsTheUserMayNotRead() {
    JsonNode response = respond("download", MISSING::equals, OID, 12);

    assertThat(response.at("/objects/0/error/code").asInt(), is(404));
  }

  @Test
  void asksForUploadAndVerifyOfMissingObjectsOnly() throws IOException {
    JsonNode response = respond("upload", all -> true, OID, 12, MISSING, 5);

    assertThat(response.toString(), is(MAPPER.readTree("""
        {"transfer":"basic","objects":[
          {"oid":"%s","size":12},
          {"oid":"%s","size":5,"actions":{
            "upload":{"href":"%s/objects/%s"},
            "verify":{"href":"%s/objects/verify"}}}
        ],"hash_algo":"sha256"}
        """.formatted(OID, MISSING, API, MISSING, API)).toString()));
  }

  @Test
  void reportsInvalidObjectsPerObject() {
    JsonNode response = respond("upload", all -> true, "xyz", 1, MISSING, -1);

    assertThat(response.at("/objects/0/error/code").asInt(), is(422));
    assertThat(response.at("/objects/1/error/code").asInt(), is(422));
  }

  @Test
  void refusesUploadsPerObjectOrForTheWholeRequest() {
    JsonNode response = respond("upload", MISSING::equals, OID, 12, MISSING, 5);
    assertThat(response.at("/objects/0/error/code").asInt(), is(403));
    assertThat(response.at("/objects/1/actions/upload/href").asText(), is(API + "/objects/" + MISSING));

    GitLfsException e = assertThrows(GitLfsException.class, () -> respond("upload", none -> false, OID, 12));
    assertThat(e.getStatus(), is(403));
  }

  private static JsonNode respond(final String operation, final Predicate<String> permitted, final Object... objects) {
    StringBuilder body = new StringBuilder("{\"operation\":\"" + operation + "\",\"objects\":[");
    for (int i = 0; i < objects.length; i += 2) {
      body.append(i == 0 ? "" : ",")
          .append("{\"oid\":\"").append(objects[i]).append("\",\"size\":").append(objects[i + 1]).append('}');
    }
    try {
      GitLfsBatchRequest request = GitLfsBatchRequest.parse(MAPPER.readTree(body.append("]}").toString()));
      return GitLfsBatch.respond(request, API,
          oid -> STORED.containsKey(oid) ? OptionalLong.of(STORED.get(oid)) : OptionalLong.empty(), permitted);
    }
    catch (IOException e) {
      throw new IllegalStateException(e);
    }
  }
}
