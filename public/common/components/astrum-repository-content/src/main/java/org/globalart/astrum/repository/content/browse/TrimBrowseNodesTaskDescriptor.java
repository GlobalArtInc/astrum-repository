
package org.globalart.astrum.repository.content.browse;

import org.globalart.astrum.common.i18n.I18N;
import org.globalart.astrum.common.i18n.MessageBundle;
import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.formfields.RepositoryCombobox;
import org.globalart.astrum.repository.RepositoryTaskSupport;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import org.springframework.stereotype.Component;

@AvailabilityVersion(from = "1.0")
@Component
public class TrimBrowseNodesTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "repository.trim-browse-tree";

  public static final String REPOSITORY_NAME_FIELD_ID = RepositoryTaskSupport.REPOSITORY_NAME_FIELD_ID;

  private interface Messages
      extends MessageBundle
  {
    @DefaultMessage("Repair - Repository trim browse tree")
    String name();

    @DefaultMessage("Repository")
    String repositoryLabel();
  }

  private static final Messages messages = I18N.create(Messages.class);

  public TrimBrowseNodesTaskDescriptor() {
    super(
        TYPE_ID,
        TrimBrowseNodesTask.class,
        messages.name(),
        VISIBLE,
        EXPOSED,
        new RepositoryCombobox(
            REPOSITORY_NAME_FIELD_ID,
            messages.repositoryLabel(),
            "Select the repository to trim empty browse nodes from",
            true).includingAnyOfFacets(BrowseFacet.class).includeAnEntryForAllRepositories());
  }

  @Override
  public boolean allowConcurrentRun() {
    return false;
  }
}
