
package org.globalart.astrum.repository.search.index;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.formfields.RepositoryCombobox;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Task descriptor for {@link RebuildIndexTask}.
 *
 * @since 3.0
 */
@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(RebuildIndexTaskDescriptor.TYPE_ID)
public class RebuildIndexTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "repository.rebuild-index";

  public static final String REPOSITORY_NAME_FIELD_ID = "repositoryName";

  @Autowired
  public RebuildIndexTaskDescriptor() {
    super(TYPE_ID,
        RebuildIndexTask.class,
        "Repair - Rebuild repository search",
        VISIBLE,
        EXPOSED,
        new RepositoryCombobox(
            REPOSITORY_NAME_FIELD_ID,
            "Repository",
            "Select the repository to rebuild index",
            true).includingAnyOfFacets(SearchIndexFacet.class).includeAnEntryForAllRepositories());
  }
}
