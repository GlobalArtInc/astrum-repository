
package org.globalart.astrum.repository.security.internal;

import org.globalart.astrum.security.config.CUser;
import org.globalart.astrum.security.config.SecurityConfigurationManager;
import org.globalart.astrum.security.internal.AuthenticatingRealmImpl;
import org.globalart.astrum.security.realm.RealmManager;
import org.globalart.astrum.security.user.UserNotFoundException;

import com.codahale.metrics.health.HealthCheck;
import org.springframework.beans.factory.annotation.Autowired;
import org.apache.shiro.authc.credential.PasswordService;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.FeatureFlags.ASTRUM_SECURITY_FIPS_ENABLED;

/**
 * Check if the default user can be used to authenticate.
 *
 * Returns unhealthy if the default admin user exists with unchanged default password.
 * Returns healthy if the admin user doesn't exist or password has been changed.
 */
@Component
@Qualifier("Default Admin Credentials")
@ConditionalOnProperty(name = ASTRUM_SECURITY_FIPS_ENABLED, havingValue = "false", matchIfMissing = true)
public class DefaultUserHealthCheck
    extends HealthCheck
{
  private static final Logger log = LoggerFactory.getLogger(DefaultUserHealthCheck.class);

  static final String ERROR_MESSAGE =
      "The default admin credentials have not been changed. It is strongly recommended that the default admin password be changed.";

  private final RealmManager realmManager;

  private final SecurityConfigurationManager securityConfigurationManager;

  private final PasswordService passwordService;

  private static final String ADMIN_USERNAME = "admin";

  private static final String DEFAULT_ADMIN_PASSWORD = "admin123";

  @Autowired
  public DefaultUserHealthCheck(
      final RealmManager realmManager,
      final SecurityConfigurationManager securityConfigurationManager,
      final PasswordService passwordService)
  {
    this.realmManager = checkNotNull(realmManager);
    this.securityConfigurationManager = checkNotNull(securityConfigurationManager);
    this.passwordService = checkNotNull(passwordService);
  }

  @Override
  protected Result check() {
    if (!realmManager.isRealmEnabled(AuthenticatingRealmImpl.NAME)) {
      return Result.healthy();
    }

    try {
      CUser adminUser = securityConfigurationManager.readUser(ADMIN_USERNAME);
      if (passwordService.passwordsMatch(DEFAULT_ADMIN_PASSWORD, adminUser.getPassword())) {
        return Result.unhealthy(ERROR_MESSAGE);
      }
    }
    catch (UserNotFoundException e) {
      log.debug("Default admin user not found: {}", e.getMessage());
    }
    return Result.healthy();
  }
}
