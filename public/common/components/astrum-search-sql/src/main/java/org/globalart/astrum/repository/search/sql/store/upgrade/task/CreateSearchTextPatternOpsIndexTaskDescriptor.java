
package org.globalart.astrum.repository.search.sql.store.upgrade.task;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

@AvailabilityVersion(from = "1.0")
@Component
public class CreateSearchTextPatternOpsIndexTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "search.text.pattern.ops.index.migration";

  @Autowired
  public CreateSearchTextPatternOpsIndexTaskDescriptor(
      @Value("${search.text.pattern.ops.index.migration.exposed:false}") final boolean exposed)
  {
    super(TYPE_ID, CreateSearchTextPatternOpsIndexTask.class,
        "Create text_pattern_ops indexes on search_components",
        exposed, exposed, true);
  }
}
