
package org.globalart.astrum.repository.security;

import java.util.List;

import org.globalart.astrum.common.i18n.I18N;
import org.globalart.astrum.common.i18n.MessageBundle;
import org.globalart.astrum.formfields.FormField;
import org.globalart.astrum.formfields.RepositoryCombobox;
import org.globalart.astrum.formfields.SetOfCheckboxesFormField;
import org.globalart.astrum.formfields.StringTextFormField;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.security.rest.ApiPrivilegeRepositoryView;
import org.globalart.astrum.repository.security.rest.ApiPrivilegeRepositoryViewRequest;
import org.globalart.astrum.security.config.CPrivilege;
import org.globalart.astrum.security.config.CPrivilegeBuilder;
import org.globalart.astrum.security.privilege.Privilege;
import org.globalart.astrum.security.privilege.PrivilegeDescriptor;
import org.globalart.astrum.security.privilege.rest.PrivilegeAction;

import com.google.common.base.Joiner;
import com.google.common.collect.ImmutableList;
import org.springframework.beans.factory.annotation.Autowired;
import org.apache.shiro.authz.Permission;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.context.annotation.Lazy;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.FeatureFlags.REACT_PRIVILEGES_NAMED_VALUE;

/**
 * Repository view {@link PrivilegeDescriptor}.
 *
 * @see RepositoryViewPermission
 * @since 3.0
 */
@Component
@Qualifier(RepositoryViewPrivilegeDescriptor.TYPE)
public class RepositoryViewPrivilegeDescriptor
    extends RepositoryPrivilegeDescriptorSupport<ApiPrivilegeRepositoryView, ApiPrivilegeRepositoryViewRequest>
{
  public static final String TYPE = RepositoryViewPermission.DOMAIN;

  public static final String P_FORMAT = "format";

  public static final String P_REPOSITORY = "repository";

  public static final String P_ACTIONS = "actions";

  private interface Messages
      extends MessageBundle
  {
    @DefaultMessage("Repository View")
    String name();

    @DefaultMessage("Format")
    String format();

    @DefaultMessage("The format(s) for the repository")
    String formatHelp();

    @DefaultMessage("Repository")
    String repository();

    @DefaultMessage("The repository name")
    String repositoryHelp();

    @DefaultMessage("Actions")
    String actions();

    @DefaultMessage("A comma-delimited list (without whitespace) of actions allowed with this privilege; " +
        "options include browse, read, edit, add, delete, and a wildcard (*) " +
        "<a href='https://links.globalart.dev/products/astrum/docs/privileges' target='_blank'>Help</a>")
    String actionsHelp();

    @DefaultMessage("The actions you wish to allow")
    String actionsCheckboxesHelp();
  }

  private static final Messages messages = I18N.create(Messages.class);

  private final List<FormField> formFields;

  private static final String P_OPTIONS = "options";

  @Autowired
  public RepositoryViewPrivilegeDescriptor(
      @Lazy final RepositoryManager repositoryManager,
      final List<Format> formats,
      @Value(REACT_PRIVILEGES_NAMED_VALUE) final boolean isReactPrivileges)
  {
    super(TYPE, repositoryManager, formats);
    this.formFields = ImmutableList.of(
        new StringTextFormField(
            P_FORMAT,
            messages.format(),
            messages.formatHelp(),
            FormField.MANDATORY),
        new RepositoryCombobox(
            P_REPOSITORY,
            messages.repository(),
            messages.repositoryHelp(),
            true).includeAnEntryForAllRepositories(),
        isReactPrivileges
            ? new SetOfCheckboxesFormField(
                P_ACTIONS,
                messages.actions(),
                messages.actionsCheckboxesHelp(),
                FormField.MANDATORY).withAttribute(P_OPTIONS, PrivilegeAction.getBreadActionStrings())
            : new StringTextFormField(
                P_ACTIONS,
                messages.actions(),
                messages.actionsHelp(),
                FormField.MANDATORY,
                "(^(browse|read|edit|add|delete)(,(browse|read|edit|add|delete)){0,4}$)|(^\\*$)"));
  }

  @Override
  public Permission createPermission(final CPrivilege privilege) {
    checkNotNull(privilege);
    String format = readProperty(privilege, P_FORMAT, ALL);
    String name = readProperty(privilege, P_REPOSITORY, ALL);
    List<String> actions = readListProperty(privilege, P_ACTIONS, ALL);
    return new RepositoryViewPermission(format, name, actions);
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

  public static String id(final String format, final String name, final String... actions) {
    return String.format("as-%s-%s-%s-%s", TYPE, format, name, Joiner.on(',').join(actions));
  }

  public static CPrivilege privilege(final String format, final String name, final String... actions) {
    checkArgument(actions.length > 0);
    return new CPrivilegeBuilder()
        .type(TYPE)
        .id(id(format, name, actions))
        .description(String.format("%s for %s repository views", humanizeActions(actions), humanizeName(name, format)))
        .property(P_FORMAT, format)
        .property(P_REPOSITORY, name)
        .property(P_ACTIONS, actions)
        .create();
  }

  @Override
  public ApiPrivilegeRepositoryView createApiPrivilegeImpl(final Privilege privilege) {
    return new ApiPrivilegeRepositoryView(privilege);
  }
}
