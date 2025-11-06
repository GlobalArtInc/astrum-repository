
package org.globalart.astrum.bootstrap.jetty;

import org.eclipse.jetty.client.ContentResponse;
import org.eclipse.jetty.client.HttpClient;
import org.eclipse.jetty.http.HttpStatus;
import org.eclipse.jetty.http2.client.HTTP2Client;
import org.eclipse.jetty.http2.client.transport.HttpClientTransportOverHTTP2;
import org.eclipse.jetty.http2.server.HTTP2CServerConnectionFactory;
import org.eclipse.jetty.server.Handler;
import org.eclipse.jetty.server.HttpConfiguration;
import org.eclipse.jetty.server.Request;
import org.eclipse.jetty.server.Response;
import org.eclipse.jetty.server.Server;
import org.eclipse.jetty.server.ServerConnector;
import org.eclipse.jetty.util.Callback;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

/**
 * STL-476: HTTP/2 has no status-line reason phrase, so {@link AstrumReasonPhraseCustomizer} cannot (and does not)
 * surface a custom reason phrase on h2. This test pins the other half of the contract that still matters on h2:
 * the internal {@link AstrumReasonPhraseCustomizer#REASON_PHRASE_HEADER} sentinel must still be stripped so it
 * never leaks to the client.
 *
 * <p>
 * The wrapper is installed per request regardless of protocol and the strip in {@code send(...)} is shared with
 * HTTP/1.1, but {@link AstrumReasonPhraseCustomizerTest} only exercises HTTP/1.1. This drives a real h2c request so
 * a future change that made the strip conditional on protocol (leaking the sentinel over HTTP/2) would fail here.
 */
class AstrumReasonPhraseCustomizerHttp2Test
{
  private Server server;

  private int port;

  @BeforeEach
  void startServer() throws Exception {
    HttpConfiguration httpConfig = new HttpConfiguration();
    httpConfig.addCustomizer(new AstrumReasonPhraseCustomizer());

    server = new Server();
    // h2c (cleartext HTTP/2) connector, so no TLS setup is needed for the test.
    ServerConnector connector = new ServerConnector(server, new HTTP2CServerConnectionFactory(httpConfig));
    connector.setPort(0); // random free port
    server.addConnector(connector);
    server.setHandler(new Handler.Abstract()
    {
      @Override
      public boolean handle(final Request request, final Response response, final Callback callback) {
        response.getHeaders().put(AstrumReasonPhraseCustomizer.REASON_PHRASE_HEADER, "package-quarantined");
        response.setStatus(HttpStatus.CONFLICT_409);
        response.write(true, null, callback);
        return true;
      }
    });
    server.start();
    port = connector.getLocalPort();
  }

  @AfterEach
  void stopServer() throws Exception {
    if (server != null) {
      server.stop();
      server = null;
    }
  }

  @Test
  void sentinelHeaderIsStrippedOnHttp2() throws Exception {
    HttpClient httpClient = new HttpClient(new HttpClientTransportOverHTTP2(new HTTP2Client()));
    httpClient.start();
    try {
      ContentResponse response = httpClient.newRequest("http://localhost:" + port + "/").send();

      assertThat(response.getStatus(), is(HttpStatus.CONFLICT_409));
      // HTTP/2 carries no reason phrase, but the internal sentinel header must never reach the client.
      assertThat(response.getHeaders().contains(AstrumReasonPhraseCustomizer.REASON_PHRASE_HEADER), is(false));
    }
    finally {
      httpClient.stop();
    }
  }
}
