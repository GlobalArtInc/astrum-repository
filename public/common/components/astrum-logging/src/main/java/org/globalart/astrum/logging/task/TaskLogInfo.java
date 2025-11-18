
package org.globalart.astrum.logging.task;

import javax.annotation.Nullable;

/**
 * Expose task information for the per task logs. See the TaskConfiguration class.
 *
 * @since 3.5
 */
public interface TaskLogInfo
{
  String getId();

  String getTypeId();

  String getName();

  String getMessage();

  @Nullable
  String getString(final String key);

  boolean getBoolean(final String key, final boolean defaultValue);

  int getInteger(final String key, final int defaultValue);

  String toString();
}
