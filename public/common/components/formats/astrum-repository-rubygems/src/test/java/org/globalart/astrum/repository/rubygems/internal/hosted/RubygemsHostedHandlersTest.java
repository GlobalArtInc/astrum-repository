package org.globalart.astrum.repository.rubygems.internal.hosted;

import java.io.IOException;
import java.util.Map;

import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.payloads.StringPayload;

import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

class RubygemsHostedHandlersTest
{
  @Test
  void readsTheYankFormFromTheBody() throws IOException {
    Request request = new Request.Builder()
        .action("DELETE")
        .path("/api/v1/gems/yank")
        .payload(new StringPayload("gem_name=hello_astrum&version=1.0.0&platform=x86_64-linux",
            "application/x-www-form-urlencoded"))
        .build();

    assertThat(RubygemsHostedHandlers.form(request),
        is(Map.of("gem_name", "hello_astrum", "version", "1.0.0", "platform", "x86_64-linux")));
  }

  @Test
  void prefersQueryParameters() throws IOException {
    Request request = new Request.Builder()
        .action("DELETE")
        .path("/api/v1/gems/yank")
        .parameter("gem_name", "from%20query")
        .parameter("version", "2.0")
        .build();

    assertThat(RubygemsHostedHandlers.form(request), is(Map.of("gem_name", "from%20query", "version", "2.0")));
  }
}
