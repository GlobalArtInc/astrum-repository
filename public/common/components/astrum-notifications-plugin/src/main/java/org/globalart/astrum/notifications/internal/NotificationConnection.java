package org.globalart.astrum.notifications.internal;

import java.io.IOException;
import java.net.ConnectException;
import java.net.UnknownHostException;
import java.util.List;

import javax.annotation.Nullable;

import org.globalart.astrum.httpclient.HttpClientManager;
import org.globalart.astrum.httpclient.HttpClientPlan;

import org.apache.http.HttpHost;
import org.apache.http.HttpRequest;
import org.apache.http.HttpResponse;
import org.apache.http.auth.AuthScope;
import org.apache.http.auth.UsernamePasswordCredentials;
import org.apache.http.client.config.AuthSchemes;
import org.apache.http.client.protocol.HttpClientContext;
import org.apache.http.config.Registry;
import org.apache.http.config.RegistryBuilder;
import org.apache.http.conn.ConnectTimeoutException;
import org.apache.http.conn.socket.ConnectionSocketFactory;
import org.apache.http.conn.socket.LayeredConnectionSocketFactory;
import org.apache.http.conn.socket.PlainConnectionSocketFactory;
import org.apache.http.impl.client.BasicCredentialsProvider;
import org.apache.http.impl.conn.BasicHttpClientConnectionManager;
import org.apache.http.impl.conn.DefaultProxyRoutePlanner;
import org.apache.http.impl.conn.DefaultRoutePlanner;
import org.apache.http.impl.conn.DefaultSchemePortResolver;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.httpclient.HttpSchemes.HTTP;
import static org.globalart.astrum.httpclient.HttpSchemes.HTTPS;

/**
 * How the messages of one channel reach the chat, applied to the HTTP clients that send them after the global HTTP
 * settings, which it therefore overrides: the global settings as they are, a direct connection past the global proxy,
 * or the channel's own HTTP or SOCKS5 proxy. The global timeouts, retries and user agent apply in every case, and the
 * request paths, which carry bot tokens and webhook secrets, are kept out of the outbound request log.
 *
 * <p>
 * A channel's own proxy gets connections of its own, so that a tunnel opened with one channel's credentials is never
 * reused by a client with other credentials or none. Its credentials are given to each request alone, never to the
 * client's defaults, where the global settings may have put others. An HTTP proxy is asked for HTTPS targets with
 * CONNECT and answers Basic or Digest authentication; a SOCKS5 proxy resolves the target's host name itself.
 */
public class NotificationConnection
    implements HttpClientPlan.Customizer
{
  private final NotificationChannel channel;

  @Nullable
  private final String proxyPassword;

  private final LayeredConnectionSocketFactory tls;

  public NotificationConnection(
      final NotificationChannel channel,
      @Nullable final String proxyPassword,
      final LayeredConnectionSocketFactory tls)
  {
    this.channel = checkNotNull(channel);
    this.proxyPassword = proxyPassword;
    this.tls = checkNotNull(tls);
  }

  @Override
  public void customize(final HttpClientPlan plan) {
    plan.getAttributes().put(HttpClientManager.REDACT_REQUEST_PATH, true);
    if (channel.isDirect()) {
      plan.getClient().setRoutePlanner(new DefaultRoutePlanner(DefaultSchemePortResolver.INSTANCE));
    }
    else if (channel.isCustomProxy()) {
      BasicHttpClientConnectionManager connections = channel.isSocksProxy()
          ? new BasicHttpClientConnectionManager(socksSockets(), null, null, Socks5ConnectionSocketFactory.REMOTE_DNS)
          : new BasicHttpClientConnectionManager(plainSockets());
      connections.setSocketConfig(plan.getSocket().build());
      connections.setConnectionConfig(plan.getConnection().build());
      plan.getClient().setConnectionManager(connections);
      plan.getClient().setConnectionManagerShared(false);
      if (channel.isSocksProxy()) {
        plan.getClient().setRoutePlanner(new DefaultRoutePlanner(DefaultSchemePortResolver.INSTANCE));
      }
      else {
        plan.getClient().setRoutePlanner(new DefaultProxyRoutePlanner(proxy()));
        plan.getRequest().setProxyPreferredAuthSchemes(List.of(AuthSchemes.BASIC, AuthSchemes.DIGEST));
      }
    }
  }

  /**
   * @return the context to execute a request of this channel in
   */
  public HttpClientContext context() {
    HttpClientContext context = HttpClientContext.create();
    if (channel.isCustomProxy()) {
      BasicCredentialsProvider credentials = new BasicCredentialsProvider();
      if (isHttpProxy() && channel.getProxyUsername() != null) {
        credentials.setCredentials(new AuthScope(proxy()),
            new UsernamePasswordCredentials(channel.getProxyUsername(), proxyPassword));
      }
      context.setCredentialsProvider(credentials);
    }
    return context;
  }

  /**
   * Tells why a request failed when it did not reach the chat.
   *
   * @param service the chat as messages name it, e.g. {@code Telegram}
   */
  public NotificationDeliveryException unreachable(final String service, final IOException e) {
    ProxyConnectException refused = cause(e, ProxyConnectException.class);
    if (refused != null) {
      return new NotificationDeliveryException(refused.getMessage(), 0, refused.isRetryable(), 0);
    }
    if (isHttpProxy()) {
      String address = channel.getProxyHost() + ':' + channel.getProxyPort();
      if (e instanceof ConnectTimeoutException) {
        return new NotificationDeliveryException("The proxy did not answer in time: " + address, 0, true, 0);
      }
      if (e instanceof UnknownHostException) {
        return new NotificationDeliveryException("The proxy host " + channel.getProxyHost() + " is unknown", 0, true,
            0);
      }
      if (e instanceof ConnectException) {
        Throwable reason = e.getCause() != null ? e.getCause() : e;
        return new NotificationDeliveryException(
            "The proxy refused the connection: " + address + " (" + reason.getMessage() + ")", 0, true, 0);
      }
    }
    String through = channel.isCustomProxy() ? " through the proxy" : "";
    return new NotificationDeliveryException("Could not reach " + service + through + ": " + e.getMessage(), 0, true,
        0);
  }

  /**
   * Tells when the answer to a request came from the channel's own HTTP proxy rather than from the chat: the proxy
   * asked for credentials, or it refused to open a tunnel to the chat.
   *
   * @param target the chat's address as messages name it, e.g. {@code api.telegram.org}
   * @return the failure, or {@code null} when the chat answered
   */
  @Nullable
  public NotificationDeliveryException refusal(
      final HttpResponse response,
      final HttpClientContext context,
      final String target)
  {
    if (!isHttpProxy()) {
      return null;
    }
    int status = response.getStatusLine().getStatusCode();
    String statusLine = status + " " + response.getStatusLine().getReasonPhrase();
    if (status == 407) {
      return NotificationDeliveryException.permanent(channel.getProxyUsername() != null
          ? "The proxy rejected the username or password (" + statusLine + ")"
          : "The proxy requires a username and password (" + statusLine + ")");
    }
    HttpRequest request = context.getRequest();
    if (request != null && "CONNECT".equalsIgnoreCase(request.getRequestLine().getMethod())) {
      return new NotificationDeliveryException("The proxy refused the connection to " + target + ": " + statusLine,
          status, status >= 500, 0);
    }
    return null;
  }

  private boolean isHttpProxy() {
    return channel.isCustomProxy() && !channel.isSocksProxy();
  }

  private HttpHost proxy() {
    return new HttpHost(channel.getProxyHost(), channel.getProxyPort());
  }

  private Registry<ConnectionSocketFactory> plainSockets() {
    return RegistryBuilder.<ConnectionSocketFactory>create()
        .register(HTTP, PlainConnectionSocketFactory.getSocketFactory())
        .register(HTTPS, tls)
        .build();
  }

  private Registry<ConnectionSocketFactory> socksSockets() {
    String host = channel.getProxyHost();
    int port = channel.getProxyPort();
    String username = channel.getProxyUsername();
    return RegistryBuilder.<ConnectionSocketFactory>create()
        .register(HTTP, new Socks5ConnectionSocketFactory(host, port, username, proxyPassword, null))
        .register(HTTPS, new Socks5ConnectionSocketFactory(host, port, username, proxyPassword, tls))
        .build();
  }

  @Nullable
  private static <T extends Throwable> T cause(final Throwable e, final Class<T> type) {
    for (Throwable cause = e; cause != null; cause = cause.getCause()) {
      if (type.isInstance(cause)) {
        return type.cast(cause);
      }
      if (cause.getCause() == cause) {
        return null;
      }
    }
    return null;
  }
}
