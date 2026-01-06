
package org.globalart.astrum.quartz.internal.datastore;

import org.quartz.JobKey;

public class JobDeletedEvent
    extends JobEventSupport
{
  JobDeletedEvent() {
    // deserialization
  }

  public JobDeletedEvent(final JobKey jobKey) {
    super(jobKey);
  }
}
