
package org.globalart.astrum.api.extdirect.common.security.role;

import java.util.List;
import java.util.stream.Collectors;
import org.springframework.beans.factory.annotation.Autowired;
import jakarta.validation.Valid;
import jakarta.validation.constraints.NotEmpty;
import jakarta.validation.constraints.NotNull;
import jakarta.validation.groups.Default;

import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.extdirect.DirectComponent;
import org.globalart.astrum.extdirect.DirectComponentSupport;
import org.globalart.astrum.api.extdirect.common.security.model.ReferenceXO;
import org.globalart.astrum.api.extdirect.common.security.role.model.RoleXO;
import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.security.authz.AuthorizationManager;
import org.globalart.astrum.security.authz.NoSuchAuthorizationManagerException;
import org.globalart.astrum.security.role.Role;
import org.globalart.astrum.validation.Validate;
import org.globalart.astrum.validation.group.Create;
import org.globalart.astrum.validation.group.Update;

import com.codahale.metrics.annotation.ExceptionMetered;
import com.codahale.metrics.annotation.Timed;
import com.softwarementors.extjs.djn.config.annotations.DirectAction;
import com.softwarementors.extjs.djn.config.annotations.DirectMethod;
import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.security.user.UserManager.DEFAULT_SOURCE;
import org.springframework.stereotype.Component;

/**
 * Role {@link DirectComponent}.
 */
@Component
@DirectAction(action = "coreui_Role")
public class RoleComponent
    extends DirectComponentSupport
{
  private final SecuritySystem securitySystem;

  private final List<AuthorizationManager> authorizationManagers;

  @Autowired
  public RoleComponent(final SecuritySystem securitySystem, final List<AuthorizationManager> authorizationManagers) {
    this.securitySystem = checkNotNull(securitySystem);
    this.authorizationManagers = checkNotNull(authorizationManagers);
  }

  /**
   * Retrieves roles from all available {@link AuthorizationManager}s.
   *
   * @return a list of roles
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresPermissions("astrum:roles:read")
  public List<RoleXO> read() throws NoSuchAuthorizationManagerException {
    return securitySystem.listRoles(DEFAULT_SOURCE)
        .stream()
        .map(this::convert)
        .collect(Collectors.toList()); // NOSONAR
  }

  /**
   * Retrieves role references from all available {@link AuthorizationManager}s.
   *
   * @return a list of role references
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresPermissions("astrum:roles:read")
  public List<ReferenceXO> readReferences() throws NoSuchAuthorizationManagerException {
    return securitySystem.listRoles(DEFAULT_SOURCE)
        .stream()
        .map(input -> new ReferenceXO(input.getRoleId(), input.getName()))
        .collect(Collectors.toList()); // NOSONAR
  }

  /**
   * Retrieves available role sources.
   *
   * @return list of sources
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  public List<ReferenceXO> readSources() {
    return authorizationManagers.stream()
        .filter(manager -> !DEFAULT_SOURCE.equals(manager.getSource()))
        .map(manager -> new ReferenceXO(manager.getSource(), manager.getSource()))
        .collect(Collectors.toList()); // NOSONAR
  }

  /**
   * Retrieves roles from specified source.
   *
   * @param source to retrieve roles from
   * @return a list of roles
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresPermissions("astrum:roles:read")
  @Validate
  public List<RoleXO> readFromSource(@NotEmpty final String source) throws NoSuchAuthorizationManagerException {
    return securitySystem.listRoles(source)
        .stream()
        .map(this::convert)
        .collect(Collectors.toList()); // NOSONAR
  }

  /**
   * Creates a role.
   *
   * @param roleXO to be created
   * @return created role
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresAuthentication
  @RequiresPermissions("astrum:roles:create")
  @Validate(groups = {Create.class, Default.class})
  public RoleXO create(@NotNull @Valid final RoleXO roleXO) throws NoSuchAuthorizationManagerException {
    // HACK: Temporary validation for external role IDs to support editable text entry in combo box (LDAP only)
    if ("LDAP".equals(roleXO.getSource())) {
      securitySystem.getAuthorizationManager(roleXO.getSource()).getRole(roleXO.getId());
    }
    return convert(securitySystem.getAuthorizationManager(DEFAULT_SOURCE)
        .addRole(
            new Role(
                roleXO.getId(),
                roleXO.getName(),
                roleXO.getDescription(),
                roleXO.getSource(),
                false,
                roleXO.getRoles(),
                roleXO.getPrivileges())));
  }

  /**
   * Updates a role.
   *
   * @param roleXO to be updated
   * @return updated role
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresAuthentication
  @RequiresPermissions("astrum:roles:update")
  @Validate(groups = {Update.class, Default.class})
  public RoleXO update(@NotNull @Valid final RoleXO roleXO) throws NoSuchAuthorizationManagerException {
    Role roleToUpdate = new Role();
    roleToUpdate.setRoleId(roleXO.getId());
    roleToUpdate.setName(roleXO.getName());
    roleToUpdate.setDescription(roleXO.getDescription());
    roleToUpdate.setSource(roleXO.getSource());
    roleToUpdate.setReadOnly(false);
    roleToUpdate.setRoles(roleXO.getRoles());
    roleToUpdate.setPrivileges(roleXO.getPrivileges());
    roleToUpdate.setVersion(Integer.parseInt(roleXO.getVersion()));
    return convert(securitySystem.getAuthorizationManager(DEFAULT_SOURCE)
        .updateRole(roleToUpdate));
  }

  /**
   * Deletes a role.
   *
   * @param id of role to be deleted
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresAuthentication
  @RequiresPermissions("astrum:roles:delete")
  @Validate
  public void remove(@NotEmpty final String id) throws NoSuchAuthorizationManagerException {
    securitySystem.getAuthorizationManager(DEFAULT_SOURCE).deleteRole(id);
  }

  /**
   * Convert role to XO.
   */
  private RoleXO convert(final Role input) {
    RoleXO roleXO = new RoleXO();
    roleXO.setId(input.getRoleId());
    roleXO.setVersion(String.valueOf(input.getVersion()));
    roleXO.setSource((DEFAULT_SOURCE.equals(input.getSource()) ||
        Strings2.isBlank(input.getSource())) ? "Astrum" : input.getSource());
    roleXO.setName(Strings2.isBlank(input.getName()) ? input.getRoleId() : input.getName());
    roleXO.setDescription(Strings2.isBlank(input.getDescription()) ? input.getRoleId() : input.getDescription());
    roleXO.setReadOnly(input.isReadOnly());
    roleXO.setPrivileges(input.getPrivileges());
    roleXO.setRoles(input.getRoles());
    return roleXO;
  }
}
