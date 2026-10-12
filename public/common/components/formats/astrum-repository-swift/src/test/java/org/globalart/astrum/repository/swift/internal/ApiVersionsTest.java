package org.globalart.astrum.repository.swift.internal;

import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

class ApiVersionsTest
{
  @Test
  void acceptsVersionOneAndOtherMediaTypes() {
    assertThat(ApiVersions.check(null).isPresent(), is(false));
    assertThat(ApiVersions.check("application/vnd.swift.registry.v1+json").isPresent(), is(false));
    assertThat(ApiVersions.check("application/vnd.swift.registry.v1+zip").isPresent(), is(false));
    assertThat(ApiVersions.check("application/vnd.swift.registry.v1+swift").isPresent(), is(false));
    assertThat(ApiVersions.check("application/vnd.swift.registry.v1").isPresent(), is(false));
    assertThat(ApiVersions.check("application/vnd.swift.registry").isPresent(), is(false));
    assertThat(ApiVersions.check("text/html, */*;q=0.8").isPresent(), is(false));
  }

  @Test
  void refusesOtherVersions() {
    assertThat(ApiVersions.check("application/vnd.swift.registry.v2+json").orElseThrow().getStatus(), is(415));
    assertThat(ApiVersions.check("application/vnd.swift.registry.vx+json").orElseThrow().getStatus(), is(400));
    assertThat(ApiVersions.check("application/vnd.swift.registry.v+json").orElseThrow().getStatus(), is(400));
  }
}
