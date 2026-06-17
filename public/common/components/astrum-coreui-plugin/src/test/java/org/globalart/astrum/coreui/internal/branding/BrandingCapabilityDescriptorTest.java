
package org.globalart.astrum.coreui.internal.branding;

import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;

import org.globalart.astrum.capability.CapabilityType;
import org.globalart.astrum.capability.Tag;
import org.globalart.astrum.formfields.FormField;

import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;

/**
 * Tests for {@link BrandingCapabilityDescriptor}.
 */
public class BrandingCapabilityDescriptorTest
{
  private BrandingCapabilityDescriptor underTest;

  @Before
  public void setUp() {
    underTest = new BrandingCapabilityDescriptor();
  }

  @Test
  public void testType() {
    CapabilityType type = underTest.type();
    assertThat(type, is(notNullValue()));
    assertThat(type, is(BrandingCapabilityDescriptor.TYPE));
  }

  @Test
  public void testTypeId() {
    assertThat(BrandingCapabilityDescriptor.TYPE_ID, is("rapture.branding"));
  }

  @Test
  public void testName() {
    assertThat(underTest.name(), is("UI: Branding"));
  }

  @Test
  public void testFormFields() {
    List<FormField> formFields = underTest.formFields();
    assertThat(formFields, is(notNullValue()));
    assertThat(formFields, hasSize(4));
  }

  @Test
  public void testFormFieldIds() {
    List<FormField> formFields = underTest.formFields();
    assertThat(formFields.get(0).getId(), is(BrandingCapabilityConfiguration.HEADER_ENABLED));
    assertThat(formFields.get(1).getId(), is(BrandingCapabilityConfiguration.HEADER_HTML));
    assertThat(formFields.get(2).getId(), is(BrandingCapabilityConfiguration.FOOTER_ENABLED));
    assertThat(formFields.get(3).getId(), is(BrandingCapabilityConfiguration.FOOTER_HTML));
  }

  @Test
  public void testGetTags() {
    Set<Tag> tags = underTest.getTags();
    assertThat(tags, is(notNullValue()));
    assertThat(tags.isEmpty(), is(false));
    assertThat(tags, is(Tag.tags(Tag.categoryTag("UI"))));
  }

  @Test
  public void testCreateConfig() {
    Map<String, String> properties = new HashMap<>();
    properties.put(BrandingCapabilityConfiguration.HEADER_ENABLED, "true");
    properties.put(BrandingCapabilityConfiguration.HEADER_HTML, "<h1>Header</h1>");
    properties.put(BrandingCapabilityConfiguration.FOOTER_ENABLED, "false");
    properties.put(BrandingCapabilityConfiguration.FOOTER_HTML, "<p>Footer</p>");

    BrandingCapabilityConfiguration config = underTest.createConfig(properties);
    assertThat(config, is(notNullValue()));
    assertThat(config.isHeaderEnabled(), is(true));
    assertThat(config.getHeaderHtml(), is("<h1>Header</h1>"));
    assertThat(config.isFooterEnabled(), is(false));
    assertThat(config.getFooterHtml(), is("<p>Footer</p>"));
  }

  @Test
  public void testCreateConfig_emptyProperties() {
    Map<String, String> properties = new HashMap<>();

    BrandingCapabilityConfiguration config = underTest.createConfig(properties);
    assertThat(config, is(notNullValue()));
    assertThat(config.isHeaderEnabled(), is(false));
    assertThat(config.isFooterEnabled(), is(false));
  }

  @Test
  public void testTypeConstant() {
    assertThat(CapabilityType.capabilityType(BrandingCapabilityDescriptor.TYPE_ID),
        is(BrandingCapabilityDescriptor.TYPE));
  }

  @Test
  public void testFormFieldLabels() {
    List<FormField> formFields = underTest.formFields();
    assertThat(formFields.get(0).getLabel(), containsString("header"));
    assertThat(formFields.get(2).getLabel(), containsString("footer"));
  }
}
