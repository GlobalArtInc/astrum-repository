package org.globalart.astrum.repository.content.usage;

import java.time.OffsetDateTime;
import javax.annotation.Nullable;

/**
 * The coordinates of a component, who added it and where the browse tree shows it, as read by {@link UsageStore}.
 */
public class ComponentSummary
{
  private int componentId;

  private int repositoryId;

  private String namespace;

  private String name;

  private String version;

  private OffsetDateTime created;

  @Nullable
  private String createdBy;

  @Nullable
  private String browsePath;

  public int getComponentId() {
    return componentId;
  }

  public void setComponentId(final int componentId) {
    this.componentId = componentId;
  }

  public int getRepositoryId() {
    return repositoryId;
  }

  public void setRepositoryId(final int repositoryId) {
    this.repositoryId = repositoryId;
  }

  public String getNamespace() {
    return namespace;
  }

  public void setNamespace(final String namespace) {
    this.namespace = namespace;
  }

  public String getName() {
    return name;
  }

  public void setName(final String name) {
    this.name = name;
  }

  public String getVersion() {
    return version;
  }

  public void setVersion(final String version) {
    this.version = version;
  }

  public OffsetDateTime getCreated() {
    return created;
  }

  public void setCreated(final OffsetDateTime created) {
    this.created = created;
  }

  /**
   * The user who uploaded the first blob of the component, if known.
   */
  @Nullable
  public String getCreatedBy() {
    return createdBy;
  }

  public void setCreatedBy(@Nullable final String createdBy) {
    this.createdBy = createdBy;
  }

  /**
   * The request path of the browse node of the component, or of one of its assets when the component has no node of
   * its own; {@code null} when the browse tree does not show it (yet).
   */
  @Nullable
  public String getBrowsePath() {
    return browsePath;
  }

  public void setBrowsePath(@Nullable final String browsePath) {
    this.browsePath = browsePath;
  }

  @Override
  public String toString() {
    return "ComponentSummary{" +
        "componentId=" + componentId +
        ", repositoryId=" + repositoryId +
        ", namespace='" + namespace + '\'' +
        ", name='" + name + '\'' +
        ", version='" + version + '\'' +
        ", created=" + created +
        ", createdBy='" + createdBy + '\'' +
        ", browsePath='" + browsePath + '\'' +
        '}';
  }
}
