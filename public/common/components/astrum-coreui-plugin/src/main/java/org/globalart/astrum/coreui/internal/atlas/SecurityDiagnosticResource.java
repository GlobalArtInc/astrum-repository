
package org.globalart.astrum.coreui.internal.atlas;

import java.util.HashMap;
import java.util.Map;
import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.NotFoundException;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.PathParam;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.core.MediaType;

import org.globalart.astrum.rest.Resource;
import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.security.authz.AuthorizationManager;
import org.globalart.astrum.security.authz.NoSuchAuthorizationManagerException;
import org.globalart.astrum.security.privilege.NoSuchPrivilegeException;
import org.globalart.astrum.security.privilege.Privilege;
import org.globalart.astrum.security.role.NoSuchRoleException;
import org.globalart.astrum.security.role.Role;
import org.globalart.astrum.security.user.User;
import org.globalart.astrum.security.user.UserNotFoundException;

import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.coreui.internal.atlas.SecurityDiagnosticResource.RESOURCE_URI;
import org.springframework.stereotype.Component;

/**
 * Renders security diagnostic information.
 */
@Component
@Path(RESOURCE_URI)
@Produces(MediaType.APPLICATION_JSON)
public class SecurityDiagnosticResource
    implements Resource
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public static final String RESOURCE_URI = "/atlas/security-diagnostic";

  private static final String USER_FIELD = "user";

  private static final String USERID_FIELD = "userId";

  private static final String NAME_FIELD = "name";

  private static final String FIRST_NAME_FIELD = "firstName";

  private static final String LAST_NAME_FIELD = "lastName";

  private static final String EMAIL_FIELD = "emailAddress";

  private static final String DESCRIPTION_FIELD = "description";

  private static final String SOURCE_FIELD = "source";

  private static final String STATUS_FIELD = "status";

  private static final String VERSION_FIELD = "version";

  private static final String ROLES_FIELD = "roles";

  private static final String PRIVILEGES_FIELD = "privileges";

  private static final String PERMISSION_FIELD = "permission";

  private static final String PROPERTIES_FIELD = "properties";

  private final SecuritySystem securitySystem;

  @Autowired
  public SecurityDiagnosticResource(final SecuritySystem securitySystem) {
    this.securitySystem = checkNotNull(securitySystem);
  }

  /**
   * Renders security diagnostic information for a specific user.
   */
  @GET
  @Path("user/{userId}")
  @RequiresPermissions("astrum:atlas:read")
  public Map<String, Object> userDiagnostic(final @PathParam("userId") String userId) {
    try {
      log.info("Generating security diagnostics for user: {}", userId);
      Map<String, Object> userDataMap = new HashMap<>();
      populateUserDataMap(userDataMap, securitySystem.getAuthorizationManager("default"), userId);
      return userDataMap;
    }
    catch (NoSuchAuthorizationManagerException e) {
      log.debug("Default AuthorizationManager not found", e);
      throw new RuntimeException(e);
    }
  }

  private void populateUserDataMap(
      final Map<String, Object> userDataMap,
      final AuthorizationManager authorizationManager,
      final String id)
  {
    try {
      userDataMap.put(USER_FIELD, toMap(securitySystem.getUser(id), authorizationManager));
    }
    catch (UserNotFoundException e) {
      log.debug("User not found: {}", id, e);
      throw new NotFoundException("User not found");
    }
  }

  private void populateRoleDataMap(
      final Map<String, Object> roleDataMap,
      final AuthorizationManager authorizationManager,
      final String id)
  {
    try {
      roleDataMap.put(id, toMap(authorizationManager.getRole(id), authorizationManager));
    }
    catch (NoSuchRoleException e) {
      roleDataMap.put(id, "ERROR: Failed to resolve role: " + id + " caused by: " + e.getMessage());
    }
  }

  private void populatePrivilegeDataMap(
      final Map<String, Object> privilegeDataMap,
      final AuthorizationManager authorizationManager,
      final String id)
  {
    try {
      privilegeDataMap.put(id, toMap(authorizationManager.getPrivilege(id)));
    }
    catch (NoSuchPrivilegeException e) {
      privilegeDataMap.put(id, "ERROR: Failed to resolve privilege: " + id + " caused by: " + e.getMessage());
    }
  }

  private Map<String, Object> toMap(final User user, final AuthorizationManager authorizationManager) {
    Map<String, Object> userData = new HashMap<>();
    userData.put(USERID_FIELD, user.getUserId());
    userData.put(NAME_FIELD, user.getName());
    userData.put(FIRST_NAME_FIELD, user.getFirstName());
    userData.put(LAST_NAME_FIELD, user.getLastName());
    userData.put(EMAIL_FIELD, user.getEmailAddress());
    userData.put(SOURCE_FIELD, user.getSource());
    userData.put(STATUS_FIELD, user.getStatus());
    userData.put(VERSION_FIELD, user.getVersion());
    Map<String, Object> roleDataMap = new HashMap<>();
    userData.put(ROLES_FIELD, roleDataMap);
    user.getRoles()
        .forEach(childRole -> populateRoleDataMap(roleDataMap, authorizationManager, childRole.getRoleId()));
    return userData;
  }

  private Map<String, Object> toMap(final Role role, final AuthorizationManager authorizationManager) {
    Map<String, Object> roleData = new HashMap<>();
    roleData.put(NAME_FIELD, role.getName());
    roleData.put(SOURCE_FIELD, role.getSource());
    roleData.put(DESCRIPTION_FIELD, role.getDescription());
    roleData.put(VERSION_FIELD, role.getVersion());
    Map<String, Object> childRoleData = new HashMap<>();
    roleData.put(ROLES_FIELD, childRoleData);
    role.getRoles().forEach(childRole -> populateRoleDataMap(childRoleData, authorizationManager, childRole));
    Map<String, Object> privilegeData = new HashMap<>();
    roleData.put(PRIVILEGES_FIELD, privilegeData);
    role.getPrivileges()
        .forEach(privilege -> populatePrivilegeDataMap(privilegeData, authorizationManager, privilege));
    return roleData;
  }

  private Map<String, Object> toMap(final Privilege privilege) {
    Map<String, Object> privilegeData = new HashMap<>();
    privilegeData.put(DESCRIPTION_FIELD, privilege.getDescription());
    privilegeData.put(NAME_FIELD, privilege.getName());
    String permission = privilege.getPermission() != null ? privilege.getPermission().toString() : "<unknown>";
    if ("<unknown>".equals(permission)) {
      log.debug("Privilege '{}' has unknown type '{}' with no registered PrivilegeDescriptor; " +
          "permission will be reported as <unknown>", privilege.getId(), privilege.getType());
    }
    privilegeData.put(PERMISSION_FIELD, permission);
    privilegeData.put(PROPERTIES_FIELD, privilege.getProperties());
    privilegeData.put(VERSION_FIELD, privilege.getVersion());
    return privilegeData;
  }
}
