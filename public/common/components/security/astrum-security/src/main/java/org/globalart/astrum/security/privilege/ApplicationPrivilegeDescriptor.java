
package org.globalart.astrum.security.privilege;

import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.common.i18n.I18N;
import org.globalart.astrum.common.i18n.MessageBundle;
import org.globalart.astrum.formfields.FormField;
import org.globalart.astrum.formfields.SetOfCheckboxesFormField;
import org.globalart.astrum.formfields.StringTextFormField;
import org.globalart.astrum.security.config.CPrivilege;
import org.globalart.astrum.security.config.CPrivilegeBuilder;
import org.globalart.astrum.security.privilege.rest.ApiPrivilegeApplication;
import org.globalart.astrum.security.privilege.rest.ApiPrivilegeApplicationRequest;
import org.globalart.astrum.security.privilege.rest.PrivilegeAction;

import com.google.common.base.Joiner;
import com.google.common.collect.ImmutableList;
import org.apache.shiro.authz.Permission;
import org.springframework.beans.factory.annotation.Value;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.FeatureFlags.REACT_PRIVILEGES_NAMED_VALUE;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Application {@link PrivilegeDescriptor}.
 *
 * @see ApplicationPermission
 * @since 3.0
 */
@Component
@Qualifier(ApplicationPrivilegeDescriptor.TYPE)
public class ApplicationPrivilegeDescriptor
    extends PrivilegeDescriptorSupport<ApiPrivilegeApplication, ApiPrivilegeApplicationRequest>
{
  public static final String TYPE = "application";

  public static final String P_DOMAIN = "domain";

  public static final String P_ACTIONS = "actions";

  private interface Messages
      extends MessageBundle
  {
    @DefaultMessage("Application")
    String name();

    @DefaultMessage("Domain")
    String domain();

    @DefaultMessage("The domain for the privilege")
    String domainHelp();

    @DefaultMessage("Actions")
    String actions();

    @DefaultMessage("A comma-delimited list (without whitespace) of actions allowed with this privilege; " +
        "options include create, read, update, delete, start, stop, associate, disassociate, and a wildcard (*) " +
        "<a href='https://links.globalart.dev/products/astrum/docs/privileges' target='_blank'>Help</a>")
    String actionsHelp();

    @DefaultMessage("The actions you wish to allow")
    String actionsCheckboxesHelp();
  }

  private static final Messages messages = I18N.create(Messages.class);

  private final List<FormField> formFields;

  private static final String P_OPTIONS = "options";

  @Autowired
  public ApplicationPrivilegeDescriptor(
      @Value(REACT_PRIVILEGES_NAMED_VALUE) final boolean isReactPrivileges)
  {
    super(TYPE);
    this.formFields = ImmutableList.of(
        new StringTextFormField(
            P_DOMAIN,
            messages.domain(),
            messages.domainHelp(),
            FormField.MANDATORY),
        isReactPrivileges
            ? new SetOfCheckboxesFormField(
                P_ACTIONS,
                messages.actions(),
                messages.actionsCheckboxesHelp(),
                FormField.MANDATORY).withAttribute(P_OPTIONS, PrivilegeAction.getCrudTaskActionStrings())
            : new StringTextFormField(
                P_ACTIONS,
                messages.actions(),
                messages.actionsHelp(),
                FormField.MANDATORY,
                "(^(create|read|update|delete|start|stop|associate|disassociate)" +
                    "(,(create|read|update|delete|start|stop|associate|disassociate)){0,3}$)|(^\\*$)"));
  }

  @Override
  public Permission createPermission(final CPrivilege privilege) {
    checkNotNull(privilege);
    String domain = readProperty(privilege, P_DOMAIN, ALL);
    List<String> actions = readListProperty(privilege, P_ACTIONS, ALL);
    return new ApplicationPermission(domain, actions);
  }

  @Override
  public List<FormField> getFormFields() {
    return formFields;
  }

  @Override
  public String getName() {
    return messages.name();
  }

  //
  // Helpers
  //

  public static String id(final String domain, final String... actions) {
    return String.format("%s-%s", domain, Joiner.on(',').join(actions));
  }

  public static CPrivilege privilege(final String domain, final String... actions) {
    return new CPrivilegeBuilder()
        .type(TYPE)
        .id(id(domain, actions))
        .property(P_DOMAIN, domain)
        .property(P_ACTIONS, actions)
        .create();
  }

  @Override
  public ApiPrivilegeApplication createApiPrivilegeImpl(final Privilege privilege) {
    return new ApiPrivilegeApplication(privilege);
  }

  @Override
  public void validate(final ApiPrivilegeApplicationRequest apiPrivilege) {
    validateActions(apiPrivilege, PrivilegeAction.getCrudTaskAction());
  }
}
