package org.globalart.astrum.repository.huggingface.internal.proxy;

import org.apache.http.HttpVersion;
import org.apache.http.message.BasicHttpResponse;
import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;

class HubResponseTest
{
  @Test
  void readsTheLinkedEtagAndSizeOfLfsRedirects() {
    BasicHttpResponse response = new BasicHttpResponse(HttpVersion.HTTP_1_1, 302, "Found");
    response.addHeader("Location", "https://cdn.example.com/blob?sig=1");
    response.addHeader("ETag", "\"pointer\"");
    response.addHeader("x-linked-etag", "\"abcdef\"");
    response.addHeader("X-Linked-Size", "1234");
    response.addHeader("X-Repo-Commit", "c0ffee");

    HubResponse hub = HubResponse.of(response);
    assertThat(hub.isRedirect(), is(true));
    assertThat(hub.linked(), is(true));
    assertThat(hub.etag(), is("abcdef"));
    assertThat(hub.size(), is(1234L));
    assertThat(hub.commit(), is("c0ffee"));
    assertThat(hub.found().status(), is(200));
  }

  @Test
  void readsPlainFilesAndErrors() {
    BasicHttpResponse file = new BasicHttpResponse(HttpVersion.HTTP_1_1, 200, "OK");
    file.addHeader("ETag", "W/\"0123\"");
    file.addHeader("Content-Length", "66");
    HubResponse hub = HubResponse.of(file);
    assertThat(hub.linked(), is(false));
    assertThat(hub.etag(), is("0123"));
    assertThat(hub.size(), is(66L));
    assertThat(hub.commit(), is(nullValue()));

    BasicHttpResponse missing = new BasicHttpResponse(HttpVersion.HTTP_1_1, 404, "Not Found");
    missing.addHeader("X-Error-Code", "RepoNotFound");
    missing.addHeader("X-Error-Message", "Repository not found");
    HubResponse error = HubResponse.of(missing);
    assertThat(error.errorCode(), is("RepoNotFound"));
    assertThat(error.errorMessage(), is("Repository not found"));
    assertThat(error.size(), is(-1L));
  }

  @Test
  void normalizesEtags() {
    assertThat(HubResponse.normalizeEtag("W/\"a\""), is("a"));
    assertThat(HubResponse.normalizeEtag(" \"b\" "), is("b"));
    assertThat(HubResponse.normalizeEtag("c"), is("c"));
    assertThat(HubResponse.normalizeEtag("\"\""), is(nullValue()));
  }
}
