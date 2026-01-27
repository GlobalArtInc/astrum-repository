
package org.globalart.astrum.blobstore.s3.internal.capability;

import java.util.HashMap;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.capability.CapabilityType;
import org.globalart.astrum.formfields.FormField;

import org.junit.Before;
import org.junit.Test;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNotNull;

public class CustomS3RegionCapabilityDescriptorTest
{

  private CustomS3RegionCapabilityDescriptor descriptor;

  @Before
  public void setUp() {
    descriptor = new CustomS3RegionCapabilityDescriptor();
  }

  @Test
  public void testType() {
    CapabilityType type = descriptor.type();
    assertEquals("customs3regions", type.toString());
  }

  @Test
  public void testName() {
    String name = descriptor.name();
    assertEquals("Custom S3 Regions", name);
  }

  @Test
  public void testFormFields() {
    List<FormField> formFields = descriptor.formFields();
    assertNotNull(formFields);
    assertEquals(1, formFields.size());
    assertEquals("regions", formFields.get(0).getId());
  }

  @Test
  public void testCreateConfig() {
    Map<String, String> properties = new HashMap<>();
    properties.put("regions", "us-east-1,us-west-2");
    assertNotNull(descriptor.createConfig(properties));
  }
}
