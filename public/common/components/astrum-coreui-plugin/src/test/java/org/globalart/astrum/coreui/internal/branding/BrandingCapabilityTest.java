
package org.globalart.astrum.coreui.internal.branding;

import java.util.HashMap;
import java.util.Map;

import org.globalart.astrum.capability.Condition;
import org.globalart.astrum.capability.condition.CapabilityConditions;
import org.globalart.astrum.capability.condition.Conditions;

import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.junit.Assert.assertThrows;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

/**
 * Tests for {@link BrandingCapability}.
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class BrandingCapabilityTest
{
  @Mock
  private Branding branding;

  @Mock
  private Conditions conditions;

  @Mock
  private CapabilityConditions capabilityConditions;

  private BrandingCapability underTest;

  @Before
  public void setUp() {
    underTest = new BrandingCapability(branding);
    when(conditions.capabilities()).thenReturn(capabilityConditions);
  }

  @Test
  public void testConstructor_nullBranding() {
    assertThrows(NullPointerException.class, () -> new BrandingCapability(null));
  }

  @Test
  public void testCreateConfig() {
    Map<String, String> properties = new HashMap<>();
    properties.put(BrandingCapabilityConfiguration.HEADER_ENABLED, "true");
    properties.put(BrandingCapabilityConfiguration.HEADER_HTML, "<h1>Test</h1>");
    properties.put(BrandingCapabilityConfiguration.FOOTER_ENABLED, "false");
    properties.put(BrandingCapabilityConfiguration.FOOTER_HTML, "<p>Footer</p>");

    BrandingCapabilityConfiguration config = underTest.createConfig(properties);
    assertThat(config, is(notNullValue()));
    assertThat(config.isHeaderEnabled(), is(true));
    assertThat(config.getHeaderHtml(), is("<h1>Test</h1>"));
    assertThat(config.isFooterEnabled(), is(false));
    assertThat(config.getFooterHtml(), is("<p>Footer</p>"));
  }

  @Test
  public void testOnActivate_setsBranding() throws Exception {
    Map<String, String> properties = new HashMap<>();
    properties.put(BrandingCapabilityConfiguration.HEADER_ENABLED, "true");
    properties.put(BrandingCapabilityConfiguration.HEADER_HTML, "<h1>Test</h1>");
    properties.put(BrandingCapabilityConfiguration.FOOTER_ENABLED, "true");
    properties.put(BrandingCapabilityConfiguration.FOOTER_HTML, "<p>Footer</p>");

    BrandingCapabilityConfiguration config = new BrandingCapabilityConfiguration(properties);
    underTest.onActivate(config);

    verify(branding).set(config);
  }

  @Test
  public void testOnPassivate_resetsBranding() throws Exception {
    Map<String, String> properties = new HashMap<>();
    BrandingCapabilityConfiguration config = new BrandingCapabilityConfiguration(properties);
    underTest.onPassivate(config);

    verify(branding).reset();
  }

  @Test
  public void testOnRemove_resetsBranding() throws Exception {
    Map<String, String> properties = new HashMap<>();
    BrandingCapabilityConfiguration config = new BrandingCapabilityConfiguration(properties);
    underTest.onRemove(config);

    verify(branding).reset();
  }

  @Test
  public void testActivationCondition() {
    Condition expectedCondition = mock(Condition.class);
    when(capabilityConditions.passivateCapabilityDuringUpdate()).thenReturn(expectedCondition);

    underTest.installConditionComponents(conditions);
    Condition condition = underTest.activationCondition();

    assertThat(condition, is(expectedCondition));
    verify(conditions).capabilities();
    verify(capabilityConditions).passivateCapabilityDuringUpdate();
  }
}
