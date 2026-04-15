
package org.globalart.astrum.repository.golang.internal.metadata;

import com.fasterxml.jackson.annotation.JsonProperty;

public class GolangInfo
{
  @JsonProperty(value = "Version")
  private String version;

  @JsonProperty(value = "Time")
  private String time;

  public GolangInfo(final String version, final String time) {
    this.version = version;
    this.time = time;
  }

  public String getVersion() {
    return version;
  }

  public void setVersion(final String version) {
    this.version = version;
  }

  public String getTime() {
    return time;
  }

  public void setTime(final String time) {
    this.time = time;
  }
}
