
package org.globalart.astrum.repository.browse.node;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.app.FeatureFlags;
import org.globalart.astrum.common.node.NodeAccess;
import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.formfields.ItemselectFormField;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

/**
 * Task descriptor for {@link RebuildBrowseNodesTask}.
 *
 * @since 3.6
 */
@AvailabilityVersion(from = "1.0")
@Component
public class RebuildBrowseNodesTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "create.browse.nodes";

  public static final String REPOSITORY_NAME_FIELD_ID = "repositoryName";

  public static final String TASK_NAME = "Repair - Rebuild repository browse";

  @Autowired
  public RebuildBrowseNodesTaskDescriptor(
      final NodeAccess nodeAccess,
      final GroupType groupType,
      @Value(FeatureFlags.DATASTORE_CLUSTERED_ENABLED_NAMED_VALUE) final boolean datastoreClustered)
  {
    super(TYPE_ID, RebuildBrowseNodesTask.class, TASK_NAME, VISIBLE, EXPOSED,
        new ItemselectFormField(REPOSITORY_NAME_FIELD_ID, "Repository",
            "Select the repository(ies) to rebuild browse tree",
            true).withStoreApi("coreui_Repository.readReferencesAddingEntryForAll")
                .withButtons("up", "add", "remove", "down")
                .withFromTitle("Available")
                .withToTitle("Selected")
                .withStoreFilter("type", "!" + groupType.getValue())
                .withValueAsString(true),
        (nodeAccess.isClustered() && !datastoreClustered) ? newLimitNodeFormField() : null);
  }
}
