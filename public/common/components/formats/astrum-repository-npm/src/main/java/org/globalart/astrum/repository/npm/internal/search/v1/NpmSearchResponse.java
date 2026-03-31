
package org.globalart.astrum.repository.npm.internal.search.v1;

import java.util.List;

import javax.annotation.Nullable;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;

/**
 * Response for an npm V1 search request.
 *
 * @since 3.7
 */
@JsonIgnoreProperties(ignoreUnknown = true)
public class NpmSearchResponse
{
  @Nullable
  private List<NpmSearchResponseObject> objects;

  @Nullable
  private Integer total;

  @Nullable
  private String time;

  @Nullable
  public List<NpmSearchResponseObject> getObjects() {
    return objects;
  }

  public void setObjects(@Nullable final List<NpmSearchResponseObject> objects) {
    this.objects = objects;
  }

  @Nullable
  public Integer getTotal() {
    return total;
  }

  public void setTotal(@Nullable final Integer total) {
    this.total = total;
  }

  @Nullable
  public String getTime() {
    return time;
  }

  public void setTime(@Nullable final String time) {
    this.time = time;
  }
}
