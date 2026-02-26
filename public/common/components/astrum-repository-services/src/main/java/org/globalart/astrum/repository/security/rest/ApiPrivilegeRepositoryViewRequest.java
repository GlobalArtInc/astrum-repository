
package org.globalart.astrum.repository.security.rest;

import java.util.Collection;

import org.globalart.astrum.repository.security.RepositoryViewPrivilegeDescriptor;
import org.globalart.astrum.security.privilege.Privilege;
import org.globalart.astrum.security.privilege.rest.PrivilegeAction;

/**
 * @since 3.19
 */
public class ApiPrivilegeRepositoryViewRequest
    extends ApiPrivilegeWithRepositoryRequest
{
  /**
   * for deserialization
   */
  private ApiPrivilegeRepositoryViewRequest() {
    super();
  }

  public ApiPrivilegeRepositoryViewRequest(
      final String name,
      final String description,
      final String format,
      final String repository,
      final Collection<PrivilegeAction> actions)
  {
    super(name, description, format, repository, actions);
  }

  public ApiPrivilegeRepositoryViewRequest(final Privilege privilege) {
    super(privilege);
  }

  @Override
  protected Privilege doAsPrivilege(final Privilege privilege) {
    super.doAsPrivilege(privilege);
    privilege.setType(RepositoryViewPrivilegeDescriptor.TYPE);
    return privilege;
  }
}
