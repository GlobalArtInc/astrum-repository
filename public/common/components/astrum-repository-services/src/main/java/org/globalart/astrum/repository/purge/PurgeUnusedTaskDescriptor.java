
package org.globalart.astrum.repository.purge;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.formfields.FormField;
import org.globalart.astrum.formfields.NumberTextFormField;
import org.globalart.astrum.formfields.RepositoryCombobox;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import static org.globalart.astrum.repository.purge.PurgeUnusedTask.LAST_USED_FIELD_ID;
import static org.globalart.astrum.repository.purge.PurgeUnusedTask.REPOSITORY_NAME_FIELD_ID;
import org.springframework.stereotype.Component;

/**
 * Task descriptor for {@link PurgeUnusedTask}.
 *
 * @since 3.0
 */
@AvailabilityVersion(from = "1.0")
@Component
public class PurgeUnusedTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TASK_NAME = "Repository - Delete unused components";

  public static final String TYPE_ID = "repository.purge-unused";

  public static final Number LAST_USED_INIT_VALUE = 1;

  public static final Number LAST_USED_MIN_VALUE = 1;

  public PurgeUnusedTaskDescriptor() {
    super(TYPE_ID,
        PurgeUnusedTask.class,
        TASK_NAME,
        VISIBLE,
        EXPOSED,
        new RepositoryCombobox(
            REPOSITORY_NAME_FIELD_ID,
            "Repository",
            "Select the repository to purge components/assets from",
            FormField.MANDATORY).includingAnyOfFacets(PurgeUnusedFacet.class).includeAnEntryForAllRepositories(),
        new NumberTextFormField(
            LAST_USED_FIELD_ID,
            "Last used in days",
            "Purge all components and assets that were last used before given number of days",
            FormField.MANDATORY).withInitialValue(LAST_USED_INIT_VALUE).withMinimumValue(LAST_USED_MIN_VALUE));
  }
}
