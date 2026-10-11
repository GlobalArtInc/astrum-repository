package org.globalart.astrum.coreui.internal.http;

import java.util.Arrays;
import java.util.List;
import java.util.Set;

import org.globalart.astrum.coreui.HttpSettingsComponent;
import org.globalart.astrum.coreui.HttpSettingsXO;
import org.globalart.astrum.coreui.internal.http.HttpSettingsApiXO.AuthXO;
import org.globalart.astrum.coreui.internal.http.HttpSettingsApiXO.ProxyXO;
import org.globalart.astrum.rapture.PasswordPlaceholder;
import org.globalart.astrum.rest.ValidationErrorsException;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension.WithUser;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.ArgumentCaptor;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.containsInAnyOrder;
import static org.hamcrest.Matchers.empty;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

/**
 * Tests {@link HttpSettingsApiResource}.
 */
@ExtendWith(MockitoExtension.class)
@ExtendWith(AuthenticationExtension.class)
@WithUser
class HttpSettingsApiResourceTest
{
  @Mock
  private HttpSettingsComponent httpSettingsComponent;

  private HttpSettingsApiResource underTest;

  @BeforeEach
  void setup() {
    underTest = new HttpSettingsApiResource(httpSettingsComponent);
  }

  @Test
  void resourceUri() {
    assertThat(HttpSettingsApiResource.RESOURCE_URI, is("/v1/http"));
  }

  @Test
  void getReturnsTheSettingsWithTheProxyPasswordMasked() {
    HttpSettingsXO stored = new HttpSettingsXO();
    stored.setUserAgentSuffix("MyCompany/1.0");
    stored.setTimeout(20);
    stored.setRetries(2);
    stored.setHttpEnabled(true);
    stored.setHttpHost("proxy.example.com");
    stored.setHttpPort(3128);
    stored.setHttpAuthEnabled(true);
    stored.setHttpAuthUsername("proxyuser");
    stored.setHttpAuthPassword(PasswordPlaceholder.get());
    stored.setNonProxyHosts(Set.of("localhost", "*.example.com"));
    when(httpSettingsComponent.read()).thenReturn(stored);

    HttpSettingsApiXO settings = underTest.getHttpSettings();

    assertThat(settings.getUserAgent(), is("MyCompany/1.0"));
    assertThat(settings.getTimeout(), is(20));
    assertThat(settings.getRetries(), is(2));
    assertThat(settings.getHttpProxy().isEnabled(), is(true));
    assertThat(settings.getHttpProxy().getHost(), is("proxy.example.com"));
    assertThat(settings.getHttpProxy().getPort(), is("3128"));
    assertThat(settings.getHttpProxy().getAuthInfo().isEnabled(), is(true));
    assertThat(settings.getHttpProxy().getAuthInfo().getUsername(), is("proxyuser"));
    assertThat(PasswordPlaceholder.is(settings.getHttpProxy().getAuthInfo().getPassword()), is(true));
    assertThat(settings.getHttpsProxy().isEnabled(), is(false));
    assertThat(settings.getHttpsProxy().getAuthInfo().isEnabled(), is(false));
    assertThat(settings.getNonProxyHosts(), contains("*.example.com", "localhost"));
  }

  @Test
  void getOfDefaultSettingsHasNoProxies() {
    when(httpSettingsComponent.read()).thenReturn(new HttpSettingsXO());

    HttpSettingsApiXO settings = underTest.getHttpSettings();

    assertThat(settings.getUserAgent(), nullValue());
    assertThat(settings.getTimeout(), nullValue());
    assertThat(settings.getHttpProxy().isEnabled(), is(false));
    assertThat(settings.getHttpsProxy().isEnabled(), is(false));
    assertThat(settings.getNonProxyHosts(), empty());
  }

  @Test
  void updatePassesTheSettingsToTheComponent() {
    when(httpSettingsComponent.update(any())).thenReturn(new HttpSettingsXO());
    HttpSettingsApiXO settings = new HttpSettingsApiXO();
    settings.setUserAgent(" ");
    settings.setTimeout(30);
    settings.setRetries(0);
    settings.setHttpProxy(proxy(true, " proxy.example.com ", "3128", auth(true, "proxyuser", "secret", "", "")));
    settings.setHttpsProxy(proxy(true, "secure.example.com", "443", auth(true, "ntlmuser",
        PasswordPlaceholder.get(), "workstation", "CORP")));
    settings.setNonProxyHosts(Arrays.asList("localhost", " ", "*.example.com "));

    underTest.updateHttpSettings(settings);

    ArgumentCaptor<HttpSettingsXO> captor = ArgumentCaptor.forClass(HttpSettingsXO.class);
    verify(httpSettingsComponent).update(captor.capture());
    HttpSettingsXO sent = captor.getValue();
    assertThat(sent.getUserAgentSuffix(), nullValue());
    assertThat(sent.getTimeout(), is(30));
    assertThat(sent.getRetries(), is(0));
    assertThat(sent.getHttpEnabled(), is(true));
    assertThat(sent.getHttpHost(), is("proxy.example.com"));
    assertThat(sent.getHttpPort(), is(3128));
    assertThat(sent.getHttpAuthEnabled(), is(true));
    assertThat(sent.getHttpAuthUsername(), is("proxyuser"));
    assertThat(sent.getHttpAuthPassword(), is("secret"));
    // blank NTLM fields would otherwise select NTLM authentication
    assertThat(sent.getHttpAuthNtlmHost(), nullValue());
    assertThat(sent.getHttpAuthNtlmDomain(), nullValue());
    assertThat(sent.getHttpsEnabled(), is(true));
    assertThat(sent.getHttpsPort(), is(443));
    assertThat(PasswordPlaceholder.is(sent.getHttpsAuthPassword()), is(true));
    assertThat(sent.getHttpsAuthNtlmHost(), is("workstation"));
    assertThat(sent.getHttpsAuthNtlmDomain(), is("CORP"));
    assertThat(sent.getNonProxyHosts(), containsInAnyOrder("localhost", "*.example.com"));
  }

  @Test
  void updateDropsTheDetailsOfDisabledProxiesAndAuthentication() {
    when(httpSettingsComponent.update(any())).thenReturn(new HttpSettingsXO());
    HttpSettingsApiXO settings = new HttpSettingsApiXO();
    settings.setHttpProxy(proxy(false, "proxy.example.com", "not a port", auth(true, "proxyuser", "secret", "", "")));
    settings.setHttpsProxy(proxy(true, "secure.example.com", "443", auth(false, "ignored", "ignored", "", "")));
    settings.setNonProxyHosts(List.of());

    underTest.updateHttpSettings(settings);

    ArgumentCaptor<HttpSettingsXO> captor = ArgumentCaptor.forClass(HttpSettingsXO.class);
    verify(httpSettingsComponent).update(captor.capture());
    HttpSettingsXO sent = captor.getValue();
    assertThat(sent.getHttpEnabled(), is(false));
    assertThat(sent.getHttpHost(), nullValue());
    assertThat(sent.getHttpAuthEnabled(), nullValue());
    assertThat(sent.getHttpsEnabled(), is(true));
    assertThat(sent.getHttpsAuthEnabled(), nullValue());
    assertThat(sent.getHttpsAuthUsername(), nullValue());
  }

  @Test
  void updateRejectsAnEnabledProxyWithoutHostPortOrUsername() {
    assertError(proxy(true, " ", "3128", null), "httpProxy.host");
    assertError(proxy(true, "proxy.example.com", "", null), "httpProxy.port");
    assertError(proxy(true, "proxy.example.com", "70000", null), "httpProxy.port");
    assertError(proxy(true, "proxy.example.com", "3128", auth(true, "", "secret", null, null)),
        "httpProxy.authInfo.username");
    verify(httpSettingsComponent, never()).update(any());
  }

  @Test
  void updateWithoutABodyIsRejected() {
    assertThrows(ValidationErrorsException.class, () -> underTest.updateHttpSettings(null));
  }

  private void assertError(final ProxyXO httpProxy, final String id) {
    HttpSettingsApiXO settings = new HttpSettingsApiXO();
    settings.setHttpProxy(httpProxy);
    ValidationErrorsException error =
        assertThrows(ValidationErrorsException.class, () -> underTest.updateHttpSettings(settings));
    assertThat(error.getValidationErrors().get(0).getId(), is(id));
  }

  private static ProxyXO proxy(final boolean enabled, final String host, final String port, final AuthXO auth) {
    ProxyXO proxy = new ProxyXO();
    proxy.setEnabled(enabled);
    proxy.setHost(host);
    proxy.setPort(port);
    proxy.setAuthInfo(auth);
    return proxy;
  }

  private static AuthXO auth(
      final boolean enabled,
      final String username,
      final String password,
      final String ntlmHost,
      final String ntlmDomain)
  {
    AuthXO auth = new AuthXO();
    auth.setEnabled(enabled);
    auth.setUsername(username);
    auth.setPassword(password);
    auth.setNtlmHost(ntlmHost);
    auth.setNtlmDomain(ntlmDomain);
    return auth;
  }
}
