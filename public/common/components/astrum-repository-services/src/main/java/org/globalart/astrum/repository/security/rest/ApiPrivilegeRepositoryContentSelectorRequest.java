
package org.globalart.astrum.repository.security.rest;

import java.util.Collection;

import org.globalart.astrum.repository.security.RepositoryContentSelectorPrivilegeDescriptor;
import org.globalart.astrum.security.internal.rest.AstrumSecurityApiConstants;
import org.globalart.astrum.security.privilege.Privilege;
import org.globalart.astrum.security.privilege.rest.PrivilegeAction;

import io.swagger.v3.oas.annotations.media.Schema;
import jakarta.validation.constraints.NotBlank;

/**
 * @since 3.19
 */
public class ApiPrivilegeRepositoryContentSelectorRequest
    extends ApiPrivilegeWithRepositoryRequest
{
  public static final String CSEL_KEY = "contentSelector";

  @NotBlank
  @Schema(description = AstrumSecurityApiConstants.PRIVILEGE_CONTENT_SELECTOR_DESCRIPTION)
  private String contentSelector;

  /**
   * for deserialization
   */
  private ApiPrivilegeRepositoryContentSelectorRequest() {
    super();
  }

  public ApiPrivilegeRepositoryContentSelectorRequest(
      final String name,
      final String description,
      final String format,
      final String repository,
      final String contentSelector,
      final Collection<PrivilegeAction> actions)
  {
    super(name, description, format, repository, actions);
    this.contentSelector = contentSelector;
  }

  public ApiPrivilegeRepositoryContentSelectorRequest(final Privilege privilege) {
    super(privilege);
    contentSelector = privilege.getPrivilegeProperty(CSEL_KEY);
  }

  public void setContentSelector(final String contentSelector) {
    this.contentSelector = contentSelector;
  }

  public String getContentSelector() {
    return contentSelector;
  }

  @Override
  protected Privilege doAsPrivilege(final Privilege privilege) {
    super.doAsPrivilege(privilege);
    privilege.setType(RepositoryContentSelectorPrivilegeDescriptor.TYPE);
    privilege.addProperty(CSEL_KEY, contentSelector);
    return privilege;
  }
}
