
package org.globalart.astrum.repository.httpbridge.legacy;

import org.globalart.astrum.capability.CapabilityContext;
import org.globalart.astrum.capability.CapabilityReference;
import org.globalart.astrum.capability.CapabilityReferenceFilterBuilder.CapabilityReferenceFilter;
import org.globalart.astrum.capability.CapabilityRegistry;

import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static java.util.Collections.emptyList;
import static java.util.Collections.singletonList;
import static org.hamcrest.core.Is.is;
import static org.hamcrest.core.IsEqual.equalTo;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.doReturn;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class LegacyUrlEnabledHelperTest
{
  private static final String LEGACY_ENABLED =
      "org.globalart.astrum.repository.httpbridge.internal.HttpBridgeModule.legacy";

  @Mock
  private CapabilityRegistry capabilityRegistry;

  @Mock
  private CapabilityReference capabilityReference;

  @Mock
  private CapabilityContext capabilityContext;

  private LegacyUrlEnabledHelper undertTest;

  @Before
  public void setup() throws Exception {
    System.setProperty(LEGACY_ENABLED, "false");

    doReturn(singletonList(capabilityReference)).when(capabilityRegistry).get(any(CapabilityReferenceFilter.class));
    when(capabilityReference.context()).thenReturn(capabilityContext);
    when(capabilityContext.isActive()).thenReturn(false);

    undertTest = new LegacyUrlEnabledHelper(capabilityRegistry);
  }

  @Test
  public void enabledOnProperty() throws Exception {
    System.setProperty(LEGACY_ENABLED, "true");

    undertTest = new LegacyUrlEnabledHelper(capabilityRegistry);

    assertThat(undertTest.isEnabled(), is(equalTo(true)));
  }

  @Test
  public void enabledWhenCapabilityActive() throws Exception {
    when(capabilityContext.isActive()).thenReturn(true);

    assertThat(undertTest.isEnabled(), is(equalTo(true)));
  }

  @Test
  public void disabledWhenPropertyFalseAndCapabilityNotActive() throws Exception {
    assertThat(undertTest.isEnabled(), is(equalTo(false)));
  }

  @Test
  public void disabledWhenPropertyFalseAndCapabilityNotFound() throws Exception {
    when(capabilityContext.isActive()).thenReturn(true);
    when(capabilityRegistry.get(any(CapabilityReferenceFilter.class))).thenReturn(emptyList());

    assertThat(undertTest.isEnabled(), is(equalTo(false)));
  }
}
