
package org.globalart.astrum.blobstore.s3.internal.capability;

import java.util.HashMap;
import java.util.Map;

import org.junit.Before;
import org.junit.Test;

import static org.junit.Assert.assertEquals;

public class CustomS3RegionCapabilityConfigurationTest
{

  private CustomS3RegionCapabilityConfiguration config;

  @Before
  public void setUp() {
    Map<String, String> properties = new HashMap<>();
    properties.put("regions", "us-east-1,us-west-2");
    config = new CustomS3RegionCapabilityConfiguration(properties);
  }

  @Test
  public void testGetRegions() {
    assertEquals("us-east-1,us-west-2", config.getRegions());
  }
}
