package org.globalart.astrum.repository.cargo.internal.security;

import jakarta.servlet.http.HttpServletRequest;

import org.junit.Test;

import static org.hamcrest.CoreMatchers.equalTo;
import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.CoreMatchers.nullValue;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

public class CargoTokenTest
{
  private final CargoToken token = new CargoToken();

  private String extract(final String header) {
    HttpServletRequest request = mock(HttpServletRequest.class);
    when(request.getHeader("Authorization")).thenReturn(header);
    return token.extract(request);
  }

  @Test
  public void extractsRawCargoTokens() {
    assertThat(extract("CargoToken.0a1b2c3d"), is(equalTo("0a1b2c3d")));
  }

  @Test
  public void extractsBearerCargoTokens() {
    assertThat(extract("Bearer CargoToken.0a1b2c3d"), is(equalTo("0a1b2c3d")));
  }

  @Test
  public void ignoresOtherCredentials() {
    assertThat(extract(null), is(nullValue()));
    assertThat(extract("Basic YWRtaW46YWRtaW4="), is(nullValue()));
    assertThat(extract("Bearer NpmToken.0a1b2c3d"), is(nullValue()));
    assertThat(extract("CargoToken."), is(nullValue()));
    assertThat(extract("CargoToken.a b"), is(nullValue()));
  }
}
