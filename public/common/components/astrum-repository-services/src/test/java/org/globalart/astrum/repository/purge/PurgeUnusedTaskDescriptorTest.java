
package org.globalart.astrum.repository.purge;

import java.util.List;

import org.globalart.astrum.formfields.FormField;
import org.globalart.astrum.formfields.NumberTextFormField;
import org.globalart.astrum.scheduling.TaskDescriptor;

import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.Matchers.is;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.globalart.astrum.repository.RepositoryTaskSupport.REPOSITORY_NAME_FIELD_ID;
import static org.globalart.astrum.repository.purge.PurgeUnusedTaskDescriptor.LAST_USED_INIT_VALUE;
import static org.globalart.astrum.repository.purge.PurgeUnusedTaskDescriptor.LAST_USED_MIN_VALUE;

public class PurgeUnusedTaskDescriptorTest
{
  private TaskDescriptor purgeUnusedTaskDescriptor;

  @Before
  public void before() {
    purgeUnusedTaskDescriptor = new PurgeUnusedTaskDescriptor();
  }

  /**
   * Ensures the construction of the descriptor has the appropriate/default values
   */
  @Test
  public void testDescriptorConfig() {
    List<FormField> formFields = purgeUnusedTaskDescriptor.getFormFields();

    assertThat(formFields.size(), is(2));
    assertThat(formFields.get(0).getId(), is(REPOSITORY_NAME_FIELD_ID));

    NumberTextFormField lastUsedField = (NumberTextFormField) formFields.get(1);

    assertThat(lastUsedField.getId(), is(PurgeUnusedTask.LAST_USED_FIELD_ID));
    assertThat(lastUsedField.getMinimumValue(), is(LAST_USED_MIN_VALUE));
    assertThat(lastUsedField.getInitialValue(), is(LAST_USED_INIT_VALUE));
  }
}
