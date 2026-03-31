
package org.globalart.astrum.repository.npm.internal.search.v1;

import java.util.List;

import javax.annotation.Nullable;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;

/**
 * Data carrier (mapping to JSON) that contains package information in an npm search response.
 *
 * @since 3.7
 */
@JsonIgnoreProperties(ignoreUnknown = true)
public class NpmSearchResponsePackage
{
  @Nullable
  private String name;

  @Nullable
  private String version;

  @Nullable
  private String description;

  @Nullable
  private List<String> keywords;

  @Nullable
  private String date;

  @Nullable
  private NpmSearchResponsePackageLinks links;

  @Nullable
  private NpmSearchResponsePerson publisher;

  @Nullable
  private List<NpmSearchResponsePerson> maintainers;

  @Nullable
  public String getName() {
    return name;
  }

  public void setName(@Nullable final String name) {
    this.name = name;
  }

  @Nullable
  public String getVersion() {
    return version;
  }

  public void setVersion(@Nullable final String version) {
    this.version = version;
  }

  @Nullable
  public String getDescription() {
    return description;
  }

  public void setDescription(@Nullable final String description) {
    this.description = description;
  }

  @Nullable
  public List<String> getKeywords() {
    return keywords;
  }

  public void setKeywords(@Nullable final List<String> keywords) {
    this.keywords = keywords;
  }

  @Nullable
  public String getDate() {
    return date;
  }

  public void setDate(@Nullable final String date) {
    this.date = date;
  }

  @Nullable
  public NpmSearchResponsePackageLinks getLinks() {
    return links;
  }

  public void setLinks(@Nullable final NpmSearchResponsePackageLinks links) {
    this.links = links;
  }

  @Nullable
  public NpmSearchResponsePerson getPublisher() {
    return publisher;
  }

  public void setPublisher(@Nullable final NpmSearchResponsePerson publisher) {
    this.publisher = publisher;
  }

  @Nullable
  public List<NpmSearchResponsePerson> getMaintainers() {
    return maintainers;
  }

  public void setMaintainers(@Nullable final List<NpmSearchResponsePerson> maintainers) {
    this.maintainers = maintainers;
  }
}
