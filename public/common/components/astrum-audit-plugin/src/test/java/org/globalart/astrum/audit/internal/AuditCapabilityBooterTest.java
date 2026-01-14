
package org.globalart.astrum.audit.internal;

import org.globalart.astrum.capability.CapabilityReferenceFilterBuilder.CapabilityReferenceFilter;
import org.globalart.astrum.capability.CapabilityRegistry;

import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class AuditCapabilityBooterTest
{
  private AuditCapabilityBooter underTest;

  @Mock
  private CapabilityRegistry capabilityRegistry;

  @Before
  public void setup() {
    underTest = new AuditCapabilityBooter();
  }

  @Test
  public void testBoot_isEnabled() throws Exception {
    when(capabilityRegistry.get(any(CapabilityReferenceFilter.class))).thenReturn(null);
    underTest.boot(capabilityRegistry);

    verify(capabilityRegistry).addNonExposed(eq(AuditCapability.TYPE), eq(true), any(), any());
  }
}
