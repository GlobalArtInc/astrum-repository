
package org.globalart.astrum.plugins.defaultrole.internal;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.plugins.defaultrole.DefaultRoleRealm;
import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.security.realm.RealmManager;
import org.globalart.astrum.security.role.Role;

import com.codahale.metrics.health.HealthCheck;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.security.user.UserManager.DEFAULT_SOURCE;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Will throw up a health check error when the default role in the realm isn't available
 *
 * @since 3.22
 */
@Component
@Qualifier("DefaultRoleRealm")
public class DefaultRoleHealthCheck
    extends HealthCheck
{
  private final RealmManager realmManager;

  private final DefaultRoleRealm defaultRoleRealm;

  private final SecuritySystem securitySystem;

  @Autowired
  public DefaultRoleHealthCheck(
      final RealmManager realmManager,
      final DefaultRoleRealm defaultRoleRealm,
      final SecuritySystem securitySystem)
  {
    this.realmManager = checkNotNull(realmManager);
    this.defaultRoleRealm = checkNotNull(defaultRoleRealm);
    this.securitySystem = checkNotNull(securitySystem);
  }

  @Override
  protected Result check() throws Exception {
    if (!realmManager.isRealmEnabled(DefaultRoleRealm.NAME)) {
      return Result.healthy("Default Role Realm not in use.");
    }

    if (defaultRoleRealm.getRole() == null) {
      return Result.unhealthy("Default Role Realm is enabled but not configured.");
    }

    Role matched = securitySystem.listRoles(DEFAULT_SOURCE)
        .stream()
        .filter(role -> role.getRoleId().equals(defaultRoleRealm.getRole()))
        .findFirst()
        .orElse(null);

    if (matched == null) {
      return Result
          .unhealthy("Default Role Realm configured to use role %s which doesn't exist.", defaultRoleRealm.getRole());
    }

    return Result.healthy("Default Role Realm configured to use role %s.", defaultRoleRealm.getRole());
  }
}
