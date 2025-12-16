
package org.globalart.astrum.bootstrap.security;

import java.util.Collection;

import jakarta.inject.Provider;

import org.globalart.astrum.common.time.Time;
import org.globalart.astrum.cache.CacheHelper;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.security.authc.FirstSuccessfulModularRealmAuthenticator;
import org.globalart.astrum.security.authz.ExceptionCatchingModularRealmAuthorizer;

import org.apache.shiro.authc.Authenticator;
import org.apache.shiro.authz.Authorizer;
import org.apache.shiro.authz.ModularRealmAuthorizer;
import org.apache.shiro.authz.permission.RolePermissionResolver;
import org.apache.shiro.event.EventBus;
import org.apache.shiro.event.support.DefaultEventBus;
import org.apache.shiro.mgt.SessionStorageEvaluator;
import org.apache.shiro.mgt.SessionsSecurityManager;
import org.apache.shiro.mgt.SubjectDAO;
import org.apache.shiro.astrum.AstrumSessionDAO;
import org.apache.shiro.astrum.AstrumSessionFactory;
import org.apache.shiro.astrum.AstrumSessionStorageEvaluator;
import org.apache.shiro.astrum.AstrumSubjectDAO;
import org.apache.shiro.astrum.AstrumWebSecurityManager;
import org.apache.shiro.astrum.AstrumWebSessionManager;
import org.apache.shiro.realm.Realm;
import org.apache.shiro.session.mgt.SessionFactory;
import org.apache.shiro.session.mgt.eis.SessionDAO;
import org.apache.shiro.web.config.ShiroFilterConfiguration;
import org.apache.shiro.web.filter.mgt.FilterChainManager;
import org.apache.shiro.web.filter.mgt.FilterChainResolver;
import org.apache.shiro.web.filter.mgt.PathMatchingFilterChainResolver;
import org.apache.shiro.web.mgt.DefaultWebSubjectFactory;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.ComponentScan;
import org.springframework.context.annotation.Configuration;

/**
 * @since 3.21.2
 */
@Configuration
@ComponentScan("org.apache.shiro.astrum")
public class WebSecurityConfiguration
{

  @Bean
  public SessionFactory sessionFactory() {
    return new AstrumSessionFactory();
  }

  @Bean
  public SessionStorageEvaluator sessionStorageEvaluator() {
    return new AstrumSessionStorageEvaluator();
  }

  @Bean
  public SubjectDAO subjectDAO() {
    AstrumSubjectDAO subjectDAO = new AstrumSubjectDAO();
    subjectDAO.setSessionStorageEvaluator(sessionStorageEvaluator());
    return subjectDAO;
  }

  @Bean
  public SessionDAO sessionDAO() {
    return new AstrumSessionDAO();
  }

  @Bean
  public Authenticator authenticator() {
    return new FirstSuccessfulModularRealmAuthenticator();
  }

  @Bean
  public Authorizer authorizer(
      final Collection<Realm> realms,
      final Provider<RolePermissionResolver> rolePermissionResolverProvider)
  {
    return new ExceptionCatchingModularRealmAuthorizer(realms, rolePermissionResolverProvider);
  }

  @Bean
  public ShiroFilterConfiguration shiroFilterConfiguration() {
    return new ShiroFilterConfiguration();
  }

  @Bean
  public FilterChainResolver filterChainResolver(final FilterChainManager manager) {
    PathMatchingFilterChainResolver resolver = new PathMatchingFilterChainResolver();
    resolver.setFilterChainManager(manager);
    return resolver;
  }

  @Bean
  public EventBus eventBus() {
    return new DefaultEventBus();
  }

  @Bean
  public SessionsSecurityManager securityManager(
      final Provider<EventManager> eventManager,
      final Provider<CacheHelper> cacheHelper,
      final AstrumWebSessionManager webSessionManager,
      @Value("${astrum.shiro.cache.defaultTimeToLive:2m}") final Provider<Time> defaultTimeToLive)
  {
    webSessionManager.setSessionDAO(sessionDAO());
    webSessionManager.setSessionFactory(sessionFactory());
    webSessionManager.setDeleteInvalidSessions(true);

    webSessionManager.setSessionIdCookieEnabled(true);
    webSessionManager.setSessionIdUrlRewritingEnabled(false);
    // webSessionManager.setSessionIdCookie(sessionCookieTemplate());

    AstrumWebSecurityManager securityManager = new AstrumWebSecurityManager(eventManager, cacheHelper, defaultTimeToLive);
    securityManager.setSubjectDAO(subjectDAO());
    securityManager.setSubjectFactory(new DefaultWebSubjectFactory());

    securityManager.setAuthenticator(authenticator());
    securityManager.setAuthorizer(authorizer());
    securityManager.setSessionManager(webSessionManager);
    securityManager.setEventBus(new DefaultEventBus());

    return securityManager;
  }

  protected Authorizer authorizer() {
    ModularRealmAuthorizer authorizer = new ModularRealmAuthorizer();
    //
    // if (permissionResolver != null) {
    // authorizer.setPermissionResolver(permissionResolver);
    // }
    //
    // if (rolePermissionResolver != null) {
    // authorizer.setRolePermissionResolver(rolePermissionResolver);
    // }

    return authorizer;
  }

}
