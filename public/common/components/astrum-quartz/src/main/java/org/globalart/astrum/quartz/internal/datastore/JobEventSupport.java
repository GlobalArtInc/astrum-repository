
package org.globalart.astrum.quartz.internal.datastore;

import org.globalart.astrum.common.event.EventWithSource;

import com.fasterxml.jackson.annotation.JsonIgnore;
import org.quartz.JobKey;

class JobEventSupport
    extends EventWithSource
{
  private String jobName;

  private String jobGroup;

  JobEventSupport() {
    // deserialization
  }

  JobEventSupport(final JobKey jobKey) {
    this.jobName = jobKey.getName();
    this.jobGroup = jobKey.getGroup();
  }

  @JsonIgnore
  public JobKey getJobKey() {
    return new JobKey(jobName, jobGroup);
  }

  public String getJobName() {
    return jobName;
  }

  public void setJobName(final String jobName) {
    this.jobName = jobName;
  }

  public String getJobGroup() {
    return jobGroup;
  }

  public void setJobGroup(final String jobGroup) {
    this.jobGroup = jobGroup;
  }
}
