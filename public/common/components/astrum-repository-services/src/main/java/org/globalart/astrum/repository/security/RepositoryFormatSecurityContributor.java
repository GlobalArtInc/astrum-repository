
package org.globalart.astrum.repository.security;

import java.util.List;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.security.config.MutableSecurityContributor;
import org.globalart.astrum.security.config.SecurityConfiguration;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Arrays.asList;
import static org.globalart.astrum.security.BreadActions.ADD;
import static org.globalart.astrum.security.BreadActions.BROWSE;
import static org.globalart.astrum.security.BreadActions.DELETE;
import static org.globalart.astrum.security.BreadActions.EDIT;
import static org.globalart.astrum.security.BreadActions.READ;
import static org.globalart.astrum.repository.security.RepositoryViewPrivilegeDescriptor.id;
import static org.globalart.astrum.repository.security.RepositoryViewPrivilegeDescriptor.privilege;
import static org.globalart.astrum.security.privilege.PrivilegeDescriptorSupport.ALL;

/**
 * Repository format security contributor.
 *
 * @since 3.0
 */
public class RepositoryFormatSecurityContributor
    extends MutableSecurityContributor
{
  // NOTE: This is not ideal, but moving forward to allow sharing, refactor eventually

  private final Format format;

  public RepositoryFormatSecurityContributor(final Format format) {
    this.format = checkNotNull(format);
  }

  /**
   * Initial (static) security configuration.
   */
  @Override
  protected void initial(final SecurityConfiguration model) {
    String format = this.format.getValue();

    // add repository-view <format> ALL privileges
    maybeAddPrivilege(model, privilege(format, ALL, ALL));
    maybeAddPrivilege(model, privilege(format, ALL, BROWSE));
    maybeAddPrivilege(model, privilege(format, ALL, READ));
    maybeAddPrivilege(model, privilege(format, ALL, EDIT));
    maybeAddPrivilege(model, privilege(format, ALL, ADD));
    maybeAddPrivilege(model, privilege(format, ALL, DELETE));
  }

  /**
   * Add security configuration for given repository.
   */
  public void add(final Repository repository) {
    checkNotNull(repository);
    final String format = repository.getFormat().getValue();
    final String name = repository.getName();

    apply((model, configurationManager) -> {
      // add repository-view <format> <name> privileges
      maybeAddPrivilege(model, privilege(format, name, ALL));
      maybeAddPrivilege(model, privilege(format, name, BROWSE));
      maybeAddPrivilege(model, privilege(format, name, READ));
      maybeAddPrivilege(model, privilege(format, name, EDIT));
      maybeAddPrivilege(model, privilege(format, name, ADD));
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
    final List<String> privilegeIds = asList(
        id(format, name, ALL),
        id(format, name, BROWSE),
        id(format, name, READ),
        id(format, name, EDIT),
        id(format, name, ADD),
        id(format, name, DELETE));

    apply((model, configurationManager) -> {
      // remove repository-view <format> <name> privileges
      privilegeIds.forEach(model::removePrivilege);
      privilegeIds.forEach(configurationManager::cleanRemovedPrivilege);
    });
  }
}
