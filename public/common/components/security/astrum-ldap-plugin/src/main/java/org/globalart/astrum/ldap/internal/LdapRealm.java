package org.globalart.astrum.ldap.internal;

import java.util.Collection;
import java.util.Optional;

import org.globalart.astrum.common.Description;
import org.globalart.astrum.security.AstrumSimpleAuthenticationInfo;
import org.globalart.astrum.security.RealmCaseMapping;
import org.globalart.astrum.security.authz.PermissionCachingAuthorizingRealm;
import org.globalart.astrum.security.authz.PrincipalPermissionsCacheFactory;

import org.apache.shiro.authc.AuthenticationInfo;
import org.apache.shiro.authc.AuthenticationToken;
import org.apache.shiro.authc.UnknownAccountException;
import org.apache.shiro.authc.UsernamePasswordToken;
import org.apache.shiro.authc.credential.AllowAllCredentialsMatcher;
import org.apache.shiro.authz.AuthorizationInfo;
import org.apache.shiro.authz.SimpleAuthorizationInfo;
import org.apache.shiro.subject.PrincipalCollection;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Authenticates users against the configured LDAP servers; their LDAP groups become role ids.
 */
@Component
@Qualifier(LdapRealm.NAME)
@Description("LDAP Realm")
public class LdapRealm
    extends PermissionCachingAuthorizingRealm
{
  public static final String NAME = "LdapRealm";

  private static final Logger log = LoggerFactory.getLogger(LdapRealm.class);

  private final LdapManager ldapManager;

  @Autowired
  public LdapRealm(
      final PrincipalPermissionsCacheFactory principalPermissionsCacheFactory,
      final LdapManager ldapManager)
  {
    super(NAME, principalPermissionsCacheFactory);
    this.ldapManager = checkNotNull(ldapManager);
    // the password is checked against LDAP in doGetAuthenticationInfo
    setCredentialsMatcher(new AllowAllCredentialsMatcher());
    setAuthenticationCachingEnabled(false);
  }

  @Override
  public boolean supports(final AuthenticationToken token) {
    return token instanceof UsernamePasswordToken;
  }

  @Override
  protected AuthenticationInfo doGetAuthenticationInfo(final AuthenticationToken token) {
    UsernamePasswordToken credentials = (UsernamePasswordToken) token;
    if (credentials.getUsername() == null || credentials.getUsername().isBlank() || !ldapManager.isConfigured()) {
      // not null: for a null result Shiro checks a simulated password, which this realm's matcher accepts
      throw new UnknownAccountException("No LDAP server is configured");
    }
    LdapUser user = ldapManager.authenticate(credentials.getUsername(), credentials.getPassword());
    return new AstrumSimpleAuthenticationInfo(user.username(), credentials.getPassword(),
        new RealmCaseMapping(NAME, false));
  }

  @Override
  protected AuthorizationInfo doGetAuthorizationInfo(final PrincipalCollection principals) {
    Collection<?> own = principals.fromRealm(getName());
    if (own.isEmpty()) {
      return null;
    }
    String userId = own.iterator().next().toString();
    Optional<LdapUser> user;
    try {
      user = ldapManager.getUser(userId);
    }
    catch (LdapServerException e) {
      log.warn("Could not read the LDAP groups of {}: {}", userId, e.getMessage());
      return null;
    }
    return user
        .filter(found -> !found.groups().isEmpty())
        .map(found -> {
          SimpleAuthorizationInfo info = new SimpleAuthorizationInfo();
          info.addRoles(found.groups());
          return (AuthorizationInfo) info;
        })
        .orElse(null);
  }
}
