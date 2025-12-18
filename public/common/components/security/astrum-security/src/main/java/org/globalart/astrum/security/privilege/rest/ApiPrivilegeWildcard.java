
package org.globalart.astrum.security.privilege.rest;

import org.globalart.astrum.security.internal.rest.AstrumSecurityApiConstants;
import org.globalart.astrum.security.privilege.Privilege;
import org.globalart.astrum.security.privilege.WildcardPrivilegeDescriptor;

import io.swagger.v3.oas.annotations.media.Schema;
import jakarta.validation.constraints.NotBlank;

/**
 * @since 3.19
 */
public class ApiPrivilegeWildcard
    extends ApiPrivilege
{
  public static final String PATTERN_KEY = "pattern";

  @NotBlank
  @Schema(description = AstrumSecurityApiConstants.PRIVILEGE_PATTERN_DESCRIPTION)
  private String pattern;

  /**
   * for deserialization
   */
  private ApiPrivilegeWildcard() {
    super(WildcardPrivilegeDescriptor.TYPE);
  }

  public ApiPrivilegeWildcard(
      final String name,
      final String description,
      final boolean readOnly,
      final String pattern)
  {
    super(WildcardPrivilegeDescriptor.TYPE, name, description, readOnly);
    this.pattern = pattern;
  }

  public ApiPrivilegeWildcard(final Privilege privilege) {
    super(privilege);
    pattern = privilege.getPrivilegeProperty(PATTERN_KEY);
  }

  public void setPattern(final String pattern) {
    this.pattern = pattern;
  }

  public String getPattern() {
    return pattern;
  }

  @Override
  protected Privilege doAsPrivilege(final Privilege privilege) {
    privilege.addProperty(PATTERN_KEY, pattern);
    return privilege;
  }
}
