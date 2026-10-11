package org.globalart.astrum.accesstoken.internal;

import jakarta.inject.Provider;
import jakarta.servlet.http.HttpServletRequest;

import org.globalart.astrum.common.Description;
import org.globalart.astrum.security.UserPrincipalsHelper;
import org.globalart.astrum.security.user.UserNotFoundException;

import org.apache.shiro.authc.AuthenticationInfo;
import org.apache.shiro.authc.AuthenticationToken;
import org.apache.shiro.authc.DisabledAccountException;
import org.apache.shiro.authc.IncorrectCredentialsException;
import org.apache.shiro.authc.SimpleAuthenticationInfo;
import org.apache.shiro.authc.UnknownAccountException;
import org.apache.shiro.authc.UsernamePasswordToken;
import org.apache.shiro.authc.credential.AllowAllCredentialsMatcher;
import org.apache.shiro.realm.AuthenticatingRealm;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Signs in clients that send an access token as their password, with their username as the user name.
 *
 * Tokens are for clients only: signing in to the UI with one is refused, because the UI session would not keep the
 * token's scope.
 */
@Component
@Qualifier(AccessTokenRealm.NAME)
@Description("Access Token Realm")
public class AccessTokenRealm
    extends AuthenticatingRealm
{
  public static final String NAME = "AccessTokenRealm";

  static final String SESSION_PATH = "/service/rapture/session";

  private static final Logger log = LoggerFactory.getLogger(AccessTokenRealm.class);

  private final AccessTokenService tokens;

  private final UserPrincipalsHelper principalsHelper;

  private Provider<HttpServletRequest> requestProvider;

  @Autowired
  public AccessTokenRealm(final AccessTokenService tokens, final UserPrincipalsHelper principalsHelper) {
    this.tokens = checkNotNull(tokens);
    this.principalsHelper = checkNotNull(principalsHelper);
    setName(NAME);
    setCredentialsMatcher(new AllowAllCredentialsMatcher());
    setAuthenticationCachingEnabled(false);
  }

  @Autowired
  void setRequestProvider(final Provider<HttpServletRequest> requestProvider) {
    this.requestProvider = requestProvider;
  }

  @Override
  public boolean supports(final AuthenticationToken token) {
    return token instanceof UsernamePasswordToken credentials
        && AccessTokenService.looksLikeToken(credentials.getPassword());
  }

  @Override
  protected AuthenticationInfo doGetAuthenticationInfo(final AuthenticationToken token) {
    UsernamePasswordToken credentials = (UsernamePasswordToken) token;
    if (isSessionLogin()) {
      log.debug("Refusing to start a UI session for {} with an access token", credentials.getUsername());
      throw new IncorrectCredentialsException("Access tokens cannot be used to sign in to the UI");
    }

    AccessTokenData accessToken = tokens.authenticate(credentials.getPassword())
        .orElseThrow(() -> new IncorrectCredentialsException("Unknown or expired access token"));
    if (credentials.getUsername() == null || !accessToken.getUsername().equalsIgnoreCase(credentials.getUsername())) {
      throw new IncorrectCredentialsException("Access token belongs to another user");
    }

    try {
      if (!principalsHelper.getUserStatus(accessToken.getPrincipals()).isActive()) {
        throw new DisabledAccountException("User " + accessToken.getUsername() + " is not active");
      }
    }
    catch (UserNotFoundException e) {
      throw new UnknownAccountException("User " + accessToken.getUsername() + " no longer exists");
    }

    return new SimpleAuthenticationInfo(AccessTokenService.principals(accessToken), credentials.getCredentials());
  }

  private boolean isSessionLogin() {
    if (requestProvider == null) {
      return false;
    }
    try {
      HttpServletRequest request = requestProvider.get();
      return request != null && request.getRequestURI() != null && request.getRequestURI().endsWith(SESSION_PATH);
    }
    catch (RuntimeException e) {
      return false;
    }
  }
}
