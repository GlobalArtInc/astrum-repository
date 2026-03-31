
package org.globalart.astrum.repository.npm.internal.search.v1;

import javax.annotation.Nullable;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;

/**
 * Data carrier (mapped to JSON) that contains score detail information for a particular npm search response entry.
 *
 * @since 3.7
 */
@JsonIgnoreProperties(ignoreUnknown = true)
public class NpmSearchResponseScoreDetail
{
  @Nullable
  private Double quality;

  @Nullable
  private Double popularity;

  @Nullable
  private Double maintenance;

  @Nullable
  public Double getQuality() {
    return quality;
  }

  public void setQuality(@Nullable final Double quality) {
    this.quality = quality;
  }

  @Nullable
  public Double getPopularity() {
    return popularity;
  }

  public void setPopularity(@Nullable final Double popularity) {
    this.popularity = popularity;
  }

  @Nullable
  public Double getMaintenance() {
    return maintenance;
  }

  public void setMaintenance(@Nullable final Double maintenance) {
    this.maintenance = maintenance;
  }
}
