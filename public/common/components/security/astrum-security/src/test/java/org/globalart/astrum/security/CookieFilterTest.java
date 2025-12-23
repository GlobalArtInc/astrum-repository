
package org.globalart.astrum.security;

import jakarta.servlet.FilterChain;
import jakarta.servlet.ServletResponse;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static com.google.common.net.HttpHeaders.SET_COOKIE;
import static java.util.Arrays.asList;
import static org.mockito.Mockito.anyString;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.verifyNoInteractions;
import static org.mockito.Mockito.when;

/**
 * Tests for {@link CookieFilter}.
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class CookieFilterTest
{
  private static String COOKIE_1_INSECURE = "JSESSIONID=98a766bc-bc33-4b3c-9d9f-d3bb85b0cf00; Path=/astrum; HttpOnly";

  private static String COOKIE_2_INSECURE = "simple=cookie";

  private static String COOKIE_3_SECURE =
      "NXSESSIONID=98a766bc-bc33-4b3c-9d9f-d3bb85b0cf00; Path=/astrum; HttpOnly; Secure";

  private static String COOKIE_4_SECURE = "rememberMe=deleteMe; Path=/astrum; Secure; HttpOnly;";

  @Mock
  private HttpServletResponse response;

  @Mock
  private HttpServletRequest request;

  @Mock
  private FilterChain filterChain;

  @Mock
  private ServletResponse notInstanceOfHttpServletResponse;

  private CookieFilter cookieFilter;

  @Before
  public void setupFilter() throws Exception {
    this.cookieFilter = new CookieFilter();
  }

  private void setup(HttpServletRequest request, HttpServletResponse response) {
    when(request.isSecure()).thenReturn(true);
    when(response.getHeaders(SET_COOKIE)).thenReturn(asList(COOKIE_1_INSECURE, COOKIE_2_INSECURE, COOKIE_3_SECURE,
        COOKIE_4_SECURE));
  }

  private void verifyResponse(HttpServletResponse response) {
    verify(response).setHeader(SET_COOKIE, COOKIE_1_INSECURE + "; Secure");
    verify(response).addHeader(SET_COOKIE, COOKIE_2_INSECURE + "; Secure");
    verify(response).addHeader(SET_COOKIE, COOKIE_3_SECURE);
    verify(response).addHeader(SET_COOKIE, COOKIE_4_SECURE);
  }

  @Test
  public void ifRequestIsSecureSetSecureCookieAttributeWhenMissingBeforeProcessingFilterchain() throws Exception {
    setup(request, response);
    cookieFilter.preHandle(request, response);
    verifyResponse(response);
  }

  @Test
  public void ifRequestIsSecureSetSecureCookieAttributeWhenMissingAfterProcessingFilterchain() throws Exception {
    setup(request, response);
    cookieFilter.postHandle(request, response);
    verifyResponse(response);
  }

  @Test
  public void ifRequestIsNotSecureDoNotChangeCookieAttributes() throws Exception {
    when(request.isSecure()).thenReturn(false);
    when(response.getHeaders(SET_COOKIE))
        .thenReturn(asList(COOKIE_1_INSECURE, COOKIE_3_SECURE));

    cookieFilter.doFilter(request, response, filterChain);

    verifyNoInteractions(response);
  }

  @Test
  public void ifNotInstanceOfHttpServletResponseDoNotProcessResponse() throws Exception {
    cookieFilter.doFilter(request, notInstanceOfHttpServletResponse, filterChain);

    verifyNoInteractions(response);
  }

  @Test
  public void ifNoCookieHeadersDoNotChangeTheResponse() throws Exception {
    when(request.isSecure()).thenReturn(true);

    cookieFilter.doFilter(request, response, filterChain);

    verify(response, times(0)).setHeader(anyString(), anyString());
    verify(response, times(0)).addHeader(anyString(), anyString());
  }
}
