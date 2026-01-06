
package org.globalart.astrum.plugins.defaultrole.internal;

import java.util.Map;

import org.globalart.astrum.capability.CapabilityConfigurationSupport;
import org.globalart.astrum.security.role.RoleExistsString;
import org.globalart.astrum.validation.group.Create;
import org.globalart.astrum.validation.group.CreateNonExposed;
import org.globalart.astrum.validation.group.Update;

/**
 * Simple configuration for {@link DefaultRoleCapability} containing a single roleId.
 *
 * @since 3.22
 */
public class DefaultRoleCapabilityConfiguration
    extends CapabilityConfigurationSupport
{
  public static final String P_ROLE = "role";

  @RoleExistsString(groups = {Create.class, CreateNonExposed.class, Update.class})
  private String role;

  public DefaultRoleCapabilityConfiguration(final Map<String, String> properties) {
    role = properties.get(P_ROLE);
  }

  public String getRole() {
    return role;
  }

  public void setRole(final String role) {
    this.role = role;
  }
}
