
package org.globalart.astrum.repository.internal.search.index.task;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.common.i18n.I18N;
import org.globalart.astrum.common.i18n.MessageBundle;
import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;
import org.springframework.stereotype.Component;

/**
 * Task descriptor for {@link SearchUpdateTask}.
 */
@AvailabilityVersion(from = "1.0")
@Component
public class SearchUpdateTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "repository.search.update";

  public static final String REPOSITORY_NAMES_FIELD_ID = "repositoryNames";

  private static final Messages messages = I18N.create(Messages.class);

  @Autowired
  public SearchUpdateTaskDescriptor() {
    super(TYPE_ID,
        SearchUpdateTask.class,
        messages.name(),
        NOT_VISIBLE,
        NOT_EXPOSED);
  }

  @Override
  public boolean isSingletonTaskType() {
    return true;
  }

  private interface Messages
      extends MessageBundle
  {
    @DefaultMessage("Update repository search indexes")
    String name();
  }
}
