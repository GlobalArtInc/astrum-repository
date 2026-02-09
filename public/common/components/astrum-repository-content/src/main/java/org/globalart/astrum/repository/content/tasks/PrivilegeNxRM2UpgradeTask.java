
package org.globalart.astrum.repository.content.tasks;

import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

import org.globalart.astrum.logging.task.TaskLogging;
import org.globalart.astrum.scheduling.TaskSupport;
import org.globalart.astrum.security.config.CPrivilege;
import org.globalart.astrum.security.config.CRole;
import org.globalart.astrum.security.config.PrivilegeUtils;
import org.globalart.astrum.security.config.SecurityConfiguration;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.logging.task.TaskLogType.ASTRUM_LOG_ONLY;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

/**
 * Background task (hidden from users) that upgrade NXRM2 privileges to NXRM3 privileges updating any roles that used
 * the NXRM2.
 * All data is loaded in memory since the number of privileges and roles should not be too big.
 *
 */
@Component
@TaskLogging(ASTRUM_LOG_ONLY)
public class PrivilegeNxRM2UpgradeTask
    extends TaskSupport
{
  private final SecurityConfiguration securityConfiguration;

  @Autowired
  public PrivilegeNxRM2UpgradeTask(final SecurityConfiguration securityConfiguration) {
    this.securityConfiguration = checkNotNull(securityConfiguration);
  }

  @Override
  public String getMessage() {
    return "Upgrading NXRM2 privileges";
  }

  @Override
  protected Void execute() throws Exception {
    log.info("Starting upgrade for NXRM2 privileges");

    List<CPrivilege> nxrm2Privileges = retrieveNXRM2Privileges();
    log.info("Found {} NXRM2 privileges", nxrm2Privileges.size());

    int upgraded = 0;
    for (CPrivilege nxrm2Privilege : nxrm2Privileges) {
      try {
        upgradePrivilege(nxrm2Privilege);
        upgraded++;
        log.debug("Successfully upgraded NXRM2 privilege: {}", nxrm2Privilege.getId());
      }
      catch (Exception e) {
        log.error("Failed to upgrade NXRM2 privilege: {}", nxrm2Privilege.getId(), e);
      }
    }

    log.info("Successfully upgraded privileges: {}", upgraded);
    log.info("Failed to upgrade privileges: {}", nxrm2Privileges.size() - upgraded);
    return null;
  }

  private List<CPrivilege> retrieveNXRM2Privileges() {
    // loading all the privileges in memory since they should not be too big
    return securityConfiguration.getPrivileges()
        .stream()
        .filter(privilege ->
        // NXRM2 privileges have IDs that do not match their names
        !privilege.getId().equals(privilege.getName()))
        .toList();
  }

  private void upgradePrivilege(final CPrivilege nxrm2Privilege) {
    String newPrivilegeId = getNewPrivilegeId(nxrm2Privilege.getName());

    CPrivilege newPrivilege = securityConfiguration.newPrivilege();
    newPrivilege.setId(newPrivilegeId);
    newPrivilege.setName(newPrivilegeId);
    newPrivilege.setDescription(nxrm2Privilege.getDescription());
    newPrivilege.setType(nxrm2Privilege.getType());
    newPrivilege.setProperties(new HashMap<>(nxrm2Privilege.getProperties()));
    newPrivilege.setReadOnly(nxrm2Privilege.isReadOnly());
    newPrivilege.setVersion(nxrm2Privilege.getVersion() + 1);

    securityConfiguration.addPrivilege(newPrivilege);
    log.debug("Created new NXRM3 privilege: {}", newPrivilegeId);

    upgradeRoles(nxrm2Privilege.getId(), newPrivilegeId);

    securityConfiguration.removePrivilege(nxrm2Privilege.getId());
    log.debug("Removed NXRM2 privilege: {}", nxrm2Privilege.getId());
  }

  private String getNewPrivilegeId(final String nxrm2PrivilegeName) {
    final String cleanedName = PrivilegeUtils.cleanName(nxrm2PrivilegeName);
    String maybeNewPrivilegeId = cleanedName;
    int counter = 1;
    while (securityConfiguration.getPrivilege(maybeNewPrivilegeId) != null) {
      log.warn("Privilege ID already exists: {}, adding suffix: {}", maybeNewPrivilegeId, counter);
      maybeNewPrivilegeId = cleanedName + "_" + counter;
      counter++;
    }

    return maybeNewPrivilegeId;
  }

  private void upgradeRoles(
      final String nxrm2PrivilegeId,
      final String nxrm3PrivilegeId)
  {
    int count = 0;
    // loading all the roles in memory since they should not be too big
    for (CRole role : securityConfiguration.getRoles()) {
      Set<String> privilegesMaybeUpgrade = role.getPrivileges();
      if (privilegesMaybeUpgrade == null || !privilegesMaybeUpgrade.contains(nxrm2PrivilegeId)) {
        continue;
      }

      Set<String> privileges = new HashSet<>(privilegesMaybeUpgrade);
      privileges.remove(nxrm2PrivilegeId);
      privileges.add(nxrm3PrivilegeId);
      role.setPrivileges(privileges);
      securityConfiguration.updateRole(role);
      log.debug("Upgraded role {} to use privilege {}", role.getId(), nxrm3PrivilegeId);
      count++;
    }

    log.debug("Upgraded {} roles with NXRM2 privilege {}", count, nxrm2PrivilegeId);
  }
}
