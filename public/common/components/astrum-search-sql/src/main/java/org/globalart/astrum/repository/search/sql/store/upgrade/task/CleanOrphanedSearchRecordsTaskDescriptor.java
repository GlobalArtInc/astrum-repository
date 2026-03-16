
package org.globalart.astrum.repository.search.sql.store.upgrade.task;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

/**
 * Task descriptor for {@link CleanOrphanedSearchRecordsTask}.
 */
@AvailabilityVersion(from = "1.0")
@Component
public class CleanOrphanedSearchRecordsTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TASK_VISIBLE_NAMED = "${astrum.cleanup.orphaned.search.records.visible:false}";

  @Autowired
  public CleanOrphanedSearchRecordsTaskDescriptor(
      @Value(TASK_VISIBLE_NAMED) final boolean visible)
  {
    super(CleanOrphanedSearchRecordsTask.TYPE_ID, CleanOrphanedSearchRecordsTask.class,
        "Admin - Cleanup Orphaned Search Records", visible, visible);
  }
}
