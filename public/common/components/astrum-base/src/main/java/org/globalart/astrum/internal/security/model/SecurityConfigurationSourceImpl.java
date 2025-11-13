
package org.globalart.astrum.internal.security.model;

import java.util.HashSet;
import java.util.List;
import java.util.Set;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport;
import org.globalart.astrum.security.config.CPrivilege;
import org.globalart.astrum.security.config.CRole;
import org.globalart.astrum.security.config.CUser;
import org.globalart.astrum.security.config.CUserRoleMapping;
import org.globalart.astrum.security.config.SecurityConfiguration;
import org.globalart.astrum.security.config.SecurityConfigurationSource;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.context.annotation.Primary;
import org.springframework.lang.Nullable;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.SCHEMAS;

/**
 * Default implementation of {@link SecurityConfigurationSource}.
 *
 * @since 3.21
 */
@Primary
@Component
@Qualifier("default")
@ManagedLifecycle(phase = SCHEMAS)
public class SecurityConfigurationSourceImpl
    extends StateGuardLifecycleSupport
    implements SecurityConfigurationSource
{
  private SecurityConfigurationImpl securityConfiguration;

  private SecurityConfigurationSource securityDefaults;

  @Autowired
  public SecurityConfigurationSourceImpl(
      final SecurityConfigurationImpl securityConfiguration,
      @Qualifier("static") @Nullable final SecurityConfigurationSource securityDefaults)
  {
    this.securityConfiguration = checkNotNull(securityConfiguration);
    this.securityDefaults = securityDefaults;
  }

  @Override
  protected void doStart() throws Exception {
    addDefaultConfigurations();
  }

  public void addDefaultConfigurations() {
    if (securityDefaults != null) {
      final Set<String> defaultUserIds = getDefaultUserIds();
      final SecurityConfiguration defaultConfigurations = securityDefaults.getConfiguration(defaultUserIds);

      addDefaultUsers(defaultConfigurations);
      addDefaultRoles(defaultConfigurations);
      addDefaultPrivileges(defaultConfigurations);
      addDefaultUserRoleMappings(defaultConfigurations);
    }
  }

  private void addDefaultUsers(final SecurityConfiguration defaultConfigurations) {
    log.info("Initializing default users");
    for (CUser user : defaultConfigurations.getUsers()) {
      if (securityConfiguration.getUser(user.getId()) == null) {
        securityConfiguration.addUser(user);
      }
    }
  }

  private void addDefaultRoles(final SecurityConfiguration defaultConfigurations) {
    log.info("Initializing default roles");

    for (CRole role : defaultConfigurations.getRoles()) {
      if (securityConfiguration.getRole(role.getId()) == null) {
        securityConfiguration.addRole(role);
      }
    }
  }

  private void addDefaultPrivileges(final SecurityConfiguration defaultConfigurations) {
    log.info("Initializing default privileges");

    for (CPrivilege privilege : defaultConfigurations.getPrivileges()) {
      if (securityConfiguration.getPrivilege(privilege.getId()) == null) {
        securityConfiguration.addPrivilege(privilege);
      }
    }
  }

  private void addDefaultUserRoleMappings(final SecurityConfiguration defaultConfigurations) {
    log.info("Initializing default user/role mappings");

    for (CUserRoleMapping mapping : defaultConfigurations.getUserRoleMappings()) {
      if (securityConfiguration.getUserRoleMapping(mapping.getUserId(), mapping.getSource()) == null) {
        securityConfiguration.addUserRoleMapping(mapping);
      }
    }
  }

  private Set<String> getDefaultUserIds() {
    Set<String> requiredDefaultConfigs = new HashSet<>();
    for (String userId : List.of(ADMIN, ANONYMOUS)) {
      if (securityConfiguration.getUser(userId) == null) {
        requiredDefaultConfigs.add(userId);
      }
    }
    return requiredDefaultConfigs;
  }

  @Override
  public SecurityConfiguration getConfiguration() {
    return securityConfiguration;
  }

  @Override
  public SecurityConfiguration loadConfiguration() {
    return securityConfiguration;
  }
}
