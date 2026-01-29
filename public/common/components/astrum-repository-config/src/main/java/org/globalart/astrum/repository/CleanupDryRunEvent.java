
package org.globalart.astrum.repository;

import java.util.Map;

import static com.google.common.base.Preconditions.checkNotNull;

public class CleanupDryRunEvent
{
  public static final String FINISHED_AT_IN_MILLISECONDS = "finishedAt_in_milliseconds";

  public static final String STARTED_AT_IN_MILLISECONDS = "startedAt_in_milliseconds";

  private final Map<String, Object> reference;

  public CleanupDryRunEvent(final Map<String, Object> reference) {
    this.reference = checkNotNull(reference);
  }

  public Map<String, Object> getReference() {
    return reference;
  }

  @Override
  public String toString() {
    return getReference().toString();
  }
}
