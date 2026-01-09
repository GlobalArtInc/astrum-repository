
package org.globalart.astrum.scheduling.internal;

import org.globalart.astrum.scheduling.TaskDescriptorSupport;

/**
 * {@link SimpleTask} descriptor.
 */
public class SimpleTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "simple";

  public SimpleTaskDescriptor() {
    super(TYPE_ID, SimpleTask.class, "simple-name", VISIBLE, EXPOSED);
  }
}
