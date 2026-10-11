package org.globalart.astrum.coreui.internal.http;

import java.util.List;

import io.swagger.v3.oas.annotations.media.Schema;

/**
 * The HTTP settings of the server as the REST API exchanges them: the connection of outgoing requests and the proxies
 * they go through.
 */
@Schema(description = "Settings of the outgoing HTTP requests of the server")
public class HttpSettingsApiXO
{
  @Schema(description = "Text added to the User-Agent header", example = "MyCompany/1.0")
  private String userAgent;

  @Schema(description = "Seconds to wait for a connection, 1 to 3600; empty for the default", example = "20")
  private Integer timeout;

  @Schema(description = "Times to retry a failed connection, 0 to 10; empty for the default", example = "2")
  private Integer retries;

  private ProxyXO httpProxy;

  private ProxyXO httpsProxy;

  @Schema(description = "Hosts reached without the proxies", example = "[\"*.example.com\", \"localhost\"]")
  private List<String> nonProxyHosts;

  public String getUserAgent() {
    return userAgent;
  }

  public void setUserAgent(final String userAgent) {
    this.userAgent = userAgent;
  }

  public Integer getTimeout() {
    return timeout;
  }

  public void setTimeout(final Integer timeout) {
    this.timeout = timeout;
  }

  public Integer getRetries() {
    return retries;
  }

  public void setRetries(final Integer retries) {
    this.retries = retries;
  }

  public ProxyXO getHttpProxy() {
    return httpProxy;
  }

  public void setHttpProxy(final ProxyXO httpProxy) {
    this.httpProxy = httpProxy;
  }

  public ProxyXO getHttpsProxy() {
    return httpsProxy;
  }

  public void setHttpsProxy(final ProxyXO httpsProxy) {
    this.httpsProxy = httpsProxy;
  }

  public List<String> getNonProxyHosts() {
    return nonProxyHosts;
  }

  public void setNonProxyHosts(final List<String> nonProxyHosts) {
    this.nonProxyHosts = nonProxyHosts;
  }

  /**
   * A proxy server.
   */
  @Schema(name = "HttpSettingsProxyXO", description = "A proxy server")
  public static class ProxyXO
  {
    private boolean enabled;

    @Schema(example = "proxy.example.com")
    private String host;

    @Schema(example = "3128")
    private String port;

    private AuthXO authInfo;

    public boolean isEnabled() {
      return enabled;
    }

    public void setEnabled(final boolean enabled) {
      this.enabled = enabled;
    }

    public String getHost() {
      return host;
    }

    public void setHost(final String host) {
      this.host = host;
    }

    public String getPort() {
      return port;
    }

    public void setPort(final String port) {
      this.port = port;
    }

    public AuthXO getAuthInfo() {
      return authInfo;
    }

    public void setAuthInfo(final AuthXO authInfo) {
      this.authInfo = authInfo;
    }
  }

  /**
   * How the server signs in to a proxy. A host or a domain selects NTLM.
   */
  @Schema(name = "HttpSettingsAuthXO",
      description = "How the server signs in to a proxy; a host or a domain selects NTLM")
  public static class AuthXO
  {
    private boolean enabled;

    private String username;

    @Schema(description = "Returned as a placeholder; sending the placeholder back keeps the stored password")
    private String password;

    private String ntlmHost;

    private String ntlmDomain;

    public boolean isEnabled() {
      return enabled;
    }

    public void setEnabled(final boolean enabled) {
      this.enabled = enabled;
    }

    public String getUsername() {
      return username;
    }

    public void setUsername(final String username) {
      this.username = username;
    }

    public String getPassword() {
      return password;
    }

    public void setPassword(final String password) {
      this.password = password;
    }

    public String getNtlmHost() {
      return ntlmHost;
    }

    public void setNtlmHost(final String ntlmHost) {
      this.ntlmHost = ntlmHost;
    }

    public String getNtlmDomain() {
      return ntlmDomain;
    }

    public void setNtlmDomain(final String ntlmDomain) {
      this.ntlmDomain = ntlmDomain;
    }
  }
}
