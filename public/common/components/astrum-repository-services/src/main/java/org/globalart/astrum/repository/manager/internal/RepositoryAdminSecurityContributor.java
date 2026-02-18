
package org.globalart.astrum.repository.manager.internal;

import java.util.Arrays;
import java.util.List;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.security.config.MutableSecurityContributor;
import org.globalart.astrum.security.config.SecurityConfiguration;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.security.RepositoryAdminPrivilegeDescriptor.id;
import static org.globalart.astrum.repository.security.RepositoryAdminPrivilegeDescriptor.privilege;
import static org.globalart.astrum.security.BreadActions.ADD;
import static org.globalart.astrum.security.BreadActions.BROWSE;
import static org.globalart.astrum.security.BreadActions.DELETE;
import static org.globalart.astrum.security.BreadActions.EDIT;
import static org.globalart.astrum.security.BreadActions.READ;
import static org.globalart.astrum.security.privilege.PrivilegeDescriptorSupport.ALL;
import org.springframework.stereotype.Component;

/**
 * Repository administration security contributor.
 *
 * @since 3.0
 */
@Component
public class RepositoryAdminSecurityContributor
    extends MutableSecurityContributor
{
  // TODO: Sort out role[-naming] scheme

  /**
   * Initial (static) security configuration.
   */
  @Override
  protected void initial(final SecurityConfiguration model) {
    maybeAddPrivilege(model, privilege(ALL, ALL, ALL));
    maybeAddPrivilege(model, privilege(ALL, ALL, BROWSE));
    maybeAddPrivilege(model, privilege(ALL, ALL, READ));
    maybeAddPrivilege(model, privilege(ALL, ALL, EDIT));
    maybeAddPrivilege(model, privilege(ALL, ALL, ADD));
    maybeAddPrivilege(model, privilege(ALL, ALL, DELETE));
  }

  /**
   * Add security configuration for given repository.
   */
  public void add(final Repository repository) {
    checkNotNull(repository);
    final String format = repository.getFormat().getValue();
    final String name = repository.getName();
    apply((model, configurationManager) -> {
      // no per-repo repository-admin ADD action
      maybeAddPrivilege(model, privilege(format, name, ALL));
      maybeAddPrivilege(model, privilege(format, name, BROWSE));
      maybeAddPrivilege(model, privilege(format, name, READ));
      maybeAddPrivilege(model, privilege(format, name, EDIT));
      maybeAddPrivilege(model, privilege(format, name, DELETE));
    });
  }

  /**
   * Remove security configuration for given repository.
   */
  public void remove(final Repository repository) {
    checkNotNull(repository);
    final String format = repository.getFormat().getValue();
    final String name = repository.getName();
    final List<String> privilegeIds = Arrays.asList(
        id(format, name, ALL),
        id(format, name, BROWSE),
        id(format, name, READ),
        id(format, name, EDIT),
        id(format, name, DELETE));
    apply((model, configurationManager) -> {
      // no per-repo repository-admin ADD action
      privilegeIds.forEach(model::removePrivilege);
      privilegeIds.forEach(configurationManager::cleanRemovedPrivilege);
    });
  }
}
