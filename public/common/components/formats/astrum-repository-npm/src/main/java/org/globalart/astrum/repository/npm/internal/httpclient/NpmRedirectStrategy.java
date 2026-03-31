
package org.globalart.astrum.repository.npm.internal.httpclient;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import java.net.URI;


import org.globalart.astrum.repository.npm.internal.NpmFormat;

import org.apache.http.HttpHeaders;
import org.apache.http.HttpRequest;
import org.apache.http.HttpResponse;
import org.apache.http.ProtocolException;
import org.apache.http.client.methods.HttpUriRequest;
import org.apache.http.impl.client.DefaultRedirectStrategy;
import org.apache.http.protocol.HttpContext;

/**
 * ASTRUM-23750: Support npm GitHub Package Registry.
 * GitHub Package Repository uses a HTTP 302 response + Location header to tell npm CLI, that tarball is on an AWS server.
 * AWS does not allow to use both the Authorization Bearer token and the X-Amz-Credential parameter.
 *
 * @since 3.next
 */
@Component
@Qualifier(NpmFormat.NAME)
public class NpmRedirectStrategy
    extends DefaultRedirectStrategy
{
  private static final String AMAZON_CREDENTIAL_PARAM = "X-Amz-Credential";

  @Override
  public HttpUriRequest getRedirect(
      final HttpRequest httpRequest, final HttpResponse httpResponse, final HttpContext httpContext)
      throws ProtocolException
  {
    boolean isRedirected = isRedirected(httpRequest, httpResponse, httpContext);
    if (isRedirected) {
      URI locationURI = createLocationURI(httpResponse.getFirstHeader(HttpHeaders.LOCATION).getValue());
      String query = locationURI.getQuery();
      if (query != null && query.contains(AMAZON_CREDENTIAL_PARAM)) {
        httpRequest.removeHeaders(HttpHeaders.AUTHORIZATION);
      }
    }
    return super.getRedirect(httpRequest, httpResponse, httpContext);
  }
}
