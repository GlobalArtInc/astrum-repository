package org.globalart.astrum.repository.rubygems.internal.security;

import jakarta.servlet.http.HttpServletRequest;

import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

class RubygemsApiKeyTest
{
  private final RubygemsApiKey underTest = new RubygemsApiKey();

  private static HttpServletRequest request(final String authorization) {
    HttpServletRequest request = mock(HttpServletRequest.class);
    when(request.getHeader("Authorization")).thenReturn(authorization);
    return request;
  }

  @Test
  void extractsTheKeyGemSendsAsItIs() {
    assertThat(underTest.extract(request("RubyGemsApiKey.0123-abcd")), is("0123-abcd"));
    assertThat(underTest.extract(request("Bearer RubyGemsApiKey.0123-abcd")), is("0123-abcd"));
  }

  @Test
  void ignoresOtherCredentials() {
    assertThat(underTest.extract(request(null)), is(nullValue()));
    assertThat(underTest.extract(request("Basic dXNlcjpwYXNz")), is(nullValue()));
    assertThat(underTest.extract(request("CargoToken.0123")), is(nullValue()));
    assertThat(underTest.extract(request("RubyGemsApiKey.")), is(nullValue()));
    assertThat(underTest.extract(request("RubyGemsApiKey.a b")), is(nullValue()));
  }
}
