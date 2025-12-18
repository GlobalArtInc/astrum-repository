
package org.globalart.astrum.security.privilege.rest;

import java.util.Arrays;
import java.util.Collection;
import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;

import org.globalart.astrum.security.internal.rest.AstrumSecurityApiConstants;
import org.globalart.astrum.security.privilege.Privilege;

import io.swagger.v3.oas.annotations.media.Schema;
import jakarta.validation.constraints.NotEmpty;

/**
 * @since 3.19
 */
public abstract class ApiPrivilegeWithActionsRequest
    extends ApiPrivilegeRequest
{
  public static final String ACTIONS_KEY = "actions";

  @NotEmpty
  @Schema(description = AstrumSecurityApiConstants.PRIVILEGE_ACTION_DESCRIPTION)
  private Collection<PrivilegeAction> actions;

  public ApiPrivilegeWithActionsRequest() {
    super();
  }

  public ApiPrivilegeWithActionsRequest(
      final String name,
      final String description,
      final Collection<PrivilegeAction> actions)
  {
    super(name, description);
    this.actions = actions;
  }

  public ApiPrivilegeWithActionsRequest(final Privilege privilege) {
    super(privilege);
    String[] parts = privilege.getPrivilegeProperty(ACTIONS_KEY).split(",");
    actions = Arrays.stream(parts)
        .map(PrivilegeAction::fromAction)
        .filter(Objects::nonNull)
        .collect(Collectors.toList());
  }

  public void setActions(Collection<PrivilegeAction> actions) {
    this.actions = actions;
  }

  public Collection<PrivilegeAction> getActions() {
    return actions;
  }

  @Override
  protected Privilege doAsPrivilege(final Privilege privilege) {
    privilege.addProperty(ACTIONS_KEY, asActionString());
    return privilege;
  }

  private String asActionString() {
    return doAsActionString();
  }

  protected abstract String doAsActionString();

  protected String toCrudActionString() {
    if (actions == null || actions.isEmpty()) {
      return null;
    }

    List<String> actionList = actions.stream()
        .map(PrivilegeAction::getCrudAction)
        .filter(Objects::nonNull)
        .collect(Collectors.toList());

    return String.join(",", actionList);
  }

  protected String toCrudTaskActionString() {
    if (actions == null || actions.isEmpty()) {
      return null;
    }

    List<String> actionList = actions.stream()
        .map(PrivilegeAction::getCrudTaskActions)
        .filter(Objects::nonNull)
        .collect(Collectors.toList());

    return String.join(",", actionList);
  }

  protected String toBreadActionString() {
    if (actions == null || actions.isEmpty()) {
      return null;
    }

    List<String> actionList = actions.stream().map(PrivilegeAction::getBreadAction).collect(Collectors.toList());

    return String.join(",", actionList);
  }

  protected String toBreadRunActionString() {
    if (actions == null || actions.isEmpty()) {
      return null;
    }

    List<String> actionList = actions.stream().map(PrivilegeAction::getBreadRunAction).collect(Collectors.toList());

    return String.join(",", actionList);
  }
}
