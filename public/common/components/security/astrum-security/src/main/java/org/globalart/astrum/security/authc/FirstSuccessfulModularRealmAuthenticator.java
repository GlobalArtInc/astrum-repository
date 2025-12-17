
package org.globalart.astrum.security.authc;

import java.util.Collection;
import java.util.EnumSet;
import java.util.Set;

import org.apache.shiro.SecurityUtils;
import org.apache.shiro.authc.AuthenticationException;
import org.apache.shiro.authc.AuthenticationInfo;
import org.apache.shiro.authc.AuthenticationToken;
import org.apache.shiro.authc.CredentialsException;
import org.apache.shiro.authc.DisabledAccountException;
import org.apache.shiro.authc.ExpiredCredentialsException;
import org.apache.shiro.authc.IncorrectCredentialsException;
import org.apache.shiro.authc.UnknownAccountException;
import org.apache.shiro.authc.pam.ModularRealmAuthenticator;
import org.apache.shiro.realm.Realm;
import org.apache.shiro.subject.Subject;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.globalart.astrum.datastore.api.DataAccessException;

/**
 * This Authenticator will only try to authenticate with each realm.
 * The first successful {@link AuthenticationInfo} found will be returned
 * (only if an authenticated user have the same realm) and other realms will not be queried.
 *
 * @see ModularRealmAuthenticator
 */
public class FirstSuccessfulModularRealmAuthenticator
    extends ModularRealmAuthenticator
{
  private static final Logger log = LoggerFactory.getLogger(FirstSuccessfulModularRealmAuthenticator.class);

  @Override
  protected AuthenticationInfo doMultiRealmAuthentication(// NOSONAR need to distinguish the exceptions to obtain reason
                                                          // for failure
      final Collection<Realm> realms,
      final AuthenticationToken token)
  {
    log.trace("Iterating through [{}] realms for PAM authentication", realms.size());

    Set<AuthenticationFailureReason> authenticationFailureReasons = EnumSet.noneOf(AuthenticationFailureReason.class);
    Subject subject = SecurityUtils.getSubject();

    for (Realm realm : realms) {
      // check if the realm supports this token
      if (realm.supports(token)) {
        log.trace("Attempting to authenticate token [{}] using realm of type [{}]", token, realm);

        try {
          AuthenticationInfo info = realm.getAuthenticationInfo(token);
          if (info != null) {
            Set<String> realmNames = info.getPrincipals().getRealmNames();
            if (subject.isAuthenticated() && !subject.getPrincipals().getRealmNames().containsAll(realmNames)) {
              // authenticated user with the different realm - continue with the others
              continue;
            }
            return info;
          }

          log.trace("Realm [{}] returned null when authenticating token [{}]", realm, token);
        }
        catch (DisabledAccountException e) {
          logExceptionForRealm(e, realm);
          authenticationFailureReasons.add(AuthenticationFailureReason.DISABLED_ACCOUNT);
        }
        catch (ExpiredCredentialsException e) {
          logExceptionForRealm(e, realm);
          authenticationFailureReasons.add(AuthenticationFailureReason.EXPIRED_CREDENTIALS);
        }
        catch (IncorrectCredentialsException e) {
          logExceptionForRealm(e, realm);
          authenticationFailureReasons.add(AuthenticationFailureReason.INCORRECT_CREDENTIALS);
        }
        catch (UnknownAccountException e) {
          logExceptionForRealm(e, realm);
          authenticationFailureReasons.add(AuthenticationFailureReason.USER_NOT_FOUND);
        }
        catch (CredentialsException e) {
          logExceptionForRealm(e, realm);
          authenticationFailureReasons.add(AuthenticationFailureReason.PASSWORD_EMPTY);
        }
        catch (AuthenticationException e) {
          logExceptionForRealm(e, realm);
          authenticationFailureReasons.add(AuthenticationFailureReason.UNKNOWN);
        }
        catch (DataAccessException e) {
          log.warn("Infrastructure failure during authentication in realm [{}]: database unavailable", realm, e);
          throw e;
        }
        catch (Throwable t) {
          logExceptionForRealm(t, realm);
        }
      }
      else {
        log.trace("Realm of type [{}] does not support token [{}]; skipping realm", realm, token);
      }
    }

    throw new AstrumAuthenticationException("Authentication token of type [" + token.getClass()
        + "] could not be authenticated by any configured realms.  Please ensure that at least one realm can "
        + "authenticate these tokens.", authenticationFailureReasons);
  }

  private void logExceptionForRealm(Throwable t, Realm realm) {
    log.trace("Realm [{}] threw an exception during a multi-realm authentication attempt", realm, t);
  }
}
