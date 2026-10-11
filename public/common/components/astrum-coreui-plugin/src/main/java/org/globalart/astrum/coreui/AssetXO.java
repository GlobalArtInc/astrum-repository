
package org.globalart.astrum.coreui;

import java.util.Date;
import java.util.Map;
import java.util.Objects;
import jakarta.validation.constraints.NotEmpty;

/**
 * Asset exchange object.
 *
 * @since 3.0
 */
public class AssetXO
{
  @NotEmpty
  private String id;

  @NotEmpty
  private String name;

  @NotEmpty
  private String format;

  @NotEmpty
  private String contentType;

  @NotEmpty
  private long size;

  @NotEmpty
  private String repositoryName;

  @NotEmpty
  private String containingRepositoryName;

  private Date blobCreated;

  private Date blobUpdated;

  private Date lastDownloaded;

  private Date lastVerified;

  /**
   * Number of times the asset was downloaded, {@code null} when it is not known.
   */
  private Long downloadCount;

  @NotEmpty
  private String blobRef;

  private String componentId;

  private String createdBy;

  private String createdByIp;

  @NotEmpty
  private Map<String, Object> attributes;

  /**
   * Top-level docker registry URL (e.g. {@code localhost:8081/my-repo}) computed at read
   * time from the repository's connector configuration. Only populated for docker assets;
   * intentionally kept off {@code attributes.docker} so it does not appear in the
   * Attributes panel — used solely for pull-snippet rendering.
   */
  private String registryUrl;

  public String getId() {
    return id;
  }

  public void setId(String id) {
    this.id = id;
  }

  public String getName() {
    return name;
  }

  public void setName(String name) {
    this.name = name;
  }

  public String getFormat() {
    return format;
  }

  public void setFormat(String format) {
    this.format = format;
  }

  public String getContentType() {
    return contentType;
  }

  public void setContentType(String contentType) {
    this.contentType = contentType;
  }

  public long getSize() {
    return size;
  }

  public void setSize(long size) {
    this.size = size;
  }

  public String getRepositoryName() {
    return repositoryName;
  }

  public void setRepositoryName(String repositoryName) {
    this.repositoryName = repositoryName;
  }

  public String getContainingRepositoryName() {
    return containingRepositoryName;
  }

  public void setContainingRepositoryName(String containingRepositoryName) {
    this.containingRepositoryName = containingRepositoryName;
  }

  public Date getBlobCreated() {
    return blobCreated;
  }

  public void setBlobCreated(Date blobCreated) {
    this.blobCreated = blobCreated;
  }

  public Date getBlobUpdated() {
    return blobUpdated;
  }

  public void setBlobUpdated(Date blobUpdated) {
    this.blobUpdated = blobUpdated;
  }

  public Date getLastDownloaded() {
    return lastDownloaded;
  }

  public void setLastDownloaded(Date lastDownloaded) {
    this.lastDownloaded = lastDownloaded;
  }

  public Date getLastVerified() {
    return lastVerified;
  }

  public void setLastVerified(Date lastVerified) {
    this.lastVerified = lastVerified;
  }

  public Long getDownloadCount() {
    return downloadCount;
  }

  public void setDownloadCount(Long downloadCount) {
    this.downloadCount = downloadCount;
  }

  public String getBlobRef() {
    return blobRef;
  }

  public void setBlobRef(String blobRef) {
    this.blobRef = blobRef;
  }

  public String getComponentId() {
    return componentId;
  }

  public void setComponentId(String componentId) {
    this.componentId = componentId;
  }

  public String getCreatedBy() {
    return createdBy;
  }

  public void setCreatedBy(String createdBy) {
    this.createdBy = createdBy;
  }

  public String getCreatedByIp() {
    return createdByIp;
  }

  public void setCreatedByIp(String createdByIp) {
    this.createdByIp = createdByIp;
  }

  public Map<String, Object> getAttributes() {
    return attributes;
  }

  public void setAttributes(Map<String, Object> attributes) {
    this.attributes = attributes;
  }

  public String getRegistryUrl() {
    return registryUrl;
  }

  public void setRegistryUrl(String registryUrl) {
    this.registryUrl = registryUrl;
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    AssetXO assetXO = (AssetXO) o;
    return Objects.equals(id, assetXO.id);
  }

  @Override
  public int hashCode() {
    return Objects.hash(id);
  }

  @Override
  public String toString() {
    return "AssetXO{" +
        "id='" + id + '\'' +
        ", name='" + name + '\'' +
        ", format='" + format + '\'' +
        ", contentType='" + contentType + '\'' +
        ", size=" + size +
        ", repositoryName='" + repositoryName + '\'' +
        ", containingRepositoryName='" + containingRepositoryName + '\'' +
        ", blobCreated=" + blobCreated +
        ", blobUpdated=" + blobUpdated +
        ", lastDownloaded=" + lastDownloaded +
        ", lastVerified=" + lastVerified +
        ", downloadCount=" + downloadCount +
        ", blobRef='" + blobRef + '\'' +
        ", componentId='" + componentId + '\'' +
        ", createdBy='" + createdBy + '\'' +
        ", createdByIp='" + createdByIp + '\'' +
        ", attributes=" + attributes +
        '}';
  }
}
