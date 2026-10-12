package org.globalart.astrum.repository.huggingface.internal.proxy;

import java.util.Set;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.huggingface.internal.HuggingFaceRequest;
import org.globalart.astrum.repository.huggingface.internal.HuggingFaceRequest.Kind;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Matcher;
import org.globalart.astrum.repository.view.Request;

import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.mock;

class HuggingFaceProxyRecipeTest
{
  private final Matcher files = HuggingFaceProxyRecipe.matcher(Set.of("GET", "HEAD"), Set.of(Kind.FILE, Kind.STORED));

  private final Matcher pathsInfo = HuggingFaceProxyRecipe.matcher(Set.of("POST"), Set.of(Kind.PATHS_INFO));

  @Test
  void matchesFilesAndRemembersTheParsedRequest() {
    Context context = context("HEAD", "/acme/model/resolve/main/config.json");
    assertThat(files.matches(context), is(true));
    HuggingFaceRequest request = context.getAttributes().require(HuggingFaceRequest.class);
    assertThat(request.repoId(), is("acme/model"));
    assertThat(request.path(), is("config.json"));
  }

  @Test
  void matchesOnlyItsMethodsAndKinds() {
    assertThat(files.matches(context("PUT", "/acme/model/resolve/main/config.json")), is(false));
    assertThat(files.matches(context("GET", "/api/models/acme/model")), is(false));
    assertThat(pathsInfo.matches(context("POST", "/api/models/acme/model/paths-info/main")), is(true));
    assertThat(pathsInfo.matches(context("GET", "/api/models/acme/model/paths-info/main")), is(false));
  }

  private static Context context(final String method, final String path) {
    return new Context(mock(Repository.class), new Request.Builder().action(method).path(path).build());
  }
}
