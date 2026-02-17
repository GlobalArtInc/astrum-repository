
package org.globalart.astrum.repository.httpbridge.internal;

import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;

import org.junit.Test;
import org.mockito.Mock;
import org.mockito.verification.VerificationMode;

import static org.apache.http.HttpHeaders.USER_AGENT;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.reset;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.PUT;
import static org.globalart.astrum.repository.http.HttpStatus.BAD_REQUEST;
import static org.globalart.astrum.repository.http.HttpStatus.OK;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

/**
 * Tests for {@link ExhaustRequestFilter}
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class ExhaustRequestFilterTest
{
  private static final String PIPE_DELIMITED_MATCHING_PATTERN = "Apache-Maven.*|Apache Ivy.*";

  private static final String UNFORTUNATELY_SUPPORTED_COMMA_DELIMITED_MATCHING_PATTERN =
      "Apache-Maven.*\\s,\\sApache Ivy.*";

  private static final String NULL_USER_AGENT = null;

  @Mock
  HttpServletRequest request;

  @Mock
  HttpServletResponse response;

  @Mock
  jakarta.servlet.FilterChain filterChain;

  @Test
  public void httpOkResponse() throws Exception {
    verifyRequestNotExhausted(OK, PUT, "Apache-Maven.Foo");
  }

  @Test
  public void http400GetResponse() throws Exception {
    verifyRequestNotExhausted(BAD_REQUEST, GET, "Apache-Maven.Foo");
  }

  @Test
  public void http400PutResponse_NullUserAgent() throws Exception {
    verifyRequestNotExhausted(BAD_REQUEST, PUT, NULL_USER_AGENT);
  }

  @Test
  public void http400PutResponse_NonMavenOrIvyUserAgent() throws Exception {
    verifyRequestNotExhausted(BAD_REQUEST, PUT, "notmavenorivy");
  }

  @Test
  public void http400PutResponse_MavenUserAgent() throws Exception {
    verifyRequestExhausted(BAD_REQUEST, PUT, "Apache-Maven.Foo");
  }

  @Test
  public void http400PutResponse_IvyUserAgent() throws Exception {
    verifyRequestExhausted(BAD_REQUEST, PUT, "Apache Ivy.Foo");
  }

  private void verifyRequestNotExhausted(
      final Integer status,
      final String method,
      final String userAgent) throws Exception
  {
    verifyRequestExhaustionStatus(status, method, userAgent, never());
  }

  private void verifyRequestExhausted(
      final Integer status,
      final String method,
      final String userAgent) throws Exception
  {
    verifyRequestExhaustionStatus(status, method, userAgent, times(1));
  }

  private void verifyRequestExhaustionStatus(
      final Integer status,
      final String method,
      final String userAgent,
      final VerificationMode verificationMode) throws Exception
  {
    doVerifyRequestExhaustionStatus(status, method, userAgent, PIPE_DELIMITED_MATCHING_PATTERN, verificationMode);
    doVerifyRequestExhaustionStatus(
        status, method, userAgent, UNFORTUNATELY_SUPPORTED_COMMA_DELIMITED_MATCHING_PATTERN, verificationMode);
  }

  private void doVerifyRequestExhaustionStatus(
      final Integer status,
      final String method,
      final String userAgent,
      final String exhaustForAgents,
      final VerificationMode verificationMode) throws Exception
  {
    setupMockResponse(status, method, userAgent);
    new ExhaustRequestFilter(exhaustForAgents).doFilter(request, response, filterChain);
    verify(request, verificationMode).getInputStream();
  }

  private void setupMockResponse(final Integer status, final String method, final String userAgent) {
    reset(request, response);
    when(response.getStatus()).thenReturn(status);
    when(request.getMethod()).thenReturn(method);
    when(request.getHeader(eq(USER_AGENT))).thenReturn(userAgent);
  }
}
