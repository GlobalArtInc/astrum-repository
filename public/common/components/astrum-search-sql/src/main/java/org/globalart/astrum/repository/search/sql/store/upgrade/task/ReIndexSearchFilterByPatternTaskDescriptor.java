
package org.globalart.astrum.repository.search.sql.store.upgrade.task;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.formfields.FormField;
import org.globalart.astrum.formfields.StringTextFormField;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.search.sql.store.upgrade.task.ReIndexSearchFilterByPatternTask.FILTER_CONDITION_FIELD_ID;

/**
 * Task descriptor for {@link ReIndexSearchFilterByPatternTask}.
 *
 * This task re-indexes search records for components matching a SQL filter condition.
 * The filter condition is a SQL WHERE clause fragment that is applied to the component table.
 *
 * Example filter conditions:
 * - "namespace LIKE '%_%' OR version LIKE '%_%'" for components with underscores
 */
@AvailabilityVersion(from = "1.0")
@Component
public class ReIndexSearchFilterByPatternTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TASK_VISIBLE_NAMED = "${astrum.reindex.search.by.pattern.visible:false}";

  @Autowired
  public ReIndexSearchFilterByPatternTaskDescriptor(
      @Value(TASK_VISIBLE_NAMED) final boolean visible)
  {
    super(ReIndexSearchFilterByPatternTask.TYPE_ID, ReIndexSearchFilterByPatternTask.class,
        "Admin - Reindex Search Tables By Pattern", visible, visible,
        new StringTextFormField(
            FILTER_CONDITION_FIELD_ID,
            "Filter Condition",
            "SQL filter condition to match components (e.g., \"namespace LIKE '%_%' OR version LIKE '%_%'\")",
            FormField.MANDATORY));
  }
}
