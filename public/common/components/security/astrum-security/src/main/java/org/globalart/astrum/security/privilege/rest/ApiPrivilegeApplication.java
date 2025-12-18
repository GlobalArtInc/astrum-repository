
package org.globalart.astrum.security.privilege.rest;

import java.util.Collection;

import org.globalart.astrum.security.internal.rest.AstrumSecurityApiConstants;
import org.globalart.astrum.security.privilege.ApplicationPrivilegeDescriptor;
import org.globalart.astrum.security.privilege.Privilege;

import io.swagger.v3.oas.annotations.media.Schema;
import jakarta.validation.constraints.NotBlank;

/**
 * @since 3.19
 */
public class ApiPrivilegeApplication
    extends ApiPrivilegeWithActions
{
  public static final String DOMAIN_KEY = "domain";

  @NotBlank
  @Schema(description = AstrumSecurityApiConstants.PRIVILEGE_DOMAIN_DESCRIPTION)
  private String domain;

  /**
   * for deserialization
   */
  private ApiPrivilegeApplication() {
    super(ApplicationPrivilegeDescriptor.TYPE);
  }

  public ApiPrivilegeApplication(
      final String name,
      final String description,
      final boolean readOnly,
      final String domain,
      final Collection<PrivilegeAction> actions)
  {
    super(ApplicationPrivilegeDescriptor.TYPE, name, description, readOnly, actions);
    this.domain = domain;
  }

  public ApiPrivilegeApplication(final Privilege privilege) {
    super(privilege);
    domain = privilege.getPrivilegeProperty(DOMAIN_KEY);
  }

  public void setDomain(final String domain) {
    this.domain = domain;
  }

  public String getDomain() {
    return domain;
  }

  @Override
  protected Privilege doAsPrivilege(final Privilege privilege) {
    super.doAsPrivilege(privilege);
    privilege.addProperty(DOMAIN_KEY, domain);
    return privilege;
  }

  @Override
  protected String doAsActionString() {
    return toCrudActionString();
  }
}
