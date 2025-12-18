
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
public class ApiPrivilegeApplicationRequest
    extends ApiPrivilegeWithActionsRequest
{
  public static final String DOMAIN_KEY = "domain";

  @NotBlank
  @Schema(description = AstrumSecurityApiConstants.PRIVILEGE_DOMAIN_DESCRIPTION)
  private String domain;

  /**
   * for deserialization
   */
  private ApiPrivilegeApplicationRequest() {
    super();
  }

  public ApiPrivilegeApplicationRequest(
      final String name,
      final String description,
      final String domain,
      final Collection<PrivilegeAction> actions)
  {
    super(name, description, actions);
    this.domain = domain;
  }

  public ApiPrivilegeApplicationRequest(final Privilege privilege) {
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
    privilege.setType(ApplicationPrivilegeDescriptor.TYPE);
    privilege.addProperty(DOMAIN_KEY, domain);
    return privilege;
  }

  @Override
  protected String doAsActionString() {
    return toCrudTaskActionString();
  }
}
