
package org.globalart.astrum.coreui;

import com.codahale.metrics.annotation.ExceptionMetered;
import com.codahale.metrics.annotation.Timed;
import com.google.common.collect.Maps;
import com.softwarementors.extjs.djn.config.annotations.DirectAction;
import com.softwarementors.extjs.djn.config.annotations.DirectMethod;
import org.apache.shiro.authz.AuthorizationException;
import org.apache.shiro.authz.Permission;
import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;

import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.extdirect.DirectComponent;
import org.globalart.astrum.extdirect.DirectComponentSupport;
import org.globalart.astrum.extdirect.model.PagedResponse;
import org.globalart.astrum.extdirect.model.StoreLoadParameters;
import org.globalart.astrum.formfields.FormField;
import org.globalart.astrum.security.SecurityHelper;
import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.security.authz.AuthorizationManager;
import org.globalart.astrum.security.authz.NoSuchAuthorizationManagerException;
import org.globalart.astrum.security.config.CPrivilege;
import org.globalart.astrum.security.config.CPrivilegeBuilder;
import org.globalart.astrum.security.privilege.Privilege;
import org.globalart.astrum.security.privilege.PrivilegeDescriptor;
import org.globalart.astrum.security.privilege.ReadonlyPrivilegeException;
import org.globalart.astrum.validation.Validate;
import org.globalart.astrum.validation.group.Create;
import org.globalart.astrum.validation.group.Update;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.validation.Valid;
import jakarta.validation.constraints.NotEmpty;
import jakarta.validation.constraints.NotNull;
import jakarta.validation.groups.Default;
import java.lang.reflect.Field;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.stream.Collectors;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.security.user.UserManager.DEFAULT_SOURCE;
import org.springframework.stereotype.Component;

/**
 * Privilege {@link DirectComponent}
 */
@Component
@DirectAction(action = "coreui_Privilege")
public class PrivilegeComponent
    extends DirectComponentSupport
{
  private final SecuritySystem securitySystem;

  private final SecurityHelper securityHelper;

  private final List<PrivilegeDescriptor> privilegeDescriptors;

  private final Map<String, PrivilegeDescriptor> privilegeDescriptorsByType;

  @Autowired
  public PrivilegeComponent(
      final SecuritySystem securitySystem,
      final SecurityHelper securityHelper,
      final List<PrivilegeDescriptor> privilegeDescriptors)
  {
    this.securitySystem = checkNotNull(securitySystem);
    this.securityHelper = checkNotNull(securityHelper);
    this.privilegeDescriptors = checkNotNull(privilegeDescriptors);
    this.privilegeDescriptorsByType = QualifierUtil.buildQualifierBeanMap(privilegeDescriptors);
  }

  /**
   * Retrieves privileges.
   *
   * @return a list of privileges
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresPermissions("astrum:privileges:read")
  public PagedResponse<PrivilegeXO> read(final StoreLoadParameters parameters) {
    List<PrivilegeXO> privileges = securitySystem.listPrivileges()
        .stream()
        .map(this::convert)
        .collect(Collectors.toList()); // NOSONAR
    return extractPage(parameters, privileges);
  }

  /**
   * Retrieves privileges and extracts name and id fields only.
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresPermissions("astrum:privileges:read")
  public List<ReferenceXO> readReferences() {
    return securitySystem.listPrivileges()
        .stream()
        .map(privilege -> new ReferenceXO(privilege.getId(), privilege.getName()))
        .collect(Collectors.toList()); // NOSONAR
  }

  /**
   * Return only those records matching the given parameters. Will apply a filter and/or sort to client-exposed
   * properties of {@link PrivilegeXO}: name, description, permission, type.
   */
  @RequiresPermissions("astrum:privileges:read")
  public PagedResponse<PrivilegeXO> extractPage(final StoreLoadParameters parameters, final List<PrivilegeXO> xos) {
    log.trace("requesting page with parameters: {} and size of: ${}", parameters, xos.size());

    checkArgument(parameters.getStart() == null || parameters.getStart() == 0 || parameters.getStart() < xos.size(),
        "Requested to skip more results than available");

    List<PrivilegeXO> result = new ArrayList<>(xos);
    if (parameters.getFilter() != null && !parameters.getFilter().isEmpty()) {
      String filter = parameters.getFilter().get(0).getValue();
      result = xos.stream()
          .filter(xo -> xo.getName().contains(filter) || xo.getDescription().contains(filter) ||
              (xo.getPermission() != null && xo.getPermission().contains(filter)) || xo.getType().contains(filter))
          .collect(Collectors.toList());
    }

    if (parameters.getSort() != null && !parameters.getSort().isEmpty()) {
      // assume one sort, not multiple props
      boolean ascending = "ASC".equals(parameters.getSort().get(0).getDirection());
      String sortProperty = parameters.getSort().get(0).getProperty();
      result.sort((a, b) -> {
        int comparison = getFieldValue(a, sortProperty).compareTo(getFieldValue(b, sortProperty));
        return ascending ? comparison : -comparison;
      });
    }

    int size = result.size();
    int start = parameters.getStart() != null ? parameters.getStart() : 0;
    int limit = parameters.getLimit() != null ? parameters.getLimit() : size;
    int end = Math.min(start + limit, size);
    List<PrivilegeXO> page = result.subList(start, end);
    return new PagedResponse<>(size, page);
  }

  /**
   * Retrieve available privilege types.
   *
   * @return a list of privilege types
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresPermissions("astrum:privileges:read")
  public List<PrivilegeTypeXO> readTypes() {
    return privilegeDescriptors.stream()
        .map(descriptor -> {
          PrivilegeTypeXO xo = new PrivilegeTypeXO();
          xo.setId(descriptor.getType());
          xo.setName(descriptor.getName());
          xo.setFormFields(convertFormFields(descriptor));
          return xo;
        })
        .collect(Collectors.toList()); // NOSONAR
  }

  /**
   * Creates a privilege.
   *
   * @param privilege the privilege info
   * @return the created privilege XO
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresAuthentication
  @RequiresPermissions("astrum:privileges:create")
  @Validate(groups = {Create.class, Default.class})
  public PrivilegeXO create(@NotNull @Valid final PrivilegeXO privilege) throws NoSuchAuthorizationManagerException {
    AuthorizationManager authorizationManager = securitySystem.getAuthorizationManager(DEFAULT_SOURCE);
    privilege.withId(
        privilege.getName()); // Use name as privilege ID (note: eventually IDs should go away in favor of names)
    checkGrantRights(privilege);
    return convert(authorizationManager.addPrivilege(convert(privilege)));
  }

  /**
   * Updates a privilege.
   *
   * @param privilege the privilege info
   * @return the updated privilege XO
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresAuthentication
  @RequiresPermissions("astrum:privileges:update")
  @Validate(groups = {Update.class, Default.class})
  public PrivilegeXO update(
      @NotNull @Valid final PrivilegeXO privilege) throws NoSuchAuthorizationManagerException, IllegalAccessException
  {
    try {
      AuthorizationManager authorizationManager = securitySystem.getAuthorizationManager(DEFAULT_SOURCE);
      checkGrantRights(privilege);
      return convert(authorizationManager.updatePrivilege(convert(privilege)));
    }
    catch (ReadonlyPrivilegeException e) {
      throw new IllegalAccessException("Privilege [" + privilege.getId() + "] is readonly and cannot be updated");
    }
  }

  /**
   * Reject mutations whose effective permission the caller does not already hold.
   * Prevents privilege self-escalation (ASTRUM-53341).
   * Fails closed when the privilege type is unknown so an unregistered type
   * cannot bypass the check.
   */
  private void checkGrantRights(final PrivilegeXO privilege) {
    PrivilegeDescriptor descriptor = privilegeDescriptorsByType.get(privilege.getType());
    if (descriptor == null) {
      log.warn("Rejected privilege '{}' mutation: unknown type '{}' has no descriptor to resolve grant rights.",
          privilege.getName(), privilege.getType());
      throw new AuthorizationException(
          "Unknown privilege type '" + privilege.getType() + "'; cannot determine effective permission.");
    }
    Permission resolved = descriptor.createPermission(toCPrivilege(privilege));
    checkNotNull(resolved, "PrivilegeDescriptor for type '%s' returned a null permission", privilege.getType());
    securityHelper.ensurePermitted(resolved);
  }

  private static CPrivilege toCPrivilege(final PrivilegeXO source) {
    String id = source.getId() != null ? source.getId() : source.getName();
    CPrivilegeBuilder builder = new CPrivilegeBuilder()
        .type(source.getType())
        .id(id)
        .name(source.getName() != null ? source.getName() : id)
        .description(source.getDescription() == null ? "" : source.getDescription());
    if (source.getProperties() != null) {
      for (Map.Entry<String, String> entry : source.getProperties().entrySet()) {
        builder.property(entry.getKey(), entry.getValue());
      }
    }
    return builder.create();
  }

  /**
   * Deletes a privilege, if is not readonly.
   *
   * @param id of privilege to be deleted
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresAuthentication
  @RequiresPermissions("astrum:privileges:delete")
  @Validate
  public void remove(@NotEmpty final String id) throws NoSuchAuthorizationManagerException, IllegalAccessException {
    AuthorizationManager authorizationManager = securitySystem.getAuthorizationManager(DEFAULT_SOURCE);
    try {
      authorizationManager.deletePrivilege(id);
    }
    catch (ReadonlyPrivilegeException e) {
      throw new IllegalAccessException("Privilege [" + id + "] is readonly and cannot be deleted");
    }
  }

  /**
   * Convert privilege to XO.
   * If the privilege type has no registered {@link PrivilegeDescriptor} (orphaned/unknown type),
   * the permission field will be set to {@code "<unknown>"}.
   */
  PrivilegeXO convert(Privilege input) {
    String permission = input.getPermission() != null ? input.getPermission().toString() : "<unknown>";
    if ("<unknown>".equals(permission)) {
      log.debug("Privilege '{}' has unknown type '{}' with no registered PrivilegeDescriptor; " +
          "permission will be reported as <unknown>", input.getId(), input.getType());
    }
    return new PrivilegeXO()
        .withId(input.getId())
        .withVersion(String.valueOf(input.getVersion()))
        .withName(input.getName() != null ? input.getName() : input.getId())
        .withDescription(input.getDescription() != null ? input.getDescription() : input.getId())
        .withType(input.getType())
        .withReadOnly(input.isReadOnly())
        .withProperties(Maps.newHashMap(input.getProperties()))
        .withPermission(permission);
  }

  /**
   * Convert XO to privilege.
   */
  Privilege convert(PrivilegeXO input) {
    Privilege privilege = new Privilege();
    privilege.setId(input.getId());
    privilege.setVersion(input.getVersion().isEmpty() ? 0 : Integer.parseInt(input.getVersion()));
    privilege.setName(input.getName());
    privilege.setDescription(input.getDescription());
    privilege.setType(input.getType());
    privilege.setProperties(Maps.newHashMap(input.getProperties()));
    return privilege;
  }

  private Comparable getFieldValue(Object obj, String fieldName) {
    try {
      Field field = obj.getClass().getDeclaredField(fieldName);
      field.setAccessible(true);
      return (Comparable<?>) field.get(obj);
    }
    catch (NoSuchFieldException | IllegalAccessException e) {
      return null;
    }
  }

  private List<FormFieldXO> convertFormFields(final PrivilegeDescriptor descriptor) {
    if (descriptor.getFormFields() == null) {
      return null;
    }

    return (List<FormFieldXO>) descriptor.getFormFields()
        .stream()
        .map(f -> FormFieldXO.create((FormField) f))
        .collect(Collectors.toList()); // NOSONAR
  }
}
