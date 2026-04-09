package org.globalart.astrum.repository.docker.internal.v2;

import java.nio.charset.StandardCharsets;
import java.util.List;

import org.globalart.astrum.repository.view.Parameters;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;

public class DockerProtocolHelpersTest
{
  private static Parameters parameters(final String n, final String last) {
    Parameters parameters = new Parameters();
    if (n != null) {
      parameters.set("n", n);
    }
    if (last != null) {
      parameters.set("last", last);
    }
    return parameters;
  }

  @Test
  public void paginationWithoutLimitReturnsEverythingSorted() {
    DockerPagination.Page page = DockerPagination.page(List.of("c", "a", "b"), parameters(null, null), "/v2/x");
    assertThat(page.getItems(), contains("a", "b", "c"));
    assertThat(page.getNextLink(), nullValue());
  }

  @Test
  public void paginationWithLimitAndLast() {
    List<String> values = List.of("a", "b", "c", "d", "e");
    DockerPagination.Page first = DockerPagination.page(values, parameters("2", null), "/v2/app/tags/list");
    assertThat(first.getItems(), contains("a", "b"));
    assertThat(first.getNextLink(), is("</v2/app/tags/list?n=2&last=b>; rel=\"next\""));
    DockerPagination.Page last = DockerPagination.page(values, parameters("2", "d"), "/v2/app/tags/list");
    assertThat(last.getItems(), contains("e"));
    assertThat(last.getNextLink(), nullValue());
  }

  @Test
  public void paginationEncodesLastValue() {
    DockerPagination.Page page = DockerPagination.page(List.of("repo/a", "repo/b"), parameters("1", null),
        "/v2/_catalog");
    assertThat(page.getNextLink(), is("</v2/_catalog?n=1&last=repo%2Fa>; rel=\"next\""));
  }

  @Test
  public void contentRangeParsing() {
    assertThat(DockerHostedHandler.parseContentRange("0-999"), is(new long[]{0, 999}));
    assertThat(DockerHostedHandler.parseContentRange("bytes 1000-1999/*"), is(new long[]{1000, 1999}));
    assertThat(DockerHostedHandler.parseContentRange("bytes=5-9"), is(new long[]{5, 9}));
    assertThat(DockerHostedHandler.parseContentRange(null), nullValue());
    assertThat(DockerHostedHandler.parseContentRange("garbage"), nullValue());
  }

  @Test
  public void digests() {
    String digest = DockerDigest.sha256("hello".getBytes(StandardCharsets.UTF_8));
    assertThat(digest, is("sha256:2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"));
    assertThat(DockerDigest.isSupported(digest), is(true));
    assertThat(DockerDigest.isDigest("sha512:abc"), is(true));
    assertThat(DockerDigest.isSupported("sha512:abc"), is(false));
    assertThat(DockerDigest.isDigest("latest"), is(false));
    assertThat(DockerDigest.hex(digest), is("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"));
  }
}
