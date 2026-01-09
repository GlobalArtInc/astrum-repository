
package org.globalart.astrum.scheduling.internal;

import org.globalart.astrum.scheduling.TaskSupport;

/**
 * Simple task.
 *
 * @see SimpleTaskDescriptor
 */
public class SimpleTask
    extends TaskSupport
{
  @Override
  protected Object execute() throws Exception {
    return "simple-result";
  }

  @Override
  public String getMessage() {
    return "simple-message";
  }
}
