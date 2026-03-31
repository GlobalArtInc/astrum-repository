
package org.globalart.astrum.repository.npm.internal.search.v1;

import javax.annotation.Nullable;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;

/**
 * Data carrier (mapping to JSON) for the links portion of a package entry in an npm V1 search response.
 *
 * @since 3.7
 */
@JsonIgnoreProperties(ignoreUnknown = true)
public class NpmSearchResponsePackageLinks
{
  @Nullable
  private String npm;

  @Nullable
  private String homepage;

  @Nullable
  private String repository;

  @Nullable
  private String bugs;

  @Nullable
  public String getNpm() {
    return npm;
  }

  public void setNpm(@Nullable final String npm) {
    this.npm = npm;
  }

  @Nullable
  public String getHomepage() {
    return homepage;
  }

  public void setHomepage(@Nullable final String homepage) {
    this.homepage = homepage;
  }

  @Nullable
  public String getRepository() {
    return repository;
  }

  public void setRepository(@Nullable final String repository) {
    this.repository = repository;
  }

  @Nullable
  public String getBugs() {
    return bugs;
  }

  public void setBugs(@Nullable final String bugs) {
    this.bugs = bugs;
  }
}
