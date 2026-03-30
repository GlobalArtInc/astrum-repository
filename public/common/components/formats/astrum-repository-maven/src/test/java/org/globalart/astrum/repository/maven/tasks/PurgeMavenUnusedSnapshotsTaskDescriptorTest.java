
package org.globalart.astrum.repository.maven.tasks;

import java.util.List;

import org.globalart.astrum.formfields.FormField;
import org.globalart.astrum.formfields.NumberTextFormField;
import org.globalart.astrum.scheduling.TaskDescriptor;

import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.Matchers.is;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.globalart.astrum.repository.RepositoryTaskSupport.REPOSITORY_NAME_FIELD_ID;
import static org.globalart.astrum.repository.maven.tasks.PurgeMavenUnusedSnapshotsTaskDescriptor.LAST_USED_INIT_VALUE;
import static org.globalart.astrum.repository.maven.tasks.PurgeMavenUnusedSnapshotsTaskDescriptor.LAST_USED_MIN_VALUE;

public class PurgeMavenUnusedSnapshotsTaskDescriptorTest
{
  private TaskDescriptor purgeMavenUnusedSnapshotsTaskDescriptor;

  @Before
  public void before() {
    purgeMavenUnusedSnapshotsTaskDescriptor = new PurgeMavenUnusedSnapshotsTaskDescriptor();
  }

  /**
   * Ensures the construction of the descriptor has the appropriate/default values
   */
  @Test
  public void testDescriptorConfig() {
    List<FormField> formFields = purgeMavenUnusedSnapshotsTaskDescriptor.getFormFields();

    assertThat(formFields.size(), is(2));
    assertThat(formFields.get(0).getId(), is(REPOSITORY_NAME_FIELD_ID));

    NumberTextFormField lastUsedField = (NumberTextFormField) formFields.get(1);

    assertThat(lastUsedField.getId(), is(PurgeMavenUnusedSnapshotsTask.LAST_USED_FIELD_ID));
    assertThat(lastUsedField.getMinimumValue(), is(LAST_USED_MIN_VALUE));
    assertThat(lastUsedField.getInitialValue(), is(LAST_USED_INIT_VALUE));
  }
}
