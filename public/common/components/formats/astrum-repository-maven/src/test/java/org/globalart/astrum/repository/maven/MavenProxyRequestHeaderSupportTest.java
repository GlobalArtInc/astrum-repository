

package org.globalart.astrum.repository.maven;

import java.util.Collections;

import org.globalart.astrum.capability.CapabilityContext;
import org.globalart.astrum.capability.CapabilityReference;
import org.globalart.astrum.capability.CapabilityReferenceFilterBuilder.CapabilityReferenceFilter;
import org.globalart.astrum.capability.CapabilityRegistry;
import org.globalart.astrum.common.app.ApplicationVersion;
import org.globalart.astrum.utils.httpclient.UserAgentGenerator;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.junit.Assert.assertEquals;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class MavenProxyRequestHeaderSupportTest

{
  @Mock
  private CapabilityRegistry capabilityRegistry;

  @Mock
  private ApplicationVersion applicationVersion;

  private UserAgentGenerator userAgentGenerator;

  private MavenProxyRequestHeaderSupport underTest;

  @Before
  public void setUp() {
    userAgentGenerator = new UserAgentGenerator(applicationVersion);
    this.underTest = new MavenProxyRequestHeaderSupport(capabilityRegistry, userAgentGenerator);
    when(applicationVersion.getEdition()).thenReturn("edition");
  }

  @Test
  public void testUserAgentWithAnalyticsNotConfigured() {
    String userAgentForAnalytics = underTest.getUserAgentForAnalytics();
    String expectedUserAgent = userAgentGenerator.generate().replace(")", "; pau)");
    assertEquals(expectedUserAgent, userAgentForAnalytics);
  }

  @Test
  public void testUserAgentWithAnalyticsEnabled() {
    CapabilityReference capabilityReference = mockCapabilityReference();
    when(capabilityReference.context().isEnabled()).thenReturn(true);
    String userAgentForAnalytics = underTest.getUserAgentForAnalytics();
    String expectedUserAgent = userAgentGenerator.generate().replace(")", "; pae)");
    assertEquals(expectedUserAgent, userAgentForAnalytics);
  }

  @Test
  public void testUserAgentWithAnalyticsDisabled() {
    CapabilityReference capabilityReference = mockCapabilityReference();
    when(capabilityReference.context().isEnabled()).thenReturn(false);
    String userAgentForAnalytics = underTest.getUserAgentForAnalytics();
    String expectedUserAgent = userAgentGenerator.generate().replace(")", "; pad)");
    assertEquals(expectedUserAgent, userAgentForAnalytics);
  }

  private CapabilityReference mockCapabilityReference() {
    CapabilityReference capabilityReference = mock(CapabilityReference.class);
    CapabilityContext capabilityContext = mock(CapabilityContext.class);
    when(capabilityRegistry.get(any(CapabilityReferenceFilter.class))).then(
        i -> Collections.singleton(capabilityReference));
    when(capabilityReference.context()).thenReturn(capabilityContext);
    return capabilityReference;
  }
}
