package org.globalart.astrum.notifications.internal;

import java.io.IOException;
import java.io.OutputStream;
import java.net.InetAddress;
import java.net.InetSocketAddress;
import java.net.ServerSocket;
import java.net.Socket;
import java.util.List;
import java.util.Map;
import java.util.concurrent.CopyOnWriteArrayList;

import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.common.io.ByteSize;
import org.globalart.astrum.common.time.Time;
import org.globalart.astrum.httpclient.HttpClientManager;
import org.globalart.astrum.httpclient.HttpClientPlan;
import org.globalart.astrum.httpclient.config.ProxyConfiguration;
import org.globalart.astrum.httpclient.config.ProxyServerConfiguration;
import org.globalart.astrum.internal.httpclient.DefaultsCustomizer;
import org.globalart.astrum.internal.httpclient.HttpClientConfigurationData;
import org.globalart.astrum.internal.httpclient.HttpClientConfigurationStore;
import org.globalart.astrum.internal.httpclient.HttpClientManagerImpl;
import org.globalart.astrum.internal.httpclient.SharedHttpClientConnectionManager;
import org.globalart.astrum.internal.httpclient.AstrumSSLConnectionSocketFactory;
import org.globalart.astrum.utils.httpclient.UserAgentGenerator;
import org.globalart.astrum.validation.ssrf.AntiSsrfService;

import com.sun.net.httpserver.HttpExchange;
import com.sun.net.httpserver.HttpServer;
import org.junit.After;
import org.junit.Before;
import org.junit.Test;

import static java.nio.charset.StandardCharsets.UTF_8;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.empty;
import static org.hamcrest.Matchers.everyItem;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.not;
import static org.hamcrest.Matchers.startsWith;
import static org.junit.Assert.assertThrows;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

/**
 * Sends through a real {@link HttpClientManagerImpl}, so that the channel's connection is applied after the global
 * HTTP settings exactly as on a running server, to a fake Telegram behind small local proxies.
 */
public class HttpNotificationTransportProxyTest
{
  private static final String TOKEN = "123456:ABC-def_ghi";

  private static final String PASSWORD = "s3cret pass";

  private final List<Map<String, String>> received = new CopyOnWriteArrayList<>();

  private HttpServer telegram;

  private ProxyServers.HttpProxy httpProxy;

  private ProxyServers.Socks5Proxy socksProxy;

  private HttpClientConfigurationData configuration;

  private HttpClientManagerImpl httpClientManager;

  private AntiSsrfService antiSsrfService;

  @Before
  public void setUp() throws Exception {
    telegram = HttpServer.create(new InetSocketAddress(InetAddress.getLoopbackAddress(), 0), 0);
    telegram.createContext("/", this::answer);
    telegram.start();
    httpProxy = new ProxyServers.HttpProxy();
    socksProxy = new ProxyServers.Socks5Proxy();

    configuration = new HttpClientConfigurationData();
    HttpClientConfigurationStore store = mock(HttpClientConfigurationStore.class);
    when(store.load()).thenAnswer(invocation -> configuration);
    UserAgentGenerator userAgentGenerator = mock(UserAgentGenerator.class);
    when(userAgentGenerator.generate()).thenReturn("Astrum/test");
    httpClientManager = new HttpClientManagerImpl(mock(EventManager.class), store, () -> configuration,
        new SharedHttpClientConnectionManager(List.of(), 20, 200, Time.seconds(30), Time.seconds(5), Time.seconds(2),
            Time.seconds(5)),
        new DefaultsCustomizer(userAgentGenerator, Time.seconds(2), Time.seconds(5), Time.seconds(30),
            ByteSize.kiloBytes(8), 2));
    httpClientManager.start();
    antiSsrfService = mock(AntiSsrfService.class);
  }

  @After
  public void tearDown() throws Exception {
    httpClientManager.stop();
    telegram.stop(0);
    httpProxy.close();
    socksProxy.close();
  }

  private void answer(final HttpExchange exchange) throws IOException {
    received.add(Map.of(
        "path", exchange.getRequestURI().getPath(),
        "body", new String(exchange.getRequestBody().readAllBytes(), UTF_8),
        "userAgent", String.valueOf(exchange.getRequestHeaders().getFirst("User-Agent"))));
    byte[] bytes = "{\"ok\":true}".getBytes(UTF_8);
    exchange.getResponseHeaders().add("Content-Type", "application/json");
    exchange.sendResponseHeaders(200, bytes.length);
    try (OutputStream out = exchange.getResponseBody()) {
      out.write(bytes);
    }
  }

  private HttpNotificationTransport transport(final String host) {
    return new HttpNotificationTransport(httpClientManager, antiSsrfService, List.of(),
        "http://" + host + ":" + telegram.getAddress().getPort());
  }

  private HttpNotificationTransport transport() {
    return transport("telegram.test");
  }

  private static NotificationChannel telegram(final String mode) {
    NotificationChannel channel = new NotificationChannel();
    channel.setType(NotificationChannel.TELEGRAM);
    channel.setName("releases");
    channel.setChatId("-100");
    channel.setProxyMode(mode);
    return channel;
  }

  private static NotificationChannel throughProxy(final String type, final int port, final String username) {
    NotificationChannel channel = telegram(NotificationChannel.PROXY_CUSTOM);
    channel.setProxyType(type);
    channel.setProxyHost("127.0.0.1");
    channel.setProxyPort(port);
    channel.setProxyUsername(username);
    return channel;
  }

  private void globalProxy(final int port) {
    ProxyServerConfiguration http = new ProxyServerConfiguration();
    http.setEnabled(true);
    http.setHost("127.0.0.1");
    http.setPort(port);
    ProxyConfiguration proxy = new ProxyConfiguration();
    proxy.setHttp(http);
    configuration.setProxy(proxy);
  }

  private static int deadPort() throws IOException {
    try (ServerSocket socket = new ServerSocket(0, 1, InetAddress.getLoopbackAddress())) {
      return socket.getLocalPort();
    }
  }

  private void assertDelivered() {
    assertThat(received, hasSize(1));
    assertThat(received.get(0).get("path"), is("/bot" + TOKEN + "/sendMessage"));
    assertThat(received.get(0).get("body"), containsString("\"chat_id\":\"-100\""));
    assertThat(received.get(0).get("userAgent"), is("Astrum/test"));
  }

  private NotificationDeliveryException failure(final NotificationChannel channel, final String password) {
    return assertThrows(NotificationDeliveryException.class,
        () -> transport().send(channel, new NotificationSecrets(TOKEN, password), "hi"));
  }

  @Test
  public void sendsThroughAnHttpProxyThatResolvesTheTelegramHost() throws Exception {
    transport().send(throughProxy(NotificationChannel.PROXY_HTTP, httpProxy.port(), null),
        NotificationSecrets.of(TOKEN), "hi");

    assertThat(httpProxy.requestLines, contains(
        "POST http://telegram.test:" + telegram.getAddress().getPort() + "/bot" + TOKEN + "/sendMessage HTTP/1.1"));
    assertThat(httpProxy.authorizations, contains(""));
    assertDelivered();
  }

  @Test
  public void answersTheBasicChallengeOfAnHttpProxy() throws Exception {
    httpProxy.credentials = "robot:" + PASSWORD;

    transport().send(throughProxy(NotificationChannel.PROXY_HTTP, httpProxy.port(), "robot"),
        new NotificationSecrets(TOKEN, PASSWORD), "hi");

    assertThat(httpProxy.authorizations, hasSize(2));
    assertThat(httpProxy.authorizations.get(0), is(""));
    assertThat(httpProxy.authorizations.get(1), is("Basic cm9ib3Q6czNjcmV0IHBhc3M="));
    assertDelivered();
  }

  @Test
  public void aWrongProxyPasswordIsReportedWithoutThePassword() {
    httpProxy.credentials = "robot:" + PASSWORD;

    NotificationDeliveryException e =
        failure(throughProxy(NotificationChannel.PROXY_HTTP, httpProxy.port(), "robot"), "wrong-password");

    assertThat(e.getMessage(), is("The proxy rejected the username or password (407 Proxy Authentication Required)"));
    assertThat(e.isRetryable(), is(false));
    assertThat(received, is(empty()));
  }

  @Test
  public void aProxyThatWantsCredentialsSaysSo() {
    httpProxy.credentials = "robot:" + PASSWORD;

    NotificationDeliveryException e = failure(throughProxy(NotificationChannel.PROXY_HTTP, httpProxy.port(), null),
        null);

    assertThat(e.getMessage(), is("The proxy requires a username and password (407 Proxy Authentication Required)"));
  }

  @Test
  public void httpsTargetsGoThroughConnectAndARefusedTunnelIsReported() {
    HttpNotificationTransport https =
        new HttpNotificationTransport(httpClientManager, antiSsrfService, List.of(), "https://telegram.test");

    NotificationDeliveryException e = assertThrows(NotificationDeliveryException.class,
        () -> https.send(throughProxy(NotificationChannel.PROXY_HTTP, httpProxy.port(), null),
            NotificationSecrets.of(TOKEN), "hi"));

    assertThat(httpProxy.requestLines, contains("CONNECT telegram.test:443 HTTP/1.1"));
    assertThat(e.getMessage(), is("The proxy refused the connection to telegram.test: 403 Forbidden"));
    assertThat(e.isRetryable(), is(false));
  }

  @Test
  public void aTunnelNeedsTheProxyCredentialsToo() {
    httpProxy.credentials = "robot:" + PASSWORD;
    HttpNotificationTransport https =
        new HttpNotificationTransport(httpClientManager, antiSsrfService, List.of(), "https://telegram.test");

    NotificationDeliveryException e = assertThrows(NotificationDeliveryException.class,
        () -> https.send(throughProxy(NotificationChannel.PROXY_HTTP, httpProxy.port(), "robot"),
            new NotificationSecrets(TOKEN, "wrong"), "hi"));

    assertThat(httpProxy.requestLines, everyItem(startsWith("CONNECT telegram.test:443 ")));
    assertThat(httpProxy.authorizations.get(httpProxy.authorizations.size() - 1), startsWith("Basic "));
    assertThat(e.getMessage(), is("The proxy rejected the username or password (407 Proxy Authentication Required)"));
  }

  @Test
  public void anUnreachableHttpProxyIsReported() throws Exception {
    int port = deadPort();

    NotificationDeliveryException e = failure(throughProxy(NotificationChannel.PROXY_HTTP, port, null), null);

    assertThat(e.getMessage(), startsWith("The proxy refused the connection: 127.0.0.1:" + port + " ("));
    assertThat(e.isRetryable(), is(true));
  }

  @Test
  public void sendsThroughASocksProxyThatResolvesTheTelegramHost() throws Exception {
    transport().send(throughProxy(NotificationChannel.PROXY_SOCKS5, socksProxy.port(), null),
        NotificationSecrets.of(TOKEN), "hi");

    assertThat(socksProxy.requests, contains("domain telegram.test:" + telegram.getAddress().getPort()));
    assertDelivered();
  }

  @Test
  public void logsIntoASocksProxyWithTheChannelsCredentials() throws Exception {
    socksProxy.username = "robot";
    socksProxy.password = PASSWORD;

    transport().send(throughProxy(NotificationChannel.PROXY_SOCKS5, socksProxy.port(), "robot"),
        new NotificationSecrets(TOKEN, PASSWORD), "hi");

    assertThat(socksProxy.logins, contains("robot:" + PASSWORD));
    assertDelivered();
  }

  @Test
  public void addressesAreSentToTheSocksProxyAsTheyAre() throws Exception {
    transport("127.0.0.1").send(throughProxy(NotificationChannel.PROXY_SOCKS5, socksProxy.port(), null),
        NotificationSecrets.of(TOKEN), "hi");

    assertThat(socksProxy.requests, contains("ipv4 127.0.0.1:" + telegram.getAddress().getPort()));
    assertDelivered();
  }

  @Test
  public void aRejectedSocksLoginIsReportedOnceWithoutThePassword() {
    socksProxy.username = "robot";
    socksProxy.password = PASSWORD;

    NotificationDeliveryException e =
        failure(throughProxy(NotificationChannel.PROXY_SOCKS5, socksProxy.port(), "robot"), "wrong-password");

    assertThat(e.getMessage(), is("The proxy rejected the username or password"));
    assertThat(e.isRetryable(), is(false));
    assertThat(socksProxy.connections.get(), is(1));
    assertThat(received, is(empty()));
  }

  @Test
  public void aSocksProxyThatWantsCredentialsSaysSo() {
    socksProxy.username = "robot";
    socksProxy.password = PASSWORD;

    NotificationDeliveryException e =
        failure(throughProxy(NotificationChannel.PROXY_SOCKS5, socksProxy.port(), null), null);

    assertThat(e.getMessage(), is("The proxy requires a username and password"));
  }

  @Test
  public void aSocksProxyThatCannotReachTelegramSaysWhy() {
    socksProxy.reply = 5;

    NotificationDeliveryException e =
        failure(throughProxy(NotificationChannel.PROXY_SOCKS5, socksProxy.port(), null), null);

    assertThat(e.getMessage(), is(
        "The proxy could not connect to telegram.test:" + telegram.getAddress().getPort() + ": connection refused"));
    assertThat(e.isRetryable(), is(true));
  }

  @Test
  public void anUnreachableSocksProxyIsReported() throws Exception {
    int port = deadPort();

    NotificationDeliveryException e = failure(throughProxy(NotificationChannel.PROXY_SOCKS5, port, null), null);

    assertThat(e.getMessage(), startsWith("The proxy refused the connection: 127.0.0.1:" + port + " ("));
    assertThat(e.isRetryable(), is(true));
  }

  @Test
  public void anHttpProxyIsNotTakenForASocksProxy() throws Exception {
    try (ServerSocket http = new ServerSocket(0, 1, InetAddress.getLoopbackAddress())) {
      Thread answer = new Thread(() -> {
        try (Socket client = http.accept()) {
          client.getOutputStream().write("HTTP/1.1 400 Bad Request\r\n\r\n".getBytes(UTF_8));
        }
        catch (IOException e) {
          // the test fails on its own
        }
      });
      answer.start();

      NotificationDeliveryException e =
          failure(throughProxy(NotificationChannel.PROXY_SOCKS5, http.getLocalPort(), null), null);

      assertThat(e.getMessage(), is("The proxy did not answer as a SOCKS5 proxy"));
      answer.join();
    }
  }

  @Test
  public void aSilentProxyIsGivenUpOnAfterTheConnectTimeout() {
    NotificationDeliveryException e =
        failure(throughProxy(NotificationChannel.PROXY_SOCKS5, httpProxy.port(), null), null);

    assertThat(e.getMessage(), is("The proxy did not answer the SOCKS5 handshake in time; is it a SOCKS5 proxy?"));
    assertThat(e.isRetryable(), is(true));
  }

  @Test
  public void directChannelsBypassTheGlobalProxy() throws Exception {
    globalProxy(deadPort());

    transport("127.0.0.1").send(telegram(NotificationChannel.PROXY_NONE), NotificationSecrets.of(TOKEN), "hi");

    assertDelivered();
  }

  @Test
  public void otherChannelsUseTheGlobalProxy() throws Exception {
    int port = deadPort();
    globalProxy(port);

    NotificationDeliveryException e = assertThrows(NotificationDeliveryException.class,
        () -> transport("127.0.0.1").send(telegram(NotificationChannel.PROXY_GLOBAL), NotificationSecrets.of(TOKEN),
            "hi"));

    assertThat(e.getMessage(), startsWith("Could not reach Telegram: Connect to 127.0.0.1:" + port));
    assertThat(e.getMessage(), not(containsString(TOKEN)));
    assertThat(received, is(empty()));
  }

  @Test
  public void theChannelsProxyReplacesTheGlobalProxy() throws Exception {
    globalProxy(deadPort());

    transport().send(throughProxy(NotificationChannel.PROXY_HTTP, httpProxy.port(), null),
        NotificationSecrets.of(TOKEN), "hi");
    transport().send(throughProxy(NotificationChannel.PROXY_SOCKS5, socksProxy.port(), null),
        NotificationSecrets.of(TOKEN), "hi");

    assertThat(httpProxy.requestLines, hasSize(1));
    assertThat(socksProxy.requests, hasSize(1));
    assertThat(received, hasSize(2));
  }

  @Test
  public void webhooksUseTheChannelsProxyAndTheSsrfCheck() throws Exception {
    NotificationChannel channel = throughProxy(NotificationChannel.PROXY_SOCKS5, socksProxy.port(), null);
    channel.setType(NotificationChannel.SLACK);

    transport().send(channel, NotificationSecrets.of("http://hooks.test:" + telegram.getAddress().getPort() + "/x"),
        "hi");

    verify(antiSsrfService).validateHost("hooks.test");
    assertThat(socksProxy.requests, contains("domain hooks.test:" + telegram.getAddress().getPort()));
    assertThat(received.get(0).get("path"), is("/x"));
  }

  @Test
  public void everyConnectionKeepsTheRequestPathsOutOfTheOutboundLog() {
    assertRedacted(telegram(NotificationChannel.PROXY_GLOBAL));
    assertRedacted(telegram(NotificationChannel.PROXY_NONE));
    assertRedacted(throughProxy(NotificationChannel.PROXY_HTTP, 3128, null));
    assertRedacted(throughProxy(NotificationChannel.PROXY_SOCKS5, 1080, "robot"));
  }

  private static void assertRedacted(final NotificationChannel channel) {
    HttpClientPlan plan = new HttpClientPlan();
    new NotificationConnection(channel, PASSWORD, new AstrumSSLConnectionSocketFactory(List.of())).customize(plan);
    assertThat(plan.getAttributes().get(HttpClientManager.REDACT_REQUEST_PATH), is(true));
  }
}
