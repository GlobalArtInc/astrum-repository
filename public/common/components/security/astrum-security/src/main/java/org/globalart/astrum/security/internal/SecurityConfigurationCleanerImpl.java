
package org.globalart.astrum.security.internal;

import java.util.ConcurrentModificationException;
import java.util.Set;

import org.globalart.astrum.security.config.CRole;
import org.globalart.astrum.security.config.CUserRoleMapping;
import org.globalart.astrum.security.config.SecurityConfiguration;
import org.globalart.astrum.security.config.SecurityConfigurationCleaner;
import org.globalart.astrum.security.role.NoSuchRoleException;
import org.globalart.astrum.security.user.NoSuchRoleMappingException;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Default {@link SecurityConfigurationCleaner}.
 *
 * Removes dead references to roles and permissions in the security model.
 *
 * When a permission is removed all roles will be updated so the permission reference can removed.
 *
 * When a Role is removed references are removed from other roles and users.
 */
@Component
public class SecurityConfigurationCleanerImpl
    implements SecurityConfigurationCleaner
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public void privilegeRemoved(SecurityConfiguration configuration, String privilegeId) {
    log.debug("Cleaning privilege id {} from roles.", privilegeId);

    // Get roles that contain this privilege - O(1) lookup instead of O(n) scan
    Set<String> affectedRoleIds = configuration.getRolesByPrivilege(privilegeId);

    for (String roleId : affectedRoleIds) {
      boolean concurrentlyUpdated;
      do {
        concurrentlyUpdated = false;
        CRole currentRole = configuration.getRole(roleId);
        if (currentRole != null && currentRole.getPrivileges().contains(privilegeId)) {
          log.debug("removing privilege {} from role {}", privilegeId, currentRole.getId());
          currentRole.removePrivilege(privilegeId);
          try {
            configuration.updateRole(currentRole);
          }
          catch (NoSuchRoleException e) {
            // role was removed in the meantime
          }
          catch (ConcurrentModificationException e) {
            concurrentlyUpdated = true;
          }
        }
      }
      while (concurrentlyUpdated);
    }
  }

  public void roleRemoved(SecurityConfiguration configuration, String roleId) {
    log.debug("Cleaning role id {} from users and roles.", roleId);
    for (CRole role : configuration.getRoles()) {
      boolean concurrentlyUpdated;
      do {
        concurrentlyUpdated = false;
        CRole currentRole = configuration.getRole(role.getId());
        if (currentRole != null && currentRole.getRoles().contains(roleId)) {
          log.debug("removing ref to role {} from role {}", roleId, currentRole.getId());
          currentRole.removeRole(roleId);
          try {
            configuration.updateRole(currentRole);
          }
          catch (NoSuchRoleException e) {
            // role was removed in the meantime
          }
          catch (ConcurrentModificationException e) {
            concurrentlyUpdated = true;
          }
        }
      }
      while (concurrentlyUpdated);
    }

    for (CUserRoleMapping mapping : configuration.getUserRoleMappings()) {
      boolean concurrentlyUpdated;
      do {
        concurrentlyUpdated = false;
        CUserRoleMapping currentMapping = configuration.getUserRoleMapping(mapping.getUserId(), mapping.getSource());
        if (currentMapping != null && currentMapping.getRoles().contains(roleId)) {
          log.debug("removing ref to role {} from user {}", roleId, currentMapping.getUserId());
          currentMapping.removeRole(roleId);
          try {
            configuration.updateUserRoleMapping(currentMapping);
          }
          catch (NoSuchRoleMappingException e) {
            // mapping was removed in the meantime
          }
          catch (ConcurrentModificationException e) {
            concurrentlyUpdated = true;
          }
        }
      }
      while (concurrentlyUpdated);
    }
  }
}
