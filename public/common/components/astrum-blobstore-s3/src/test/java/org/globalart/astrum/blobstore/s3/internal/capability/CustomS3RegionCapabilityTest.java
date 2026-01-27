
package org.globalart.astrum.blobstore.s3.internal.capability;

import java.util.HashMap;
import java.util.Map;

import org.junit.Before;
import org.junit.Test;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNotNull;

public class CustomS3RegionCapabilityTest
{
  private CustomS3RegionCapability capability;

  @Before
  public void setUp() {
    capability = new CustomS3RegionCapability();
  }

  @Test
  public void testCreateConfig() {
    Map<String, String> properties = new HashMap<>();
    properties.put("regions", "us-east-1,us-west-2");

    CustomS3RegionCapabilityConfiguration config = capability.createConfig(properties);

    assertNotNull(config);
    assertEquals("us-east-1,us-west-2", config.getRegions());
  }
}
