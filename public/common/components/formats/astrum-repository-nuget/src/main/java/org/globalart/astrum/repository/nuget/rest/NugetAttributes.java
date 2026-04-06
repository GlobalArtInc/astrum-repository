package org.globalart.astrum.repository.nuget.rest;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;

public class NugetAttributes
{
  public static final String QUERY_CACHE_ITEM_MAX_AGE = "queryCacheItemMaxAge";

  public static final String NUGET_VERSION = "nugetVersion";

  public static final int DEFAULT_QUERY_CACHE_ITEM_MAX_AGE = 3600;

  @Schema(description = "How long to cache query results from the proxied repository (in seconds)", example = "3600")
  private final Integer queryCacheItemMaxAge;

  @Schema(description = "NuGet protocol version of the remote repository", example = "V3",
      allowableValues = {"V2", "V3"})
  private final String nugetVersion;

  @JsonCreator
  public NugetAttributes(
      @JsonProperty(QUERY_CACHE_ITEM_MAX_AGE) final Integer queryCacheItemMaxAge,
      @JsonProperty(NUGET_VERSION) final String nugetVersion)
  {
    this.queryCacheItemMaxAge = queryCacheItemMaxAge;
    this.nugetVersion = nugetVersion;
  }

  public Integer getQueryCacheItemMaxAge() {
    return queryCacheItemMaxAge;
  }

  public String getNugetVersion() {
    return nugetVersion;
  }
}
