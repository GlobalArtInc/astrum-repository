
package org.globalart.astrum.repository.npm.internal.search.v1;

import javax.annotation.Nullable;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * Data carrier for a single search response object for npm V1 search.
 *
 * @since 3.7
 */
@JsonIgnoreProperties(ignoreUnknown = true)
public class NpmSearchResponseObject
{
  @Nullable
  private NpmSearchResponsePackage packageEntry;

  @Nullable
  private NpmSearchResponseScore score;

  @Nullable
  private Double searchScore;

  @Nullable
  @JsonProperty("package")
  public NpmSearchResponsePackage getPackageEntry() {
    return packageEntry;
  }

  public void setPackageEntry(@Nullable final NpmSearchResponsePackage packageEntry) {
    this.packageEntry = packageEntry;
  }

  @Nullable
  public NpmSearchResponseScore getScore() {
    return score;
  }

  public void setScore(@Nullable final NpmSearchResponseScore score) {
    this.score = score;
  }

  @Nullable
  public Double getSearchScore() {
    return searchScore;
  }

  public void setSearchScore(@Nullable final Double searchScore) {
    this.searchScore = searchScore;
  }
}
