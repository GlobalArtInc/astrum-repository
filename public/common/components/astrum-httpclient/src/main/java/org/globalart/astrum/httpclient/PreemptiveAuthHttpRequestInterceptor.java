
package org.globalart.astrum.httpclient;

import java.io.IOException;

import org.apache.http.HttpException;
import org.apache.http.HttpHost;
import org.apache.http.HttpRequest;
import org.apache.http.HttpRequestInterceptor;
import org.apache.http.auth.AuthScope;
import org.apache.http.auth.AuthState;
import org.apache.http.auth.Credentials;
import org.apache.http.client.CredentialsProvider;
import org.apache.http.client.protocol.HttpClientContext;
import org.apache.http.impl.auth.BasicScheme;
import org.apache.http.protocol.HttpContext;

/**
 * {@link HttpRequestInterceptor} to force use of preemptive authentication.
 *
 * Use with caution, as this might provide leak of sensitive information! Usually, this should be used to access
 * trusted but protected HTTP services (ie. internal services, within same firewall area, "in house").
 *
 * @since 2.8
 */
public class PreemptiveAuthHttpRequestInterceptor
    implements HttpRequestInterceptor
{
  @Override
  public void process(final HttpRequest request, final HttpContext context) throws HttpException, IOException {
    HttpClientContext clientContext = HttpClientContext.adapt(context);
    AuthState authState = clientContext.getTargetAuthState();
    if (authState.getAuthScheme() == null) {
      CredentialsProvider credsProvider = clientContext.getCredentialsProvider();
      HttpHost targetHost = clientContext.getTargetHost();
      Credentials creds = credsProvider.getCredentials(new AuthScope(targetHost.getHostName(), targetHost.getPort()));
      if (creds != null) {
        authState.update(new BasicScheme(), creds);
      }
    }
  }
}
