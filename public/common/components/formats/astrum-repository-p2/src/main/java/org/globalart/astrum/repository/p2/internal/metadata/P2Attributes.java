
package org.globalart.astrum.repository.p2.internal.metadata;

import java.util.Objects;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.p2.internal.AssetKind;

import static java.util.Optional.ofNullable;

public final class P2Attributes
{
  private final String pluginName;

  private final String componentName;

  private final String componentVersion;

  private final String path;

  private final String fileName;

  private final String extension;

  private final AssetKind assetKind;

  private P2Attributes(final Builder builder) {
    this.pluginName = builder.pluginName;
    this.componentName = builder.componentName;
    this.componentVersion = builder.componentVersion;
    this.path = builder.path;
    this.fileName = builder.fileName;
    this.extension = builder.extension;
    this.assetKind = builder.assetKind;
  }

  public static Builder builder() {
    return new Builder();
  }

  public boolean isEmpty() {
    return pluginName == null && componentName == null && componentVersion == null && path == null &&
        fileName == null && extension == null && assetKind == null;
  }

  @Nullable
  public String getPluginName() {
    return pluginName;
  }

  @Nullable
  public String getComponentName() {
    return componentName;
  }

  @Nullable
  public String getComponentVersion() {
    return componentVersion;
  }

  @Nullable
  public String getPath() {
    return path;
  }

  @Nullable
  public String getFileName() {
    return fileName;
  }

  @Nullable
  public String getExtension() {
    return extension;
  }

  @Nullable
  public AssetKind getAssetKind() {
    return assetKind;
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (!(o instanceof P2Attributes)) {
      return false;
    }
    P2Attributes that = (P2Attributes) o;
    return Objects.equals(pluginName, that.pluginName) &&
        Objects.equals(componentName, that.componentName) &&
        Objects.equals(componentVersion, that.componentVersion) &&
        Objects.equals(path, that.path) &&
        Objects.equals(fileName, that.fileName) &&
        Objects.equals(extension, that.extension) &&
        assetKind == that.assetKind;
  }

  @Override
  public int hashCode() {
    return Objects.hash(pluginName, componentName, componentVersion, path, fileName, extension, assetKind);
  }

  @Override
  public String toString() {
    return "P2Attributes{" +
        "pluginName='" + pluginName + '\'' +
        ", componentName='" + componentName + '\'' +
        ", componentVersion='" + componentVersion + '\'' +
        ", path='" + path + '\'' +
        ", fileName='" + fileName + '\'' +
        ", extension='" + extension + '\'' +
        ", assetKind=" + assetKind +
        '}';
  }

  public static final class Builder
  {
    private String pluginName;

    private String componentName;

    private String componentVersion;

    private String path;

    private String fileName;

    private String extension;

    private AssetKind assetKind;

    private Builder() {
    }

    public Builder pluginName(final String pluginName) {
      this.pluginName = pluginName;
      return this;
    }

    public Builder componentName(final String componentName) {
      this.componentName = componentName;
      return this;
    }

    public Builder componentVersion(final String componentVersion) {
      this.componentVersion = componentVersion;
      return this;
    }

    public Builder path(final String path) {
      this.path = path;
      return this;
    }

    public Builder fileName(final String fileName) {
      this.fileName = fileName;
      return this;
    }

    public Builder extension(final String extension) {
      this.extension = extension;
      return this;
    }

    public Builder assetKind(final AssetKind assetKind) {
      this.assetKind = assetKind;
      return this;
    }

    public Builder merge(final P2Attributes one, final P2Attributes two) {
      componentVersion(ofNullable(two.getComponentVersion()).orElse(one.getComponentVersion()));
      componentName(ofNullable(two.getComponentName()).orElse(one.getComponentName()));
      pluginName(ofNullable(two.getPluginName()).orElse(one.getPluginName()));
      path(ofNullable(two.getPath()).orElse(one.getPath()));
      fileName(ofNullable(two.getFileName()).orElse(one.getFileName()));
      extension(ofNullable(two.getExtension()).orElse(one.getExtension()));
      assetKind(ofNullable(two.getAssetKind()).orElse(one.getAssetKind()));
      return this;
    }

    public P2Attributes build() {
      return new P2Attributes(this);
    }
  }
}
