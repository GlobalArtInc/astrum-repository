package org.globalart.astrum.repository.docker.rest;

import java.util.ArrayList;
import java.util.List;
import java.util.Locale;

import javax.annotation.Nullable;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.docker.DockerConfig;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonInclude.Include;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;

@JsonIgnoreProperties(ignoreUnknown = true)
public class DockerProxyAttributes
{
  public static final String HUB_INDEX_URL = "https://index.docker.io/";

  @Schema(description = "Type of Docker Index", allowableValues = {"HUB", "REGISTRY", "CUSTOM"}, example = "HUB")
  private final String indexType;

  @Schema(description = "Url of Docker Index to use")
  @JsonInclude(Include.NON_NULL)
  private final String indexUrl;

  @Schema(description = "Allow Astrum Repository to download and cache foreign layers")
  private final Boolean cacheForeignLayers;

  @Schema(description = "A set of regular expressions used to identify URLs that are allowed for foreign layer requests")
  private final List<String> foreignLayerUrlWhitelist;

  @JsonCreator
  public DockerProxyAttributes(
      @JsonProperty("indexType") final String indexType,
      @JsonProperty("indexUrl") final String indexUrl,
      @JsonProperty("cacheForeignLayers") final Boolean cacheForeignLayers,
      @JsonProperty("foreignLayerUrlWhitelist") final List<String> foreignLayerUrlWhitelist)
  {
    this.indexType = indexType;
    this.indexUrl = indexUrl;
    this.cacheForeignLayers = cacheForeignLayers;
    this.foreignLayerUrlWhitelist = foreignLayerUrlWhitelist;
  }

  public String getIndexType() {
    return indexType;
  }

  public String getIndexUrl() {
    return indexUrl;
  }

  public Boolean getCacheForeignLayers() {
    return cacheForeignLayers;
  }

  public List<String> getForeignLayerUrlWhitelist() {
    return foreignLayerUrlWhitelist;
  }

  public static void apply(@Nullable final DockerProxyAttributes dockerProxy, final NestedAttributesMap target) {
    String type = dockerProxy == null || dockerProxy.indexType == null
        ? DockerConfig.INDEX_TYPE_REGISTRY
        : dockerProxy.indexType.trim().toUpperCase(Locale.ENGLISH);
    target.set(DockerConfig.INDEX_TYPE, type);
    String url = dockerProxy == null ? null : dockerProxy.indexUrl;
    if ("HUB".equals(type) && (url == null || url.isBlank())) {
      url = HUB_INDEX_URL;
    }
    if (url != null && !url.isBlank() && !DockerConfig.INDEX_TYPE_REGISTRY.equals(type)) {
      target.set(DockerConfig.INDEX_URL, url.trim());
    }
    boolean cache = dockerProxy != null && Boolean.TRUE.equals(dockerProxy.cacheForeignLayers);
    target.set(DockerConfig.CACHE_FOREIGN_LAYERS, cache);
    List<String> whitelist = dockerProxy == null || dockerProxy.foreignLayerUrlWhitelist == null
        ? new ArrayList<>()
        : new ArrayList<>(dockerProxy.foreignLayerUrlWhitelist);
    target.set(DockerConfig.FOREIGN_LAYER_URL_WHITELIST, whitelist);
  }

  public static DockerProxyAttributes from(final NestedAttributesMap dockerProxy) {
    Object type = dockerProxy.get(DockerConfig.INDEX_TYPE);
    Object url = dockerProxy.get(DockerConfig.INDEX_URL);
    return new DockerProxyAttributes(
        type == null ? DockerConfig.INDEX_TYPE_REGISTRY : type.toString(),
        url == null ? null : url.toString(),
        DockerConfig.bool(dockerProxy.get(DockerConfig.CACHE_FOREIGN_LAYERS), false),
        new ArrayList<>(DockerConfig.stringList(dockerProxy.get(DockerConfig.FOREIGN_LAYER_URL_WHITELIST))));
  }
}
