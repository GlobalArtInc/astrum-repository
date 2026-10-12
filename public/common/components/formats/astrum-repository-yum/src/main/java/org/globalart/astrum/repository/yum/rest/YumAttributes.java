package org.globalart.astrum.repository.yum.rest;

import java.util.Map;

import javax.annotation.Nullable;

import jakarta.validation.constraints.Max;
import jakarta.validation.constraints.Min;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.yum.internal.LayoutPolicy;
import org.globalart.astrum.repository.yum.internal.hosted.YumHostedFacet;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;

public class YumAttributes
{
  @Schema(description = "Directory depth below the repository root at which repodata is generated (0 to 5)",
      example = "0", minimum = "0", maximum = "5")
  @Min(0)
  @Max(5)
  private final Integer repodataDepth;

  @Schema(description = "Whether uploads must be RPM packages at least repodataDepth directories deep (STRICT) "
      + "or may be any file at any path (PERMISSIVE)", example = "STRICT")
  private final LayoutPolicy deployPolicy;

  @JsonCreator
  public YumAttributes(
      @JsonProperty(YumHostedFacet.REPODATA_DEPTH) final Integer repodataDepth,
      @JsonProperty(YumHostedFacet.DEPLOY_POLICY) final LayoutPolicy deployPolicy)
  {
    this.repodataDepth = repodataDepth;
    this.deployPolicy = deployPolicy;
  }

  public Integer getRepodataDepth() {
    return repodataDepth;
  }

  public LayoutPolicy getDeployPolicy() {
    return deployPolicy;
  }

  public static YumAttributes of(final Repository repository) {
    Object section = repository.getConfiguration().getAttributes().get(YumHostedFacet.CONFIG_KEY);
    Integer depth = 0;
    LayoutPolicy policy = LayoutPolicy.STRICT;
    if (section instanceof Map<?, ?> map) {
      if (map.get(YumHostedFacet.REPODATA_DEPTH) instanceof Number number) {
        depth = number.intValue();
      }
      if (map.get(YumHostedFacet.DEPLOY_POLICY) instanceof String value) {
        try {
          policy = LayoutPolicy.valueOf(value);
        }
        catch (IllegalArgumentException e) {
          policy = LayoutPolicy.STRICT;
        }
      }
    }
    return new YumAttributes(depth, policy);
  }

  public static void apply(final Configuration configuration, @Nullable final YumAttributes yum) {
    int depth = yum == null || yum.getRepodataDepth() == null ? 0 : yum.getRepodataDepth();
    LayoutPolicy policy = yum == null || yum.getDeployPolicy() == null ? LayoutPolicy.STRICT : yum.getDeployPolicy();
    configuration.attributes(YumHostedFacet.CONFIG_KEY).set(YumHostedFacet.REPODATA_DEPTH, depth);
    configuration.attributes(YumHostedFacet.CONFIG_KEY).set(YumHostedFacet.DEPLOY_POLICY, policy.name());
  }
}
