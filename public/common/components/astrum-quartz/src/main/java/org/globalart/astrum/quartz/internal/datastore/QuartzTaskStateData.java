
package org.globalart.astrum.quartz.internal.datastore;

import org.quartz.JobDataMap;

public class QuartzTaskStateData
{
  private String jobName;

  private JobDataMap jobData;

  private String state;

  public JobDataMap getJobData() {
    return jobData;
  }

  public String getJobName() {
    return jobName;
  }

  public String getState() {
    return state;
  }

  public void setJobData(final JobDataMap jobData) {
    this.jobData = jobData;
  }

  public void setJobName(final String jobName) {
    this.jobName = jobName;
  }

  public void setState(final String state) {
    this.state = state;
  }
}
