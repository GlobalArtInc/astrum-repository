
package org.globalart.astrum.security.token;

import java.util.Optional;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.security.SecurityHelper;
import org.globalart.astrum.security.authc.apikey.ApiKey;
import org.globalart.astrum.security.authc.apikey.ApiKeyService;

import org.apache.shiro.subject.PrincipalCollection;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Allows the logic for managing bearer tokens to be shared between formats and it is intended that this will be
 * subclassed and a format-specific concrete implementation provided.
 *
 * @since 3.6
 */
public abstract class BearerTokenManager
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  protected final ApiKeyService apiKeyService;

  protected final SecurityHelper securityHelper;

  private final String format;

  @Autowired(required = false)
  protected BearerTokenEchoResolver echoResolver;

  @Autowired
  public BearerTokenManager(
      final ApiKeyService apiKeyService,
      final SecurityHelper securityHelper,
      final String format)
  {
    this.apiKeyService = checkNotNull(apiKeyService);
    this.securityHelper = checkNotNull(securityHelper);
    this.format = checkNotNull(format);
  }

  /**
   * Creates (if not already exists) a bearer token mapped to given principal and returns the newly created token.
   * If an {@link BearerTokenEchoResolver} is present and the current request carries a service-account credential,
   * that credential is echoed back instead of minting a new token.
   */
  protected String createToken(final PrincipalCollection principals) {
    checkNotNull(principals);
    if (echoResolver != null) {
      Optional<String> echoed = echoResolver.presentedToken(principals);
      if (echoed.isPresent()) {
        return echoed.get();
      }
    }
    char[] apiKey = apiKeyService.getApiKey(format, principals).map(ApiKey::getApiKey).orElse(null);
    if (apiKey != null) {
      return format + "." + new String(apiKey);
    }
    return format + "." + new String(apiKeyService.createApiKey(format, principals));
  }

  /**
   * Removes any Bearer token for current user, if exists, and returns {@code true}.
   */
  public boolean deleteToken() {
    final PrincipalCollection principals = securityHelper.subject().getPrincipals();
    if (apiKeyService.getApiKey(format, principals).isPresent()) {
      apiKeyService.deleteApiKey(format, securityHelper.subject().getPrincipals());
      return true;
    }
    return false;
  }
}
