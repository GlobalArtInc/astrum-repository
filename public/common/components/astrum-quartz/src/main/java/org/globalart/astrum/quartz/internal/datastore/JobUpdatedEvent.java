
package org.globalart.astrum.quartz.internal.datastore;

import org.quartz.JobKey;

public class JobUpdatedEvent
    extends JobEventSupport
{
  JobUpdatedEvent() {
    // deserialization
  }

  public JobUpdatedEvent(final JobKey jobKey) {
    super(jobKey);
  }
}
