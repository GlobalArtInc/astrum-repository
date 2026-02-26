
package org.globalart.astrum.repository.security.rest;

import java.util.Collection;

import org.globalart.astrum.repository.security.RepositoryViewPrivilegeDescriptor;
import org.globalart.astrum.security.privilege.Privilege;
import org.globalart.astrum.security.privilege.rest.PrivilegeAction;

/**
 * @since 3.19
 */
public class ApiPrivilegeRepositoryView
    extends ApiPrivilegeWithRepository
{
  /**
   * for deserialization
   */
  private ApiPrivilegeRepositoryView() {
    super(RepositoryViewPrivilegeDescriptor.TYPE);
  }

  public ApiPrivilegeRepositoryView(
      final String name,
      final String description,
      final boolean readOnly,
      final String format,
      final String repository,
      final Collection<PrivilegeAction> actions)
  {
    super(RepositoryViewPrivilegeDescriptor.TYPE, name, description, readOnly, format, repository, actions);
  }

  public ApiPrivilegeRepositoryView(final Privilege privilege) {
    super(privilege);
  }
}
