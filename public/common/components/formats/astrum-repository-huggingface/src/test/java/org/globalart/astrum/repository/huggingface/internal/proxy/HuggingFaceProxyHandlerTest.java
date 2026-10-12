package org.globalart.astrum.repository.huggingface.internal.proxy;

import java.nio.charset.StandardCharsets;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.payloads.BytesPayload;

import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.mockito.Mockito.mock;

class HuggingFaceProxyHandlerTest
{
  @Test
  void readsRequestedPathsFromForms() throws Exception {
    Context context = context("paths=config.json&paths=onnx%2Fmodel.onnx&expand=false",
        "application/x-www-form-urlencoded");
    assertThat(HuggingFaceProxyHandler.requestedPaths(context), contains("config.json", "onnx/model.onnx"));
  }

  @Test
  void readsRequestedPathsFromJson() throws Exception {
    Context context = context("{\"paths\":[\"a.txt\",\"dir\"],\"expand\":true}", "application/json");
    assertThat(HuggingFaceProxyHandler.requestedPaths(context), contains("a.txt", "dir"));
  }

  @Test
  void readsRequestedPathsAlreadyParsedIntoParameters() throws Exception {
    Request request = new Request.Builder().action("POST").path("/api/models/a/b/paths-info/main")
        .parameter("paths", "README.md").build();
    assertThat(HuggingFaceProxyHandler.requestedPaths(new Context(mock(Repository.class), request)),
        contains("README.md"));
  }

  private static Context context(final String body, final String contentType) {
    Request request = new Request.Builder()
        .action("POST")
        .path("/api/models/acme/model/paths-info/main")
        .payload(new BytesPayload(body.getBytes(StandardCharsets.UTF_8), contentType))
        .build();
    return new Context(mock(Repository.class), request);
  }
}
