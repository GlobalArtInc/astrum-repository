
package org.globalart.astrum.blobstore.s3.internal.capability;

import java.util.Arrays;
import java.util.List;
import java.util.Map;
import java.util.stream.Collectors;
import org.springframework.beans.factory.annotation.Autowired;
import jakarta.validation.constraints.NotBlank;

import org.globalart.astrum.blobstore.SelectOption;
import org.globalart.astrum.capability.CapabilityConfigurationSupport;

import static com.google.common.base.Preconditions.checkNotNull;

public class CustomS3RegionCapabilityConfiguration
    extends CapabilityConfigurationSupport
{
  public static final String REGIONS = "regions";

  @NotBlank
  private String customRegions;

  @Autowired
  public CustomS3RegionCapabilityConfiguration(final Map<String, String> properties) {
    checkNotNull(properties);
    this.customRegions = properties.get(REGIONS);
  }

  public String getRegions() {
    return customRegions;
  }

  public List<SelectOption> getRegionsList() {
    return Arrays.stream(customRegions.split(","))
        .map(String::trim)
        .map(region -> new SelectOption(region, region))
        .collect(Collectors.toList());
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "regions='" + customRegions + '\'' +
        '}';
  }
}
