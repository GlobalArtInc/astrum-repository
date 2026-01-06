
package org.globalart.astrum.quartz.internal.datastore;

import org.quartz.JobKey;

public class JobCreatedEvent
    extends JobEventSupport
{
  JobCreatedEvent() {
    // deserialization
  }

  public JobCreatedEvent(final JobKey jobKey) {
    super(jobKey);
  }
}
