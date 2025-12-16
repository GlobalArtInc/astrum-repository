
package org.apache.shiro.astrum;

import java.util.Optional;
import java.util.Set;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.inject.Provider;

import org.globalart.astrum.common.time.Time;
import org.globalart.astrum.cache.CacheHelper;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.security.UserIdMdcHelper;
import org.globalart.astrum.security.authc.AuthenticationEvent;
import org.globalart.astrum.security.authc.AuthenticationFailureReason;
import org.globalart.astrum.security.authc.LoginEvent;
import org.globalart.astrum.security.authc.AstrumAuthenticationException;

import org.apache.shiro.authc.AuthenticationException;
import org.apache.shiro.authc.AuthenticationToken;
import org.apache.shiro.session.mgt.eis.CachingSessionDAO;
import org.apache.shiro.session.mgt.eis.SessionDAO;
import org.apache.shiro.subject.Subject;
import org.apache.shiro.web.mgt.DefaultWebSecurityManager;
import org.apache.shiro.web.mgt.WebSecurityManager;
import org.springframework.beans.factory.annotation.Value;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Collections.emptySet;
import static org.globalart.astrum.common.app.ManagedLifecycleManager.isShuttingDown;

/**
 * Custom {@link WebSecurityManager}.
 *
 * @since 2.7.2
 */
public class AstrumWebSecurityManager
    extends DefaultWebSecurityManager
{
  private final Provider<EventManager> eventManager;

  @Autowired
  public AstrumWebSecurityManager(
      final Provider<EventManager> eventManager,
      final Provider<CacheHelper> cacheHelper,
      @Value("${astrum.shiro.cache.defaultTimeToLive:2m}") final Provider<Time> defaultTimeToLive)
  {
    this.eventManager = checkNotNull(eventManager);
    setCacheManager(new ShiroJCacheManagerAdapter(cacheHelper, defaultTimeToLive));
    // explicitly disable rememberMe
    this.setRememberMeManager(null);
  }

  /**
   * Post {@link AuthenticationEvent}.
   */
  private void post(
      final AuthenticationToken token,
      final boolean successful,
      final Set<AuthenticationFailureReason> authenticationFailureReasons)
  {
    eventManager.get()
        .post(new AuthenticationEvent(token.getPrincipal().toString(), successful, authenticationFailureReasons));
  }

  /**
   * After login set the userId MDC attribute.
   */
  @Override
  public Subject login(Subject subject, final AuthenticationToken token) {
    // anonymous user isn't allowed to authenticate
    if ("anonymous".equals(token.getPrincipal())) {
      throw new AuthenticationException("Cannot login with anonymous user");
    }
    try {
      subject = super.login(subject, token);
      UserIdMdcHelper.set(subject);
      post(token, true, emptySet());
      Optional<String> realmName = subject.getPrincipals()
          .getRealmNames()
          .stream()
          .filter(realm -> realm.equals("SamlRealm"))
          .findFirst();
      String principal = subject.getPrincipal().toString();
      realmName.ifPresent(realm -> eventManager.get().post(new LoginEvent(principal, realm)));

      return subject;
    }
    catch (AstrumAuthenticationException e) {
      post(token, false, e.getAuthenticationFailureReasons());
      throw e;
    }
    catch (AuthenticationException e) {
      post(token, false, emptySet());
      throw e;
    }
  }

  /**
   * After logout unset the userId MDC attribute.
   */
  @Override
  public void logout(final Subject subject) {
    super.logout(subject);
    UserIdMdcHelper.unset();
  }

  @Override
  public void destroy() {
    // underlying manager cannot be restarted, so avoid shutting it down when bouncing the service
    if (isShuttingDown()) {
      super.destroy();
    }
    else {
      // null out the session cache to force it to be recreated on the next request after bouncing
      SessionDAO sessionDAO = ((AstrumWebSessionManager) getSessionManager()).getSessionDAO();
      if (sessionDAO instanceof CachingSessionDAO) {
        ((CachingSessionDAO) sessionDAO).setActiveSessionsCache(null);
      }
    }
  }
}
