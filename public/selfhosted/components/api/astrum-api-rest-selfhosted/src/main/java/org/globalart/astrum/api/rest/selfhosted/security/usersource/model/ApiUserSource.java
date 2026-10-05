

package org.globalart.astrum.api.rest.selfhosted.security.usersource.model;

import org.globalart.astrum.security.user.UserManager;

/**
 * @since 3.17
 */
public class ApiUserSource
{
  private String id;

  private String name;

  public ApiUserSource(final UserManager manager) {
    this.id = manager.getSource();
    this.name = manager.getAuthenticationRealmName();
  }

  public String getId() {
    return id;
  }

  public String getName() {
    return name;
  }
}
