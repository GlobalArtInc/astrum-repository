
package org.globalart.astrum.httpclient.config;

import java.util.List;

import org.apache.http.client.config.RequestConfig;
import org.apache.http.impl.client.HttpClientBuilder;
import org.junit.Test;
import org.mockito.Mock;
import org.globalart.astrum.httpclient.HttpClientPlan;
import org.globalart.astrum.httpclient.internal.AstrumHttpRoutePlanner;
import org.apache.http.HttpHost;
import org.apache.http.HttpRequest;
import org.apache.http.client.RedirectStrategy;
import org.apache.http.client.config.CookieSpecs;
import org.apache.http.conn.routing.HttpRoute;
import org.apache.http.protocol.HttpContext;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.nullValue;
import static org.mockito.Mockito.*;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class ConfigurationCustomizerTest
{

  @Mock
  private RedirectStrategy redirectStrategy;

  private HttpClientConfiguration httpClientConfiguration = mock(HttpClientConfiguration.class);

  private ConfigurationCustomizer configurationCustomizer = new ConfigurationCustomizer(httpClientConfiguration);

  private HttpHost httpProxyHost = new HttpHost("http-proxy", 8080);

  private HttpHost httpsProxyHost = new HttpHost("https-proxy", 8443);

  private AstrumHttpRoutePlanner create(String[] nonProxyHosts) {
    ProxyServerConfiguration http = new ProxyServerConfiguration();
    http.setHost(httpProxyHost.getHostName());
    http.setPort(httpProxyHost.getPort());
    http.setEnabled(true);
    ProxyServerConfiguration https = new ProxyServerConfiguration();
    https.setHost(httpsProxyHost.getHostName());
    https.setPort(httpsProxyHost.getPort());
    https.setEnabled(true);
    ProxyConfiguration proxyConfiguration = new ProxyConfiguration();
    proxyConfiguration.setHttp(http);
    proxyConfiguration.setHttps(https);
    proxyConfiguration.setNonProxyHosts(nonProxyHosts);
    return configurationCustomizer.createRoutePlanner(proxyConfiguration);
  }

  @Test
  public void setRetryHandlerIsCalledWhenRetriesAreSetInTheConfiguration() {
    ConnectionConfiguration configuration = new ConnectionConfiguration();
    configuration.setRetries(4);
    when(httpClientConfiguration.getConnection()).thenReturn(configuration);
    HttpClientPlan plan = new HttpClientPlan();
    ConfigurationCustomizer spyCustomizer = spy(configurationCustomizer);
    spyCustomizer.customize(plan);
    verify(spyCustomizer).setRetryHandler(configuration, plan);
  }

  @Test
  public void planIsUpdatedWithCircularRedirectAndCookieSettingsFromConfiguration() {
    ConnectionConfiguration configuration = new ConnectionConfiguration();
    configuration.setEnableCircularRedirects(true);
    configuration.setEnableCookies(true);

    when(httpClientConfiguration.getConnection())
        .thenReturn(configuration);
    HttpClientPlan plan = new HttpClientPlan();
    configurationCustomizer.customize(plan);
    RequestConfig request = plan.getRequest().build();
    assertThat(request.isCircularRedirectsAllowed(), equalTo(true));
    assertThat(request.getMaxRedirects(), equalTo(50));
    assertThat(request.getCookieSpec(), equalTo(CookieSpecs.DEFAULT));
  }

  @Test
  public void planIsNotUpdatedWhenCircularRedirectAndCookieSettingsMissingFromConfiguration() {
    HttpClientPlan plan = new HttpClientPlan();
    configurationCustomizer.customize(plan);
    RequestConfig request = plan.getRequest().build();
    assertThat(request.isCircularRedirectsAllowed(), equalTo(false));
    assertThat(request.getMaxRedirects(), equalTo(50));
    assertThat(request.getCookieSpec(), nullValue());
  }

  @Test
  public void planIsUpdatedWithRedirectStrategy() {
    when(httpClientConfiguration.getRedirectStrategy()).thenReturn(redirectStrategy);
    HttpClientPlan plan = spy(new HttpClientPlan());
    HttpClientBuilder clientBuilder = mock(HttpClientBuilder.class);
    when(plan.getClient()).thenReturn(clientBuilder);
    configurationCustomizer.customize(plan);
    verify(clientBuilder).setRedirectStrategy(redirectStrategy);
  }

  @Test
  public void sanityTest() throws Exception {
    AstrumHttpRoutePlanner planner = create(null);
    HttpRoute route;

    route = planner.determineRoute(new HttpHost("globalart.dev", 80, "http"), mock(HttpRequest.class),
        mock(HttpContext.class));
    assertThat(route.getHopTarget(0), equalTo(httpProxyHost));

    route = planner.determineRoute(new HttpHost("globalart.dev", 443, "https"), mock(HttpRequest.class),
        mock(HttpContext.class));
    assertThat(route.getHopTarget(0), equalTo(httpsProxyHost));

    HttpHost localhost = new HttpHost("localhost", 80);
    route = planner.determineRoute(localhost, mock(HttpRequest.class), mock(HttpContext.class));
    assertThat(route.getHopTarget(0), equalTo(httpProxyHost));
  }

  @Test
  public void customNonProxyHosts() throws Exception {
    AstrumHttpRoutePlanner planner =
        create(new String[]{"*.globalart.*", "*.example.com", "localhost", "10.*", "[:*", "*:8]"});
    HttpRoute route;

    // must not have proxy used
    for (String host : List.of("www.globalart.dev", "www.globalart.dev", "smtp.example.com", "localhost", "10.0.0.1",
        "[::8]", "[::9]")) {
      HttpHost target = new HttpHost(host, 80);
      route = planner.determineRoute(target, mock(HttpRequest.class), mock(HttpContext.class));
      assertThat(route.getHopTarget(0), equalTo(target));
    }

    // must have HTTP proxy used
    for (String host : List.of("www.theglobalart.com", "www.google.com", "example.com", "example.org")) {
      HttpHost target = new HttpHost(host, 80, "http");
      route = planner.determineRoute(target, mock(HttpRequest.class), mock(HttpContext.class));
      assertThat(route.getHopTarget(0), equalTo(httpProxyHost));
    }

    // must have HTTPS proxy used
    for (String host : List.of("www.google.com", "example.com", "example.org")) {
      HttpHost target = new HttpHost(host, 8443, "https");
      route = planner.determineRoute(target, mock(HttpRequest.class), mock(HttpContext.class));
      assertThat(route.getHopTarget(0), equalTo(httpsProxyHost));
    }
  }

  @Test
  public void ipv6NonProxyHosts() throws Exception {
    AstrumHttpRoutePlanner planner;
    HttpRoute route;

    planner = create(new String[]{"*:8]"});
    // must not have proxy used
    HttpHost target = new HttpHost("[::8]", 80);
    route = planner.determineRoute(target, mock(HttpRequest.class), mock(HttpContext.class));
    assertThat(route.getHopTarget(0), equalTo(target));

    // must have HTTP proxy used
    target = new HttpHost("[::9]", 80, "http");
    route = planner.determineRoute(target, mock(HttpRequest.class), mock(HttpContext.class));
    assertThat(route.getHopTarget(0), equalTo(httpProxyHost));

    planner = create(new String[]{"[:*"});
    // must not have proxy used
    for (String host : List.of("[::8]", "[::9]")) {
      target = new HttpHost(host, 80);
      route = planner.determineRoute(target, mock(HttpRequest.class), mock(HttpContext.class));
      assertThat(route.getHopTarget(0), equalTo(target));
    }

    // must have HTTP proxy used
    for (String host : List.of("globalart.dev", "1.2.3.4")) {
      target = new HttpHost(host, 80, "http");
      route = planner.determineRoute(target, mock(HttpRequest.class), mock(HttpContext.class));
      assertThat(route.getHopTarget(0), equalTo(httpProxyHost));
    }
  }

  @Test
  public void planIsUpdatedWithDisableContentCompression() {
    checkContentDisableCompressionPropagation(true);
    checkContentDisableCompressionPropagation(false);
    checkContentDisableCompressionPropagation(null);
  }

  private void checkContentDisableCompressionPropagation(final Boolean disableContentCompression) {
    when(httpClientConfiguration.getDisableContentCompression()).thenReturn(disableContentCompression);
    HttpClientPlan plan = spy(new HttpClientPlan());
    HttpClientBuilder clientBuilder = mock(HttpClientBuilder.class);
    when(plan.getClient()).thenReturn(clientBuilder);
    configurationCustomizer.customize(plan);
    if (disableContentCompression == null || !disableContentCompression) {
      verify(clientBuilder, never()).disableContentCompression();
    }
    else {
      verify(clientBuilder).disableContentCompression();
    }
  }
}
