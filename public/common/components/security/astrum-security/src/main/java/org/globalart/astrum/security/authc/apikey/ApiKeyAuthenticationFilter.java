
package org.globalart.astrum.security.authc.apikey;

import java.util.List;
import java.util.Map;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.servlet.ServletRequest;
import jakarta.servlet.ServletResponse;
import jakarta.servlet.annotation.WebFilter;
import jakarta.servlet.http.HttpServletRequest;

import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.common.app.WebFilterPriority;
import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.security.authc.AstrumApiKeyAuthenticationToken;
import org.globalart.astrum.security.authc.AstrumBasicHttpAuthenticationFilter;

import org.apache.shiro.authc.AuthenticationToken;
import org.apache.shiro.web.filter.authc.AuthenticatingFilter;
import org.apache.shiro.web.util.WebUtils;
import org.springframework.core.annotation.Order;

import static com.google.common.base.Preconditions.checkNotNull;

// FIXME: This isn't basic-auth, so shouldn't extend from that base-class?

/**
 * {@link AuthenticatingFilter} that looks for credentials with help of registered {@link ApiKeyExtractor}s.
 */
@WebFilter(filterName = ApiKeyAuthenticationFilter.NAME)
@Order(WebFilterPriority.AUTHENTICATION)
public class ApiKeyAuthenticationFilter
    extends AstrumBasicHttpAuthenticationFilter
{
  private static final String NX_APIKEY_PRINCIPAL = ApiKeyAuthenticationFilter.class.getName() + ".principal";

  private static final String NX_APIKEY_TOKEN = ApiKeyAuthenticationFilter.class.getName() + ".apiKey";

  public static final String NAME = "nx-apikey-authc";

  private final Map<String, ApiKeyExtractor> apiKeys;

  @Autowired
  public ApiKeyAuthenticationFilter(final List<ApiKeyExtractor> apiKeysList) {
    this.apiKeys = QualifierUtil.buildQualifierBeanMap(checkNotNull(apiKeysList));
  }

  @Override
  protected boolean isLoginAttempt(final ServletRequest request, final ServletResponse response) {
    final HttpServletRequest http = WebUtils.toHttp(request);
    for (final Map.Entry<String, ApiKeyExtractor> apiKeyEntry : apiKeys.entrySet()) {
      final String apiKey = apiKeyEntry.getValue().extract(http);
      if (null != apiKey) {
        log.trace("ApiKeyExtractor {} detected presence of API Key", apiKeyEntry.getKey());
        request.setAttribute(NX_APIKEY_PRINCIPAL, apiKeyEntry.getKey());
        request.setAttribute(NX_APIKEY_TOKEN, apiKey);
        return true;
      }
    }
    return super.isLoginAttempt(request, response);
  }

  @Override
  protected AuthenticationToken createToken(final ServletRequest request, final ServletResponse response) {
    final String principal = (String) request.getAttribute(NX_APIKEY_PRINCIPAL);
    final String token = (String) request.getAttribute(NX_APIKEY_TOKEN);
    if (!Strings2.isBlank(principal) && !Strings2.isBlank(token)) {
      return new AstrumApiKeyAuthenticationToken(principal, token.toCharArray(), request.getRemoteHost());
    }
    return super.createToken(request, response);
  }
}
