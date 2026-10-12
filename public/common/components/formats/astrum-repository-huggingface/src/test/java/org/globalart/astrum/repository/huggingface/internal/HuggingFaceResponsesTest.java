package org.globalart.astrum.repository.huggingface.internal;

import java.io.InputStream;
import java.nio.charset.StandardCharsets;

import org.globalart.astrum.repository.view.Response;

import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;

class HuggingFaceResponsesTest
{
  @Test
  void answersFilesWithTheHeadersClientsKeyTheirCacheBy() {
    Response response = HuggingFaceResponses.file(new HeadPayload(42, null), "abc", "def", 42);
    assertThat(response.getStatus().getCode(), is(200));
    assertThat(response.getHeaders().get("X-Repo-Commit"), is("abc"));
    assertThat(response.getHeaders().get("ETag"), is("\"def\""));
    assertThat(response.getHeaders().get("X-Linked-Etag"), is("\"def\""));
    assertThat(response.getHeaders().get("X-Linked-Size"), is("42"));
    assertThat(response.getHeaders().get("Location"), is(nullValue()));
    assertThat(response.getPayload().getSize(), is(42L));
  }

  @Test
  void answersErrorsWithTheirCodes() throws Exception {
    Response response = HuggingFaceResponses.error(HuggingFaceException.entryNotFound("abc", "x.bin"));
    assertThat(response.getStatus().getCode(), is(404));
    assertThat(response.getHeaders().get("X-Error-Code"), is("EntryNotFound"));
    assertThat(response.getHeaders().get("X-Repo-Commit"), is("abc"));
    assertThat(response.getHeaders().get("X-Error-Message"), is("x.bin does not exist at commit abc"));
    try (InputStream in = response.getPayload().openInputStream()) {
      assertThat(new String(in.readAllBytes(), StandardCharsets.UTF_8),
          is("{\"error\":\"x.bin does not exist at commit abc\"}"));
    }
  }

  @Test
  void keepsHeaderValuesOnOneLine() {
    assertThat(HuggingFaceResponses.headerValue("line\r\nInjected: yes"), is("line  Injected: yes"));
    assertThat(HuggingFaceResponses.headerValue("café"), is("caf"));
    assertThat(HuggingFaceResponses.headerValue(null), is("Not found"));
  }
}
