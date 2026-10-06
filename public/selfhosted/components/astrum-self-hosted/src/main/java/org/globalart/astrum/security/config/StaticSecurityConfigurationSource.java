
package org.globalart.astrum.security.config;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.Set;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.security.Roles;
import org.globalart.astrum.security.config.memory.MemoryCUser;
import org.globalart.astrum.security.config.memory.MemoryCUserRoleMapping;

import com.google.common.annotations.VisibleForTesting;
import org.apache.commons.lang3.StringUtils;
import org.apache.shiro.authc.credential.PasswordService;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.core.Ordered;
import org.springframework.core.annotation.Order;
import org.springframework.stereotype.Component;

// FIXME: Perhaps this would be better in astrum-core internal.security?

/**
 * Security model configuration defaults.
 *
 * @since 3.0
 */
@Component
@Qualifier("static")
@Order(Ordered.LOWEST_PRECEDENCE)
public class StaticSecurityConfigurationSource
    implements SecurityConfigurationSource
{
  private static final String ASTRUM_SECURITY_INITIAL_PASSWORD = "ASTRUM_SECURITY_INITIAL_PASSWORD";

  private final PasswordService passwordService;

  private final AdminPasswordSource adminPasswordSource;

  private final boolean randomPassword;

  private final String password;

  private SecurityConfiguration configuration;

  public static final String ANONYMOUS = "anonymous";

  @Autowired
  public StaticSecurityConfigurationSource(
      final PasswordService passwordService,
      @Nullable final AdminPasswordSource adminPasswordSource,
      @Value("${astrum.security.randompassword:true}") final boolean randomPassword)
  {
    this(passwordService, adminPasswordSource, randomPassword, System.getenv(ASTRUM_SECURITY_INITIAL_PASSWORD));
  }

  public StaticSecurityConfigurationSource(
      final PasswordService passwordService,
      final AdminPasswordSource adminPasswordSource,
      final boolean randomPassword,
      @Nullable final String password)
  {
    this.passwordService = passwordService;
    this.adminPasswordSource = adminPasswordSource;
    this.password = password;

    if (StringUtils.isBlank(password)) {
      boolean enabled = Optional.ofNullable(System.getenv("ASTRUM_SECURITY_RANDOMPASSWORD"))
          .map(Boolean::valueOf)
          .orElse(true);
      this.randomPassword = randomPassword && enabled;
    }
    else {
      this.randomPassword = false;
    }
  }

  @Override
  public SecurityConfiguration getConfiguration() {
    if (configuration != null) {
      return configuration;
    }
    return loadConfiguration();
  }

  @Override
  public SecurityConfiguration getConfiguration(final Set<String> userIds) {
    final List<MemoryCUserRoleMapping> roleMappings = new ArrayList<>();
    final List<MemoryCUser> users = new ArrayList<>();
    MemorySecurityConfiguration memorySecurityConfiguration = new MemorySecurityConfiguration();

    for (String userId : userIds) {
      if (ADMIN.equals(userId)) {
        users.add(getAdminUser());
        roleMappings.add(getAdminRoleMapping());
      }
      if (ANONYMOUS.equals(userId)) {
        users.add(getAnonymousUser());
        roleMappings.add(getAnonymousRoleMapping());
      }
    }
    return memorySecurityConfiguration
        .withUsers(users.toArray(MemoryCUser[]::new))
        .withUserRoleMappings(roleMappings.toArray(MemoryCUserRoleMapping[]::new));
  }

  @Override
  public synchronized SecurityConfiguration loadConfiguration() {
    configuration = new MemorySecurityConfiguration().withUsers(
        getAdminUser(),
        getAnonymousUser())
        .withUserRoleMappings(
            getAdminRoleMapping(),
            getAnonymousRoleMapping());
    return configuration;
  }

  private MemoryCUser getAdminUser() {
    String encryptedPassword = passwordService.encryptPassword(getPassword());
    return new MemoryCUser()
        .withId(ADMIN)
        .withPassword(encryptedPassword)
        .withFirstName("Administrator")
        .withLastName("User")
        .withStatus(randomPassword ? CUser.STATUS_CHANGE_PASSWORD : CUser.STATUS_ACTIVE)
        .withEmail("admin@example.org");
  }

  private MemoryCUser getAnonymousUser() {
    return new MemoryCUser()
        .withId(ANONYMOUS)
        .withPassword("unused")
        .withFirstName("Anonymous")
        .withLastName("User")
        .withStatus(CUser.STATUS_ACTIVE)
        .withEmail("anonymous@example.org");
  }

  private static MemoryCUserRoleMapping getAdminRoleMapping() {
    return new MemoryCUserRoleMapping()
        .withUserId(ADMIN)
        .withSource("default")
        .withRoles(Roles.ADMIN_ROLE_ID);
  }

  private static MemoryCUserRoleMapping getAnonymousRoleMapping() {
    return new MemoryCUserRoleMapping()
        .withUserId(ANONYMOUS)
        .withSource("default")
        .withRoles(Roles.ANONYMOUS_ROLE_ID);
  }

  @VisibleForTesting
  protected String getPassword() {
    if (StringUtils.isNotBlank(password)) {
      return password;
    }

    if (adminPasswordSource != null) {
      return adminPasswordSource.getPassword(randomPassword);
    }
    throw new IllegalStateException(
        "No password source available. Please set the ASTRUM_SECURITY_INITIAL_PASSWORD environment variable.");
  }
}
