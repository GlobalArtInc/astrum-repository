package org.globalart.astrum.coreui.internal.http;

import java.util.HashSet;
import java.util.List;
import java.util.Objects;
import java.util.Set;

import javax.annotation.Nullable;

import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.PUT;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;

import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.coreui.HttpSettingsComponent;
import org.globalart.astrum.coreui.HttpSettingsXO;
import org.globalart.astrum.coreui.internal.http.HttpSettingsApiXO.AuthXO;
import org.globalart.astrum.coreui.internal.http.HttpSettingsApiXO.ProxyXO;
import org.globalart.astrum.rest.Resource;
import org.globalart.astrum.rest.ValidationErrorsException;

import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import static java.util.stream.Collectors.toList;
import static org.globalart.astrum.rest.APIConstants.V1_API_PREFIX;

/**
 * REST API for the HTTP settings of the server, backed by the same conversion as the ExtDirect
 * {@link HttpSettingsComponent}: proxy passwords are stored as secrets and only ever returned as a placeholder.
 */
@Component
@Path(HttpSettingsApiResource.RESOURCE_URI)
@Produces(APPLICATION_JSON)
public class HttpSettingsApiResource
    implements Resource, HttpSettingsApiResourceDoc
{
  public static final String RESOURCE_URI = V1_API_PREFIX + "/http";

  private static final String HTTP_PROXY = "httpProxy";

  private static final String HTTPS_PROXY = "httpsProxy";

  private final HttpSettingsComponent httpSettingsComponent;

  @Autowired
  public HttpSettingsApiResource(final HttpSettingsComponent httpSettingsComponent) {
    this.httpSettingsComponent = checkNotNull(httpSettingsComponent);
  }

  @Override
  @GET
  @RequiresAuthentication
  @RequiresPermissions("astrum:settings:read")
  public HttpSettingsApiXO getHttpSettings() {
    return asApi(httpSettingsComponent.read());
  }

  @Override
  @PUT
  @Consumes(APPLICATION_JSON)
  @RequiresAuthentication
  @RequiresPermissions("astrum:settings:update")
  public HttpSettingsApiXO updateHttpSettings(final HttpSettingsApiXO settings) {
    if (settings == null) {
      throw new ValidationErrorsException("HTTP settings are required");
    }
    return asApi(httpSettingsComponent.update(asComponent(settings)));
  }

  static HttpSettingsApiXO asApi(final HttpSettingsXO settings) {
    HttpSettingsApiXO result = new HttpSettingsApiXO();
    result.setUserAgent(settings.getUserAgentSuffix());
    result.setTimeout(settings.getTimeout());
    result.setRetries(settings.getRetries());
    result.setHttpProxy(proxy(settings.getHttpEnabled(), settings.getHttpHost(), settings.getHttpPort(),
        auth(settings.getHttpAuthEnabled(), settings.getHttpAuthUsername(), settings.getHttpAuthPassword(),
            settings.getHttpAuthNtlmHost(), settings.getHttpAuthNtlmDomain())));
    result.setHttpsProxy(proxy(settings.getHttpsEnabled(), settings.getHttpsHost(), settings.getHttpsPort(),
        auth(settings.getHttpsAuthEnabled(), settings.getHttpsAuthUsername(), settings.getHttpsAuthPassword(),
            settings.getHttpsAuthNtlmHost(), settings.getHttpsAuthNtlmDomain())));
    result.setNonProxyHosts(settings.getNonProxyHosts() == null
        ? List.of()
        : settings.getNonProxyHosts().stream().filter(Objects::nonNull).sorted().collect(toList()));
    return result;
  }

  static HttpSettingsXO asComponent(final HttpSettingsApiXO settings) {
    HttpSettingsXO result = new HttpSettingsXO();
    result.setUserAgentSuffix(blankToNull(settings.getUserAgent()));
    result.setTimeout(settings.getTimeout());
    result.setRetries(settings.getRetries());

    ProxyXO http = settings.getHttpProxy();
    if (http != null && http.isEnabled()) {
      result.setHttpEnabled(true);
      result.setHttpHost(host(HTTP_PROXY, http));
      result.setHttpPort(port(HTTP_PROXY, http));
      AuthXO auth = http.getAuthInfo();
      if (auth != null && auth.isEnabled()) {
        result.setHttpAuthEnabled(true);
        result.setHttpAuthUsername(username(HTTP_PROXY, auth));
        result.setHttpAuthPassword(auth.getPassword());
        result.setHttpAuthNtlmHost(blankToNull(auth.getNtlmHost()));
        result.setHttpAuthNtlmDomain(blankToNull(auth.getNtlmDomain()));
      }
    }
    else {
      result.setHttpEnabled(false);
    }

    ProxyXO https = settings.getHttpsProxy();
    if (https != null && https.isEnabled()) {
      result.setHttpsEnabled(true);
      result.setHttpsHost(host(HTTPS_PROXY, https));
      result.setHttpsPort(port(HTTPS_PROXY, https));
      AuthXO auth = https.getAuthInfo();
      if (auth != null && auth.isEnabled()) {
        result.setHttpsAuthEnabled(true);
        result.setHttpsAuthUsername(username(HTTPS_PROXY, auth));
        result.setHttpsAuthPassword(auth.getPassword());
        result.setHttpsAuthNtlmHost(blankToNull(auth.getNtlmHost()));
        result.setHttpsAuthNtlmDomain(blankToNull(auth.getNtlmDomain()));
      }
    }
    else {
      result.setHttpsEnabled(false);
    }

    if (settings.getNonProxyHosts() != null) {
      Set<String> hosts = new HashSet<>();
      settings.getNonProxyHosts()
          .stream()
          .filter(host -> !Strings2.isBlank(host))
          .map(String::trim)
          .forEach(hosts::add);
      result.setNonProxyHosts(hosts);
    }
    return result;
  }

  private static ProxyXO proxy(
      @Nullable final Boolean enabled,
      @Nullable final String host,
      @Nullable final Integer port,
      final AuthXO auth)
  {
    ProxyXO proxy = new ProxyXO();
    proxy.setEnabled(Boolean.TRUE.equals(enabled));
    proxy.setHost(host);
    proxy.setPort(port == null ? null : port.toString());
    proxy.setAuthInfo(auth);
    return proxy;
  }

  private static AuthXO auth(
      @Nullable final Boolean enabled,
      @Nullable final String username,
      @Nullable final String password,
      @Nullable final String ntlmHost,
      @Nullable final String ntlmDomain)
  {
    AuthXO auth = new AuthXO();
    auth.setEnabled(Boolean.TRUE.equals(enabled));
    auth.setUsername(username);
    auth.setPassword(password);
    auth.setNtlmHost(ntlmHost);
    auth.setNtlmDomain(ntlmDomain);
    return auth;
  }

  private static String host(final String proxy, final ProxyXO settings) {
    if (Strings2.isBlank(settings.getHost())) {
      throw new ValidationErrorsException(proxy + ".host", "Host is required");
    }
    return settings.getHost().trim();
  }

  private static Integer port(final String proxy, final ProxyXO settings) {
    try {
      int port = Integer.parseInt(Strings2.isBlank(settings.getPort()) ? "" : settings.getPort().trim());
      if (port >= 1 && port <= 65535) {
        return port;
      }
    }
    catch (NumberFormatException e) {
      // reported below
    }
    throw new ValidationErrorsException(proxy + ".port", "Port must be a number between 1 and 65535");
  }

  private static String username(final String proxy, final AuthXO auth) {
    if (Strings2.isBlank(auth.getUsername())) {
      throw new ValidationErrorsException(proxy + ".authInfo.username", "Username is required");
    }
    return auth.getUsername().trim();
  }

  @Nullable
  private static String blankToNull(@Nullable final String value) {
    return Strings2.isBlank(value) ? null : value.trim();
  }
}
