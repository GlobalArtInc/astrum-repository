
package org.globalart.astrum.repository.npm.internal.search.v1;

import javax.annotation.Nullable;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * Data carrier (mapping to JSON) that contains the search result score information for a particular package for npm
 * search V1.
 *
 * @since 3.7
 */
@JsonIgnoreProperties(ignoreUnknown = true)
public class NpmSearchResponseScore
{
  @Nullable
  private Double finalScore;

  @Nullable
  private NpmSearchResponseScoreDetail detail;

  @Nullable
  @JsonProperty("final")
  public Double getFinalScore() {
    return finalScore;
  }

  public void setFinalScore(@Nullable final Double finalScore) {
    this.finalScore = finalScore;
  }

  @Nullable
  public NpmSearchResponseScoreDetail getDetail() {
    return detail;
  }

  public void setDetail(@Nullable final NpmSearchResponseScoreDetail detail) {
    this.detail = detail;
  }
}
