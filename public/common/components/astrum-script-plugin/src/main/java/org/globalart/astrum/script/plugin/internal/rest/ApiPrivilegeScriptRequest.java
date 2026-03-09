
package org.globalart.astrum.script.plugin.internal.rest;

import java.util.Collection;

import org.globalart.astrum.script.plugin.internal.security.ScriptPrivilegeDescriptor;
import org.globalart.astrum.security.internal.rest.AstrumSecurityApiConstants;
import org.globalart.astrum.security.privilege.Privilege;
import org.globalart.astrum.security.privilege.rest.ApiPrivilegeWithActionsRequest;
import org.globalart.astrum.security.privilege.rest.PrivilegeAction;

import io.swagger.v3.oas.annotations.media.Schema;
import jakarta.validation.constraints.NotBlank;

/**
 * @since 3.19
 */
public class ApiPrivilegeScriptRequest
    extends ApiPrivilegeWithActionsRequest
{
  public static final String SCRIPT_KEY = "name";

  @NotBlank
  @Schema(description = AstrumSecurityApiConstants.PRIVILEGE_SCRIPT_DESCRIPTION)
  private String scriptName;

  /**
   * for deserialization
   */
  private ApiPrivilegeScriptRequest() {
    super();
  }

  public ApiPrivilegeScriptRequest(
      final String name,
      final String description,
      final String scriptName,
      final Collection<PrivilegeAction> actions)
  {
    super(name, description, actions);
    this.scriptName = scriptName;
  }

  public ApiPrivilegeScriptRequest(final Privilege privilege) {
    super(privilege);
    scriptName = privilege.getPrivilegeProperty(SCRIPT_KEY);
  }

  public void setScriptName(final String scriptName) {
    this.scriptName = scriptName;
  }

  public String getScriptName() {
    return scriptName;
  }

  @Override
  protected Privilege doAsPrivilege(final Privilege privilege) {
    super.doAsPrivilege(privilege);
    privilege.addProperty(SCRIPT_KEY, scriptName);
    privilege.setType(ScriptPrivilegeDescriptor.TYPE);
    return privilege;
  }

  @Override
  protected String doAsActionString() {
    return toBreadRunActionString();
  }
}
