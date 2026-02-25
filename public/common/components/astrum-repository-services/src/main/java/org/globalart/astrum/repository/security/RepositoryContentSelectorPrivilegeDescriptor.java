
package org.globalart.astrum.repository.security;

import java.util.List;
import jakarta.ws.rs.core.MediaType;
import jakarta.ws.rs.core.Response.Status;

import org.globalart.astrum.common.i18n.I18N;
import org.globalart.astrum.common.i18n.MessageBundle;
import org.globalart.astrum.formfields.FormField;
import org.globalart.astrum.formfields.RepositoryCombobox;
import org.globalart.astrum.formfields.SelectorComboFormField;
import org.globalart.astrum.formfields.SetOfCheckboxesFormField;
import org.globalart.astrum.formfields.StringTextFormField;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.security.rest.ApiPrivilegeRepositoryContentSelector;
import org.globalart.astrum.repository.security.rest.ApiPrivilegeRepositoryContentSelectorRequest;
import org.globalart.astrum.rest.WebApplicationMessageException;
import org.globalart.astrum.security.config.CPrivilege;
import org.globalart.astrum.security.config.CPrivilegeBuilder;
import org.globalart.astrum.security.privilege.Privilege;
import org.globalart.astrum.security.privilege.PrivilegeDescriptor;
import org.globalart.astrum.security.privilege.rest.PrivilegeAction;
import org.globalart.astrum.selector.SelectorManager;

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
 * Repository selector {@link PrivilegeDescriptor}.
 *
 * @see RepositoryContentSelectorPermission
 * @since 3.1
 */
@Component
@Qualifier(RepositoryContentSelectorPrivilegeDescriptor.TYPE)
public class RepositoryContentSelectorPrivilegeDescriptor
    extends
    RepositoryPrivilegeDescriptorSupport<ApiPrivilegeRepositoryContentSelector, ApiPrivilegeRepositoryContentSelectorRequest>
{
  public static final String INVALID_SELECTOR = "\"Invalid selector '%s' supplied.\"";

  public static final String TYPE = RepositoryContentSelectorPermission.DOMAIN;

  public static final String P_CONTENT_SELECTOR = "contentSelector";

  public static final String P_REPOSITORY = "repository";

  public static final String P_ACTIONS = "actions";

  private interface Messages
      extends MessageBundle
  {
    @DefaultMessage("Repository Content Selector")
    String name();

    @DefaultMessage("Content Selector")
    String contentSelector();

    @DefaultMessage("The content selector for the repository")
    String contentSelectorHelp();

    @DefaultMessage("Repository")
    String repository();

    @DefaultMessage("The repository or repositories to grant access")
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

  private final SelectorManager selectorManager;

  private static final String P_OPTIONS = "options";

  @Autowired
  public RepositoryContentSelectorPrivilegeDescriptor(
      @Lazy final RepositoryManager repositoryManager,
      @Lazy final SelectorManager selectorManager,
      final List<Format> formats,
      @Value(REACT_PRIVILEGES_NAMED_VALUE) final boolean isReactPrivileges)
  {
    super(TYPE, repositoryManager, formats);
    this.selectorManager = checkNotNull(selectorManager);
    this.formFields = ImmutableList.of(
        new SelectorComboFormField(
            P_CONTENT_SELECTOR,
            messages.contentSelector(),
            messages.contentSelectorHelp(),
            FormField.MANDATORY),
        new RepositoryCombobox(
            P_REPOSITORY,
            messages.repository(),
            messages.repositoryHelp(),
            true).includeEntriesForAllFormats(),
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
    String contentSelector = readProperty(privilege, P_CONTENT_SELECTOR, ALL);
    String name = readProperty(privilege, P_REPOSITORY, ALL);
    List<String> actions = readListProperty(privilege, P_ACTIONS, ALL);
    RepositorySelector selector = RepositorySelector.fromSelector(name);
    return new RepositoryContentSelectorPermission(contentSelector, selector.getFormat(), selector.getName(), actions);
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

  public static String id(
      final String contentSelector,
      final String format,
      final String name,
      final String... actions)
  {
    return String.format("as-%s-%s-%s-%s-%s", TYPE, contentSelector, format, name, Joiner.on(',').join(actions));
  }

  public static CPrivilege privilege(
      final String contentSelector,
      final String format,
      final String name,
      final String... actions)
  {
    checkArgument(actions.length > 0);
    RepositorySelector selector = RepositorySelector.fromNameAndFormat(name, format);
    return new CPrivilegeBuilder()
        .type(TYPE)
        .id(id(contentSelector, format, name, actions))
        .description(String
            .format("%s for %s repository content selector %s", humanizeActions(actions), selector.humanizeSelector(),
                contentSelector))
        .property(P_CONTENT_SELECTOR, contentSelector)
        .property(P_REPOSITORY, selector.toSelector())
        .property(P_ACTIONS, actions)
        .create();
  }

  @Override
  public ApiPrivilegeRepositoryContentSelector createApiPrivilegeImpl(final Privilege privilege) {
    return new ApiPrivilegeRepositoryContentSelector(privilege);
  }

  @Override
  public void validate(final ApiPrivilegeRepositoryContentSelectorRequest apiPrivilege) {
    super.validate(apiPrivilege);
    validateContentSelector(apiPrivilege);
  }

  protected void validateContentSelector(final ApiPrivilegeRepositoryContentSelectorRequest apiPrivilege) {
    String cs = apiPrivilege.getContentSelector();

    if (!selectorManager.findByName(cs).isPresent()) {
      throw new WebApplicationMessageException(Status.BAD_REQUEST,
          String.format(INVALID_SELECTOR, apiPrivilege.getContentSelector()), MediaType.APPLICATION_JSON);
    }
  }
}
