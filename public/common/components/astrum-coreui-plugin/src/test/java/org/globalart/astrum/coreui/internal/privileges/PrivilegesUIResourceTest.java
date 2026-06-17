
package org.globalart.astrum.coreui.internal.privileges;

import java.util.Arrays;
import java.util.LinkedList;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.formfields.FormField;
import org.globalart.astrum.formfields.SetOfCheckboxesFormField;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.security.RepositoryAdminPrivilegeDescriptor;
import org.globalart.astrum.repository.security.RepositoryContentSelectorPrivilegeDescriptor;
import org.globalart.astrum.repository.security.RepositoryViewPrivilegeDescriptor;
import org.globalart.astrum.script.ScriptManager;
import org.globalart.astrum.script.plugin.internal.security.ScriptPrivilegeDescriptor;
import org.globalart.astrum.security.privilege.ApplicationPrivilegeDescriptor;
import org.globalart.astrum.security.privilege.PrivilegeDescriptor;
import org.globalart.astrum.selector.SelectorManager;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension.WithUser;
import org.globalart.astrum.testcommon.validation.ValidationExtension;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.Mock;

import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.MatcherAssert.assertThat;

@ExtendWith(MockitoExtension.class)
@ExtendWith(ValidationExtension.class)
@ExtendWith(AuthenticationExtension.class)
@WithUser
class PrivilegesUIResourceTest
{
  public static final String HELP_TEXT = "The actions you wish to allow";

  public static final String FORM_TYPE = "setOfCheckboxes";

  public static final String FORM_ID = "actions";

  public static final String ACTIONS_KEY = "options";

  public static final List<String> CRUD_ACTION_STRINGS = Arrays.asList(
      "create", "read", "update", "delete", "start", "stop", "associate", "disassociate");

  public static final List<String> BREAD_ACTION_STRINGS = Arrays.asList("browse", "read", "edit", "add", "delete");

  public static final List<String> BREAD_RUN_ACTION_STRINGS =
      Arrays.asList("browse", "read", "edit", "add", "delete", "run");

  @Mock
  private RepositoryManager repositoryManager;

  @Mock
  private SelectorManager selectorManager;

  @Mock
  private ScriptManager scriptManager;

  @Mock
  private Format format1;

  @Mock
  private Format format2;

  @Mock
  private Repository repository1;

  @Mock
  private Repository repository2;

  private PrivilegesUIResource underTest;

  @BeforeEach
  void setup() {
    List<Format> formats = Arrays.asList(format1, format2);

    List<PrivilegeDescriptor> privilegeDescriptors = new LinkedList<>();
    privilegeDescriptors.add(new ApplicationPrivilegeDescriptor(true));
    privilegeDescriptors.add(new RepositoryAdminPrivilegeDescriptor(repositoryManager, formats, true));
    privilegeDescriptors.add(new RepositoryViewPrivilegeDescriptor(repositoryManager, formats, true));
    privilegeDescriptors
        .add(new RepositoryContentSelectorPrivilegeDescriptor(repositoryManager, selectorManager, formats, true));
    privilegeDescriptors.add(new ScriptPrivilegeDescriptor(scriptManager, true));

    underTest = new PrivilegesUIResource(privilegeDescriptors);
  }

  @Test
  void listPrivilegesTypes() {
    List<PrivilegesTypesUIResponse> responses = underTest.listPrivilegesTypes();
    assertThat(responses.size(), is(5));

    PrivilegesTypesUIResponse responseApplication = responses.stream()
        .filter(r -> r.getId() == ApplicationPrivilegeDescriptor.TYPE)
        .findFirst()
        .orElse(null);
    assertSetOfCheckboxesFormField(responseApplication, HELP_TEXT, FORM_TYPE, CRUD_ACTION_STRINGS);

    PrivilegesTypesUIResponse responseRepositoryAdmin = responses.stream()
        .filter(r -> r.getId() == RepositoryAdminPrivilegeDescriptor.TYPE)
        .findFirst()
        .orElse(null);
    assertSetOfCheckboxesFormField(responseRepositoryAdmin, HELP_TEXT, FORM_TYPE, BREAD_ACTION_STRINGS);

    PrivilegesTypesUIResponse responseRepositoryView = responses.stream()
        .filter(r -> r.getId() == RepositoryViewPrivilegeDescriptor.TYPE)
        .findFirst()
        .orElse(null);
    assertSetOfCheckboxesFormField(responseRepositoryView, HELP_TEXT, FORM_TYPE, BREAD_ACTION_STRINGS);

    PrivilegesTypesUIResponse responseRepositoryContentSelector = responses.stream()
        .filter(r -> r.getId() == RepositoryContentSelectorPrivilegeDescriptor.TYPE)
        .findFirst()
        .orElse(null);
    assertSetOfCheckboxesFormField(responseRepositoryContentSelector, HELP_TEXT, FORM_TYPE, BREAD_ACTION_STRINGS);

    PrivilegesTypesUIResponse responseScript = responses.stream()
        .filter(r -> r.getId() == ScriptPrivilegeDescriptor.TYPE)
        .findFirst()
        .orElse(null);
    assertSetOfCheckboxesFormField(responseScript, HELP_TEXT, FORM_TYPE, BREAD_RUN_ACTION_STRINGS);
  }

  private static void assertSetOfCheckboxesFormField(
      final PrivilegesTypesUIResponse response,
      final String helpText,
      final String type,
      final List<String> actions)
  {
    List<FormField> formFields = response.getFormFields();
    SetOfCheckboxesFormField formField = (SetOfCheckboxesFormField) formFields.stream()
        .filter(r -> r.getId() == FORM_ID)
        .findFirst()
        .orElse(null);
    assertThat(formField.getType(), is(type));
    assertThat(formField.getHelpText(), is(helpText));

    Map<String, Object> attributes = formField.getAttributes();
    assertThat(attributes.get(ACTIONS_KEY), is(actions));
  }
}
