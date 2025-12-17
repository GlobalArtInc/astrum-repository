
package org.globalart.astrum.security.config;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

import org.globalart.astrum.security.config.memory.MemoryCPrivilege;
import org.globalart.astrum.security.config.memory.MemoryCPrivilege.MemoryCPrivilegeBuilder;
import org.globalart.astrum.security.config.memory.MemoryCRole;

import static org.apache.commons.lang3.StringUtils.capitalize;

/**
 * @since 3.37
 */
public abstract class SecurityContributorSupport
    implements SecurityContributor
{
  protected static final String TYPE_APPLICATION = "application";

  protected static final String APPLICATION_DOMAIN = "domain";

  protected static final String APPLICATION_ACTIONS = "actions";

  protected static final String ACTION_CREATE_ONLY = "create";

  protected static final String ACTION_READ = "read";

  protected static final String ACTION_UPDATE_ONLY = "update";

  protected static final String ACTION_DELETE_ONLY = "delete";

  protected static final String ACTION_CREATE = ACTION_CREATE_ONLY + "," + ACTION_READ;

  protected static final String ACTION_UPDATE = ACTION_UPDATE_ONLY + "," + ACTION_READ;

  protected static final String ACTION_DELETE = ACTION_DELETE_ONLY + "," + ACTION_READ;

  protected static final String ACTION_ALL = "*";

  protected static final String TYPE_WILDCARD = "wildcard";

  protected static final String WILDCARD_PATTERN = "pattern";

  protected static final String ALL_DESCRIPTION_BASE = "All permissions for ";

  protected static final String CREATE_DESCRIPTION_BASE = "Create permission for ";

  protected static final String READ_DESCRIPTION_BASE = "Read permission for ";

  protected static final String UPDATE_DESCRIPTION_BASE = "Update permission for ";

  protected static final String DELETE_DESCRIPTION_BASE = "Delete permission for ";

  protected MemoryCPrivilege createWildcardPrivilege(final String id, final String description, final String pattern) {
    return new MemoryCPrivilegeBuilder(id)
        .description(description)
        .type(TYPE_WILDCARD)
        .readOnly(true)
        .name(id)
        .property(WILDCARD_PATTERN, pattern)
        .build();
  }

  protected MemoryCPrivilege createApplicationPrivilege(
      final String id,
      final String description,
      final String domain,
      final String actions)
  {
    return new MemoryCPrivilegeBuilder(id)
        .description(description)
        .type(TYPE_APPLICATION)
        .readOnly(true)
        .name(id)
        .property(APPLICATION_DOMAIN, domain)
        .property(APPLICATION_ACTIONS, actions)
        .build();
  }

  protected List<MemoryCPrivilege> createCrudApplicationPrivileges(
      final String idBase,
      final String domain)
  {
    return doCreateCrudApplicationPrivileges(idBase, domain, false);
  }

  protected List<MemoryCPrivilege> createCrudAndAllApplicationPrivileges(
      final String idBase,
      final String domain)
  {
    return doCreateCrudApplicationPrivileges(idBase, domain, true);
  }

  private List<MemoryCPrivilege> doCreateCrudApplicationPrivileges(
      final String idBase,
      final String domain,
      final boolean addAll)
  {
    List<MemoryCPrivilege> results = new ArrayList<>();

    String domainUpper = capitalize(domain);

    if (addAll) {
      results.add(createApplicationPrivilege(idBase + "-all", ALL_DESCRIPTION_BASE + domainUpper, domain, ACTION_ALL));
    }
    results.add(
        createApplicationPrivilege(idBase + "-create", CREATE_DESCRIPTION_BASE + domainUpper, domain, ACTION_CREATE));
    results
        .add(createApplicationPrivilege(idBase + "-read", READ_DESCRIPTION_BASE + domainUpper, domain, ACTION_READ));
    results.add(
        createApplicationPrivilege(idBase + "-update", UPDATE_DESCRIPTION_BASE + domainUpper, domain, ACTION_UPDATE));
    results.add(
        createApplicationPrivilege(idBase + "-delete", DELETE_DESCRIPTION_BASE + domainUpper, domain, ACTION_DELETE));

    return results;
  }

  protected MemoryCRole createRole(
      final String id,
      final String name,
      final String description,
      final String... privileges)
  {
    MemoryCRole role = new MemoryCRole();
    role.setId(id);
    role.setName(name);
    role.setDescription(description);
    role.setReadOnly(true);
    Arrays.stream(privileges).forEach(role::addPrivilege);
    return role;
  }
}
