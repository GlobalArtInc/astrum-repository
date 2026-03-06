
package org.globalart.astrum.cleanup.rest;

import java.util.List;
import java.util.Map;
import java.util.concurrent.TimeUnit;
import jakarta.validation.constraints.Min;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotEmpty;
import jakarta.validation.constraints.Pattern;
import jakarta.validation.constraints.Size;

import org.globalart.astrum.cleanup.storage.CleanupPolicy;
import org.globalart.astrum.cleanup.storage.CleanupPolicyReleaseType;
import org.globalart.astrum.cleanup.storage.SortType;
import org.globalart.astrum.cleanup.storage.config.CleanupPolicyAssetNamePattern;
import org.globalart.astrum.cleanup.storage.config.UniqueCleanupPolicyName;
import org.globalart.astrum.validation.constraint.AnyOf;
import org.globalart.astrum.validation.group.Create;

import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.IS_PRERELEASE_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.LAST_BLOB_UPDATED_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.LAST_DOWNLOADED_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.REGEX_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.RETAIN_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.RETAIN_SORT_BY_KEY;
import static org.globalart.astrum.cleanup.storage.CleanupPolicy.ALL_FORMATS;
import static org.globalart.astrum.cleanup.storage.CleanupPolicy.ALL_CLEANUP_POLICY_FORMAT;
import static org.globalart.astrum.cleanup.storage.CleanupPolicyReleaseType.PRERELEASES;
import static org.globalart.astrum.cleanup.storage.CleanupPolicyReleaseType.RELEASES;
import static org.globalart.astrum.validation.constraint.NamePatternConstants.MESSAGE;
import static org.globalart.astrum.validation.constraint.NamePatternConstants.REGEX;

/**
 * @since 3.29
 */
public class CleanupPolicyXO
{
  @Pattern(regexp = REGEX, message = MESSAGE)
  @UniqueCleanupPolicyName(groups = Create.class)
  @NotEmpty
  @Size(max = 255)
  private String name;

  @NotEmpty
  private String format;

  private String notes;

  private Long criteriaLastBlobUpdated; // days

  private Long criteriaLastDownloaded; // days

  @Min(value = 1, message = "retain must be > 0")
  private Integer retain;

  @AnyOf(enumClass = SortType.class, parameterName = "sortBy")
  private String sortBy;

  private CleanupPolicyReleaseType criteriaReleaseType;

  @CleanupPolicyAssetNamePattern
  private String criteriaAssetRegex;

  private int inUseCount;

  private List<@NotBlank String> repositories;

  public String getName() {
    return name;
  }

  public String getFormat() {
    return format;
  }

  public String getNotes() {
    return notes;
  }

  public CleanupPolicyReleaseType getCriteriaReleaseType() {
    return criteriaReleaseType;
  }

  public Long getCriteriaLastBlobUpdated() {
    return criteriaLastBlobUpdated;
  }

  public Long getCriteriaLastDownloaded() {
    return criteriaLastDownloaded;
  }

  public Integer getRetain() {
    return retain;
  }

  public String getSortBy() {
    return sortBy;
  }

  public String getCriteriaAssetRegex() {
    return criteriaAssetRegex;
  }

  public int getInUseCount() {
    return inUseCount;
  }

  public List<String> getRepositories() {
    return repositories;
  }

  public void setName(final String name) {
    this.name = name;
  }

  public void setFormat(final String format) {
    this.format = format;
  }

  public void setNotes(final String notes) {
    this.notes = notes;
  }

  public void setCriteriaAssetRegex(final String criteriaAssetRegex) {
    this.criteriaAssetRegex = criteriaAssetRegex;
  }

  public void setCriteriaLastBlobUpdated(final Long criteriaLastBlobUpdated) {
    this.criteriaLastBlobUpdated = criteriaLastBlobUpdated;
  }

  public void setCriteriaLastDownloaded(final Long criteriaLastDownloaded) {
    this.criteriaLastDownloaded = criteriaLastDownloaded;
  }

  public void setRetain(final Integer retain) {
    this.retain = retain;
  }

  public void setSortBy(final String sortBy) {
    this.sortBy = sortBy;
  }

  public void setCriteriaReleaseType(final CleanupPolicyReleaseType criteriaReleaseType) {
    this.criteriaReleaseType = criteriaReleaseType;
  }

  public void setInUseCount(int inUseCount) {
    this.inUseCount = inUseCount;
  }

  public void setRepositories(final List<String> repositories) {
    this.repositories = repositories;
  }

  public static CleanupPolicyXO fromCleanupPolicy(final CleanupPolicy cleanupPolicy, int inUseCount) {
    CleanupPolicyXO xo = new CleanupPolicyXO();
    xo.setName(cleanupPolicy.getName());
    xo.setFormat(cleanupPolicy.getFormat().equals(ALL_CLEANUP_POLICY_FORMAT) ? ALL_FORMATS : cleanupPolicy.getFormat());
    xo.setNotes(cleanupPolicy.getNotes());
    xo.setCriteriaAssetRegex(cleanupPolicy.getCriteria().get(REGEX_KEY));
    xo.setCriteriaLastBlobUpdated(toDays(getNullableLong(cleanupPolicy.getCriteria(), LAST_BLOB_UPDATED_KEY)));
    xo.setCriteriaLastDownloaded(toDays(getNullableLong(cleanupPolicy.getCriteria(), LAST_DOWNLOADED_KEY)));
    xo.setRetain(getNullableInt(cleanupPolicy.getCriteria(), RETAIN_KEY));
    xo.setSortBy(cleanupPolicy.getCriteria().getOrDefault(RETAIN_SORT_BY_KEY, null));
    xo.setCriteriaReleaseType(getNullableReleaseType(cleanupPolicy.getCriteria()));
    xo.setInUseCount(inUseCount);
    return xo;
  }

  private static Integer getNullableInt(final Map<String, String> map, final String key) {
    String value = map.get(key);

    if (value == null) {
      return null;
    }

    return Integer.valueOf(value);
  }

  private static Long getNullableLong(final Map<String, String> map, final String key) {
    String value = map.get(key);

    if (value == null) {
      return null;
    }

    return Long.valueOf(value);
  }

  private static CleanupPolicyReleaseType getNullableReleaseType(final Map<String, String> map) {
    String value = map.get(IS_PRERELEASE_KEY);

    if (value == null) {
      return null;
    }

    return Boolean.parseBoolean(value) ? PRERELEASES : RELEASES;
  }

  private static Long toDays(final Long seconds) {
    if (seconds == null) {
      return null;
    }
    else {
      return TimeUnit.SECONDS.toDays(seconds);
    }
  }
}
