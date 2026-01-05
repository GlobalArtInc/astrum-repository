
package org.globalart.astrum.security.token;

import jakarta.servlet.http.HttpServletRequest;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static com.google.common.net.HttpHeaders.AUTHORIZATION;
import static java.util.UUID.randomUUID;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.core.Is.is;
import static org.hamcrest.core.IsEqual.equalTo;
import static org.hamcrest.core.IsNull.nullValue;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class BearerTokenTest
{
  private static final String TOKEN = randomUUID().toString();

  private static final String FORMAT = "Format";

  private static final String BEARER_TOKEN = "Bearer " + FORMAT + "." + TOKEN;

  @Mock
  private HttpServletRequest request;

  private BearerToken underTest;

  @Before
  public void setup() throws Exception {
    when(request.getHeader(AUTHORIZATION)).thenReturn(BEARER_TOKEN);
    underTest = new BearerToken(FORMAT);
  }

  @Test
  public void extractToken() throws Exception {
    String token = underTest.extract(request);
    assertThat(token, is(equalTo(TOKEN)));
  }

  @Test
  public void nullIfNotBearer() throws Exception {
    when(request.getHeader(AUTHORIZATION)).thenReturn("Basic " + TOKEN);
    assertThat(underTest.extract(request), is(nullValue()));
  }

  @Test
  public void nullIfTokenNotPresent() throws Exception {
    when(request.getHeader(AUTHORIZATION)).thenReturn("Bearer ");
    assertThat(underTest.extract(request), is(nullValue()));
  }

  @Test
  public void nullIfHeaderNotPresent() throws Exception {
    when(request.getHeader(AUTHORIZATION)).thenReturn(null);
    assertThat(underTest.extract(request), is(nullValue()));
  }

  @Test
  public void nullIfFormatDoesNotMatch() throws Exception {
    when(request.getHeader(AUTHORIZATION)).thenReturn("Bearer NotFormat." + TOKEN);
    assertThat(underTest.extract(request), is(nullValue()));
  }
}
