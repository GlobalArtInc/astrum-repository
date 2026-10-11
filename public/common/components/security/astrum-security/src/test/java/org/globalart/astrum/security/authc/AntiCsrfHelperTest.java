
package org.globalart.astrum.security.authc;

import jakarta.servlet.http.Cookie;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.ws.rs.HttpMethod;

import com.google.common.net.HttpHeaders;
import org.apache.shiro.mgt.SecurityManager;
import org.apache.shiro.session.Session;
import org.apache.shiro.subject.Subject;
import org.apache.shiro.util.ThreadContext;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.lenient;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.reset;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.verifyNoInteractions;
import static org.mockito.Mockito.verifyNoMoreInteractions;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class AntiCsrfHelperTest
{
  private AntiCsrfHelper underTest;

  @Mock
  SecurityManager securityManager;

  @Mock
  HttpServletRequest httpServletRequest;

  @Mock
  Subject subject;

  @BeforeEach
  void setup() {
    underTest = new AntiCsrfHelper(true, true);
    lenient().when(httpServletRequest.getRequestURI()).thenReturn("/somepath");
    lenient().when(httpServletRequest.getHeader(HttpHeaders.SEC_FETCH_SITE)).thenReturn(null);

    ThreadContext.bind(securityManager);
    ThreadContext.bind(subject);
  }

  @AfterEach
  void teardown() {
    ThreadContext.unbindSubject();
    ThreadContext.unbindSecurityManager();
  }

  /*
   * Test that the filter passes requests when disabled
   */
  @Test
  void testIsAccessAllowed_Disabled() {
    underTest = new AntiCsrfHelper(false, true);
    assertThat(underTest.isAccessAllowed(httpServletRequest), is(true));
    verifyNoInteractions(httpServletRequest);
  }

  /*
   * Test that the filter allows requests with 'safe' HTTP methods without a token
   */
  @Test
  void testIsAccessAllowed_SafeMethodsAllowed() {
    when(httpServletRequest.getMethod()).thenReturn(HttpMethod.HEAD);
    assertThat(underTest.isAccessAllowed(httpServletRequest), is(true));

    when(httpServletRequest.getMethod()).thenReturn(HttpMethod.GET);
    assertThat(underTest.isAccessAllowed(httpServletRequest), is(true));
  }

  /*
   * Test that the filter blocks requests with 'unsafe' HTTP methods without a token
   */
  @Test
  void testIsAccessAllowed_UnsafeMethodsBlocked() {
    setupBrowserSubject();
    when(httpServletRequest.getMethod()).thenReturn(HttpMethod.POST);
    assertThat(underTest.isAccessAllowed(httpServletRequest), is(false));

    when(httpServletRequest.getMethod()).thenReturn(HttpMethod.PUT);
    assertThat(underTest.isAccessAllowed(httpServletRequest), is(false));

    when(httpServletRequest.getMethod()).thenReturn(HttpMethod.DELETE);
    assertThat(underTest.isAccessAllowed(httpServletRequest), is(false));
  }

  /*
   * Test the behaviour without a User-Agent or with a non-browser User-Agent.
   * clients.
   */
  @Test
  void testIsAccessAllowed_NotBrowser() {
    when(httpServletRequest.getMethod()).thenReturn(HttpMethod.POST);
    assertThat(underTest.isAccessAllowed(httpServletRequest), is(true));

    when(httpServletRequest.getMethod()).thenReturn(HttpMethod.POST);
    setupClientSubject();
    assertThat(underTest.isAccessAllowed(httpServletRequest), is(true));
  }

  /*
   * PowerShell inexplicably includes a UserAgent pretending to be a browser, this ensures our whitelist allows it.
   */
  @Test
  void testIsAccessAllowed_PowerShell() {
    when(httpServletRequest.getMethod()).thenReturn(HttpMethod.POST);
    assertThat(underTest.isAccessAllowed(httpServletRequest), is(true));

    when(httpServletRequest.getMethod()).thenReturn(HttpMethod.POST);
    lenient().when(httpServletRequest.getHeader(HttpHeaders.USER_AGENT))
        .thenReturn("Mozilla/5.0 (Windows NT; Windows NT 10.0; en-CA) WindowsPowerShell/5.1.17134.590");
    assertThat(underTest.isAccessAllowed(httpServletRequest), is(true));
  }

  @Test
  void shouldAllowAccessWhenMissingSubject() {
    ThreadContext.unbindSubject();
    underTest = new AntiCsrfHelper(true, true);

    assertThat(underTest.isAccessAllowed(httpServletRequest), is(true));
  }

  /*
   * Test that a request with a valid CSRF token is allowed
   */
  @Test
  void testIsAccessAllowed_ValidCsrfToken() {
    when(httpServletRequest.getMethod()).thenReturn(HttpMethod.POST);
    setupBrowserSubject();
    when(httpServletRequest.getHeader("AS-ANTI-CSRF-TOKEN")).thenReturn("avalue");
    when(httpServletRequest.getCookies()).thenReturn(new Cookie[]{new Cookie("AS-ANTI-CSRF-TOKEN", "avalue")});

    assertThat(underTest.isAccessAllowed(httpServletRequest), is(true));

    // simple validation, we expect the code to access the cookies once
    verify(httpServletRequest, times(1)).getCookies();
  }

  /*
   * Test that a request missing a CSRF cookie but with a header is rejected
   */
  @Test
  void testIsAccessAllowed_MissingCsrfCookie() {
    when(httpServletRequest.getMethod()).thenReturn(HttpMethod.POST);
    setupBrowserSubject();
    when(httpServletRequest.getHeader("AS-ANTI-CSRF-TOKEN")).thenReturn("avalue");

    assertThat(underTest.isAccessAllowed(httpServletRequest), is(false));

    // simple validation, we expect the code to access the cookies once
    verify(httpServletRequest, times(1)).getCookies();
  }

  /*
   * Test that a request missing a CSRF header but with a cookie is rejected
   */
  @Test
  void testIsAccessAllowed_MissingCsrfHeader() {
    setupBrowserSubject();
    when(httpServletRequest.getMethod()).thenReturn(HttpMethod.POST);
    // AS-ANTI-CSRF-TOKEN header not set

    assertThat(underTest.isAccessAllowed(httpServletRequest), is(false));

    // without a token there is nothing to compare, so the cookies are not read
    verify(httpServletRequest, never()).getCookies();
  }

  /*
   * Test that a request with mismatched CSRF tokens is rejected
   */
  @Test
  void testIsAccessAllowed_MismatchedCsrfToken() {
    setupBrowserSubject();
    when(httpServletRequest.getMethod()).thenReturn(HttpMethod.POST);
    when(httpServletRequest.getHeader("AS-ANTI-CSRF-TOKEN")).thenReturn("some-value");
    when(httpServletRequest.getCookies())
        .thenReturn(new Cookie[]{new Cookie("AS-ANTI-CSRF-TOKEN", "some-other-value")});

    assertThat(underTest.isAccessAllowed(httpServletRequest), is(false));

    // simple validation, we expect the code to access the cookies once
    verify(httpServletRequest, times(1)).getCookies();
  }

  /*
   * Test that a token matching any of several CSRF cookies of the same name is allowed
   */
  @Test
  void testIsAccessAllowed_TokenMatchingOneOfSeveralCookies() {
    setupBrowserSubject();
    when(httpServletRequest.getMethod()).thenReturn(HttpMethod.POST);
    when(httpServletRequest.getHeader("AS-ANTI-CSRF-TOKEN")).thenReturn("current");
    when(httpServletRequest.getCookies()).thenReturn(new Cookie[]{
        new Cookie("AS-ANTI-CSRF-TOKEN", "stale"),
        new Cookie("ASTRUMSESSIONID", "current"),
        new Cookie("AS-ANTI-CSRF-TOKEN", "current")});

    assertThat(underTest.isAccessAllowed(httpServletRequest), is(true));
  }

  /*
   * Test that a token matching none of several CSRF cookies, or only a cookie of another name, is rejected
   */
  @Test
  void testIsAccessAllowed_TokenMatchingNoneOfSeveralCookies() {
    setupBrowserSubject();
    when(httpServletRequest.getMethod()).thenReturn(HttpMethod.POST);
    when(httpServletRequest.getHeader("AS-ANTI-CSRF-TOKEN")).thenReturn("guess");
    when(httpServletRequest.getCookies()).thenReturn(new Cookie[]{
        new Cookie("AS-ANTI-CSRF-TOKEN", "one"),
        new Cookie("ASTRUMSESSIONID", "guess"),
        new Cookie("AS-ANTI-CSRF-TOKEN", "two")});

    assertThat(underTest.isAccessAllowed(httpServletRequest), is(false));
  }

  /*
   * Test that an empty token is rejected even when the cookie is empty too
   */
  @Test
  void testIsAccessAllowed_EmptyTokenAndCookie() {
    setupBrowserSubject();
    when(httpServletRequest.getMethod()).thenReturn(HttpMethod.POST);
    when(httpServletRequest.getHeader("AS-ANTI-CSRF-TOKEN")).thenReturn("");
    lenient().when(httpServletRequest.getCookies()).thenReturn(new Cookie[]{new Cookie("AS-ANTI-CSRF-TOKEN", "")});

    assertThat(underTest.isAccessAllowed(httpServletRequest), is(false));
  }

  /*
   * Verify variations of Sec-Fetch-Site headers
   */
  @Test
  void testIsCrossSite() {
    when(httpServletRequest.getHeader(HttpHeaders.SEC_FETCH_SITE)).thenReturn(null);
    assertFalse(underTest.isCrossSiteRequest(httpServletRequest), "Missing header not considered cross-site");

    // same hostname
    when(httpServletRequest.getHeader(HttpHeaders.SEC_FETCH_SITE)).thenReturn("same-origin");
    assertFalse(underTest.isCrossSiteRequest(httpServletRequest));

    // user entered the address manually
    when(httpServletRequest.getHeader(HttpHeaders.SEC_FETCH_SITE)).thenReturn("none");
    assertFalse(underTest.isCrossSiteRequest(httpServletRequest));

    // same-site -> this would allow other subdomains
    when(httpServletRequest.getHeader(HttpHeaders.SEC_FETCH_SITE)).thenReturn("same-site");
    assertTrue(underTest.isCrossSiteRequest(httpServletRequest));

    when(httpServletRequest.getHeader(HttpHeaders.SEC_FETCH_SITE)).thenReturn("cross-site");
    assertTrue(underTest.isCrossSiteRequest(httpServletRequest));

    // Fail CSRF when cross-site
    reset(httpServletRequest);
    when(httpServletRequest.getHeader(HttpHeaders.SEC_FETCH_SITE)).thenReturn("cross-site");
    when(httpServletRequest.getMethod()).thenReturn(HttpMethod.POST);

    assertThat(underTest.isAccessAllowed(httpServletRequest), is(false));

    verify(httpServletRequest).getMethod();
    verify(httpServletRequest, times(2)).getHeader(HttpHeaders.SEC_FETCH_SITE);
    verifyNoMoreInteractions(httpServletRequest);
  }

  /*
   * Disabled usage of Sec-Fetch-Site header
   */
  @Test
  void testIsCrossSite_disabled() {
    underTest = new AntiCsrfHelper(true, false);

    lenient().when(httpServletRequest.getHeader(HttpHeaders.SEC_FETCH_SITE)).thenReturn(null);
    assertFalse(underTest.isCrossSiteRequest(httpServletRequest), "Missing header not considered cross-site");

    lenient().when(httpServletRequest.getHeader(HttpHeaders.SEC_FETCH_SITE)).thenReturn("same-site");
    assertFalse(underTest.isCrossSiteRequest(httpServletRequest));

    lenient().when(httpServletRequest.getHeader(HttpHeaders.SEC_FETCH_SITE)).thenReturn("same-origin");
    assertFalse(underTest.isCrossSiteRequest(httpServletRequest));

    lenient().when(httpServletRequest.getHeader(HttpHeaders.SEC_FETCH_SITE)).thenReturn("cross-site");
    assertFalse(underTest.isCrossSiteRequest(httpServletRequest));
  }

  private void setupBrowserSubject() {
    Session session = mock(Session.class);
    when(subject.getSession(false)).thenReturn(session);
  }

  private void setupClientSubject() {
    when(subject.getSession(false)).thenReturn(null);
  }
}
