
package org.globalart.astrum.script.plugin.internal.security;

import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.core.MediaType;
import jakarta.ws.rs.core.Response;

import org.globalart.astrum.common.i18n.I18N;
import org.globalart.astrum.common.i18n.MessageBundle;
import org.globalart.astrum.formfields.FormField;
import org.globalart.astrum.formfields.SetOfCheckboxesFormField;
import org.globalart.astrum.formfields.StringTextFormField;
import org.globalart.astrum.rest.WebApplicationMessageException;
import org.globalart.astrum.script.ScriptManager;
import org.globalart.astrum.script.plugin.internal.rest.ApiPrivilegeScript;
import org.globalart.astrum.script.plugin.internal.rest.ApiPrivilegeScriptRequest;
import org.globalart.astrum.security.config.CPrivilege;
import org.globalart.astrum.security.config.CPrivilegeBuilder;
import org.globalart.astrum.security.privilege.Privilege;
import org.globalart.astrum.security.privilege.PrivilegeDescriptorSupport;
import org.globalart.astrum.security.privilege.rest.PrivilegeAction;

import com.google.common.base.Joiner;
import org.apache.shiro.authz.Permission;
import org.springframework.beans.factory.annotation.Value;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.FeatureFlags.REACT_PRIVILEGES_NAMED_VALUE;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Descriptor for {@link ScriptPermission}
 *
 * @since 3.0
 */
@Component
@Qualifier("script")
public class ScriptPrivilegeDescriptor
    extends PrivilegeDescriptorSupport<ApiPrivilegeScript, ApiPrivilegeScriptRequest>
{
  public static final String INVALID_SCRIPT = "\"Invalid script '%s' supplied.\"";

  public static final String TYPE = ScriptPermission.DOMAIN;

  public static final String P_NAME = "name";

  public static final String P_ACTIONS = "actions";

  private interface Messages
      extends MessageBundle
  {
    @DefaultMessage("Script")
    String name();

    @DefaultMessage("Script Name")
    String scriptName();

    @DefaultMessage("The name of the script")
    String scriptNameHelp();

    @DefaultMessage("Actions")
    String actions();

    @DefaultMessage("A comma-delimited list (without whitespace) of actions allowed with this privilege; options include browse, read, edit, add, delete, run, and a wildcard (*) <a href=\"https://links.globalart.dev/products/astrum/docs/privileges\" target='_blank'>Help</a>")
    String actionsHelp();

    @DefaultMessage("The actions you wish to allow")
    String actionsCheckboxesHelp();
  }

  private static final Messages messages = I18N.create(Messages.class);

  private final List<FormField> formFields;

  private final ScriptManager scriptManager;

  private static final String P_OPTIONS = "options";

  @Autowired
  public ScriptPrivilegeDescriptor(
      final ScriptManager scriptManager,
      @Value(REACT_PRIVILEGES_NAMED_VALUE) final boolean isReactPrivileges)
  {
    super(TYPE);
    this.formFields = List.of(
        new StringTextFormField(
            P_NAME,
            messages.scriptName(),
            messages.scriptNameHelp(),
            FormField.MANDATORY),
        isReactPrivileges
            ? new SetOfCheckboxesFormField(
                P_ACTIONS,
                messages.actions(),
                messages.actionsCheckboxesHelp(),
                FormField.MANDATORY).withAttribute(P_OPTIONS, PrivilegeAction.getBreadRunActionStrings())
            : new StringTextFormField(
                P_ACTIONS,
                messages.actions(),
                messages.actionsHelp(),
                FormField.MANDATORY,
                "(^(browse|read|edit|add|delete|run)(,(browse|read|edit|add|delete|run)){0,5}$)|(^\\*$)"));
    this.scriptManager = checkNotNull(scriptManager);
  }

  @Override
  public Permission createPermission(final CPrivilege privilege) {
    if (privilege == null) {
      throw new IllegalArgumentException("Privilege cannot be null");
    }
    return new ScriptPermission(readProperty(privilege, P_NAME, ALL), readListProperty(privilege, P_ACTIONS, ALL));
  }

  @Override
  public List<FormField> getFormFields() {
    return formFields;
  }

  @Override
  public String getName() {
    return messages.name();
  }

  public static CPrivilege privilege(final String name, final String... actions) {
    return new CPrivilegeBuilder()
        .type(TYPE)
        .id(id(name, actions))
        .description(humanizeActions(actions) + " for script: " + name)
        .property(P_NAME, name)
        .property(P_ACTIONS, actions)
        .create();
  }

  public static String id(final String name, final String... actions) {
    checkArgument(actions.length > 0);
    return "as-" + TYPE + "-" + name + "-" + Joiner.on(',').join(actions);
  }

  @Override
  public ApiPrivilegeScript createApiPrivilegeImpl(final Privilege privilege) {
    return new ApiPrivilegeScript(privilege);
  }

  @Override
  public void validate(final ApiPrivilegeScriptRequest apiPrivilege) {
    validateActions(apiPrivilege, PrivilegeAction.getBreadRunActions());
    validateScript(apiPrivilege.getScriptName());
  }

  private void validateScript(final String scriptName) {
    if (scriptManager.get(scriptName) == null) {
      throw new WebApplicationMessageException(Response.Status.BAD_REQUEST,
          String.format(INVALID_SCRIPT, scriptName), MediaType.APPLICATION_JSON);
    }
  }
}
