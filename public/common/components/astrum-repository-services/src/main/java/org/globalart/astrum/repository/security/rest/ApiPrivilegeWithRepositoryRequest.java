
package org.globalart.astrum.repository.security.rest;

import java.util.Collection;

import org.globalart.astrum.security.internal.rest.AstrumSecurityApiConstants;
import org.globalart.astrum.security.privilege.Privilege;
import org.globalart.astrum.security.privilege.rest.ApiPrivilegeWithActionsRequest;
import org.globalart.astrum.security.privilege.rest.PrivilegeAction;

import io.swagger.v3.oas.annotations.media.Schema;
import jakarta.validation.constraints.NotBlank;

/**
 * @since 3.19
 */
public abstract class ApiPrivilegeWithRepositoryRequest
    extends ApiPrivilegeWithActionsRequest
{
  public static final String FORMAT_KEY = "format";

  public static final String REPOSITORY_KEY = "repository";

  @NotBlank
  @Schema(description = AstrumSecurityApiConstants.PRIVILEGE_REPOSITORY_FORMAT_DESCRIPTION)
  private String format;

  @NotBlank
  @Schema(description = AstrumSecurityApiConstants.PRIVILEGE_REPOSITORY_DESCRIPTION)
  private String repository;

  public ApiPrivilegeWithRepositoryRequest() {
    super();
  }

  public ApiPrivilegeWithRepositoryRequest(
      final String name,
      final String description,
      final String format,
      final String repository,
      final Collection<PrivilegeAction> actions)
  {
    super(name, description, actions);
    this.format = format;
    this.repository = repository;
  }

  public ApiPrivilegeWithRepositoryRequest(final Privilege privilege) {
    super(privilege);
    format = privilege.getPrivilegeProperty(FORMAT_KEY);
    repository = privilege.getPrivilegeProperty(REPOSITORY_KEY);
  }

  public void setRepository(final String repository) {
    this.repository = repository;
  }

  public void setFormat(final String format) {
    this.format = format;
  }

  public String getRepository() {
    return repository;
  }

  public String getFormat() {
    return format;
  }

  @Override
  protected Privilege doAsPrivilege(final Privilege privilege) {
    super.doAsPrivilege(privilege);
    privilege.addProperty(FORMAT_KEY, getFormat());
    privilege.addProperty(REPOSITORY_KEY, getRepository());

    return privilege;
  }

  @Override
  protected String doAsActionString() {
    return toBreadActionString();
  }
}
