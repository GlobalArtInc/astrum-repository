
package org.globalart.astrum.cleanup.internal.rest;

import jakarta.validation.constraints.NotEmpty;

import org.globalart.astrum.cleanup.storage.CleanupPolicyReleaseType;
import org.globalart.astrum.cleanup.storage.config.CleanupPolicyAssetNamePattern;

/**
 * @since 3.29
 */
public class PreviewRequestXO
{
  private String name; // cleanup policy name

  @NotEmpty
  private String repository;

  private Integer criteriaLastBlobUpdated;

  private Integer criteriaLastDownloaded;

  private CleanupPolicyReleaseType criteriaReleaseType;

  @CleanupPolicyAssetNamePattern
  private String criteriaAssetRegex;

  private Integer criteriaRetain;

  private String criteriaSortBy;

  private String filter;

  public String getName() {
    return name;
  }

  public void setName(final String name) {
    this.name = name;
  }

  public String getRepository() {
    return repository;
  }

  public void setRepository(final String repository) {
    this.repository = repository;
  }

  public CleanupPolicyReleaseType getCriteriaReleaseType() {
    return criteriaReleaseType;
  }

  public Integer getCriteriaLastBlobUpdated() {
    return criteriaLastBlobUpdated;
  }

  public Integer getCriteriaLastDownloaded() {
    return criteriaLastDownloaded;
  }

  public String getCriteriaAssetRegex() {
    return criteriaAssetRegex;
  }

  public Integer getCriteriaRetain() {
    return criteriaRetain;
  }

  public String getCriteriaSortBy() {
    return criteriaSortBy;
  }

  public void setCriteriaAssetRegex(final String criteriaAssetRegex) {
    this.criteriaAssetRegex = criteriaAssetRegex;
  }

  public void setCriteriaLastBlobUpdated(final Integer criteriaLastBlobUpdated) {
    this.criteriaLastBlobUpdated = criteriaLastBlobUpdated;
  }

  public void setCriteriaLastDownloaded(final Integer criteriaLastDownloaded) {
    this.criteriaLastDownloaded = criteriaLastDownloaded;
  }

  public void setCriteriaReleaseType(final CleanupPolicyReleaseType criteriaReleaseType) {
    this.criteriaReleaseType = criteriaReleaseType;
  }

  public void setCriteriaRetain(Integer criteriaRetain) {
    this.criteriaRetain = criteriaRetain;
  }

  public void setCriteriaSortBy(String criteriaSortBy) {
    this.criteriaSortBy = criteriaSortBy;
  }

  public String getFilter() {
    return filter;
  }

  public void setFilter(final String filter) {
    this.filter = filter;
  }
}
