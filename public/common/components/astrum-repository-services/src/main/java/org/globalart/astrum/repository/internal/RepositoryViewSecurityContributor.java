
package org.globalart.astrum.repository.internal;

import org.globalart.astrum.security.config.MemorySecurityConfiguration;
import org.globalart.astrum.security.config.SecurityConfiguration;
import org.globalart.astrum.security.config.SecurityContributor;
import org.globalart.astrum.security.config.SecurityContributorSupport;
import org.globalart.astrum.security.config.memory.MemoryCPrivilege;
import org.globalart.astrum.security.config.memory.MemoryCPrivilege.MemoryCPrivilegeBuilder;

import static org.apache.commons.lang3.StringUtils.capitalize;
import static org.globalart.astrum.repository.security.RepositoryViewPrivilegeDescriptor.P_ACTIONS;
import static org.globalart.astrum.repository.security.RepositoryViewPrivilegeDescriptor.P_FORMAT;
import static org.globalart.astrum.repository.security.RepositoryViewPrivilegeDescriptor.P_REPOSITORY;
import static org.globalart.astrum.repository.security.RepositoryViewPrivilegeDescriptor.TYPE;
import static org.globalart.astrum.security.privilege.PrivilegeDescriptorSupport.ALL;
import org.springframework.stereotype.Component;

/**
 * Repository view security configuration.
 *
 * @since 3.0
 */
@Component
public class RepositoryViewSecurityContributor
    extends SecurityContributorSupport
    implements SecurityContributor
{
  public static final String REPOSITORY_VIEW_ALL_PREFIX = "as-repository-view-*-*-";

  public static final String REPOSITORY_VIEW_ALL_DESCRIPTION_SUFFIX = " permissions for all repository views";

  public static final String ACTION_BROWSE = "browse";

  public static final String ACTION_READ = "read";

  public static final String ACTION_EDIT = "edit";

  public static final String ACTION_ADD = "add";

  public static final String ACTION_DELETE = "delete";

  @Override
  public SecurityConfiguration getContribution() {
    MemorySecurityConfiguration config = new MemorySecurityConfiguration();

    config.addPrivilege(createRepositoryViewPrivilege(ALL));
    config.addPrivilege(createRepositoryViewPrivilege(ACTION_BROWSE));
    config.addPrivilege(createRepositoryViewPrivilege(ACTION_READ));
    config.addPrivilege(createRepositoryViewPrivilege(ACTION_EDIT));
    config.addPrivilege(createRepositoryViewPrivilege(ACTION_ADD));
    config.addPrivilege(createRepositoryViewPrivilege(ACTION_DELETE));

    return config;
  }

  private MemoryCPrivilege createRepositoryViewPrivilege(
      final String action)
  {
    final String id = REPOSITORY_VIEW_ALL_PREFIX + action;
    final String description =
        (ALL.equals(action) ? "All" : capitalize(action)) + REPOSITORY_VIEW_ALL_DESCRIPTION_SUFFIX;

    return new MemoryCPrivilegeBuilder(id)
        .type(TYPE)
        .readOnly(true)
        .name(id)
        .description(description)
        .property(P_FORMAT, ALL)
        .property(P_REPOSITORY, ALL)
        .property(P_ACTIONS, action)
        .build();
  }
}
