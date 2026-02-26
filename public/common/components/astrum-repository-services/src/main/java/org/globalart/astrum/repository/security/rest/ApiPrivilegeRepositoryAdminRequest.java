
package org.globalart.astrum.repository.security.rest;

import java.util.Collection;

import org.globalart.astrum.repository.security.RepositoryAdminPrivilegeDescriptor;
import org.globalart.astrum.security.privilege.Privilege;
import org.globalart.astrum.security.privilege.rest.PrivilegeAction;

/**
 * @since 3.19
 */
public class ApiPrivilegeRepositoryAdminRequest
    extends ApiPrivilegeWithRepositoryRequest
{
  /**
   * for deserialization
   */
  private ApiPrivilegeRepositoryAdminRequest() {
    super();
  }

  public ApiPrivilegeRepositoryAdminRequest(
      final String name,
      final String description,
      final String format,
      final String repository,
      final Collection<PrivilegeAction> actions)
  {
    super(name, description, format, repository, actions);
  }

  public ApiPrivilegeRepositoryAdminRequest(final Privilege privilege) {
    super(privilege);
  }

  @Override
  protected Privilege doAsPrivilege(final Privilege privilege) {
    super.doAsPrivilege(privilege);
    privilege.setType(RepositoryAdminPrivilegeDescriptor.TYPE);
    return privilege;
  }
}
