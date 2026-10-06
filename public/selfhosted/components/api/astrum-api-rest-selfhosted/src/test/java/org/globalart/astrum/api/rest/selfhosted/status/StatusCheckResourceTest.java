
package org.globalart.astrum.api.rest.selfhosted.status;

import java.util.SortedMap;

import org.globalart.astrum.testcommon.extensions.AuthenticationExtension;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension.WithUser;
import org.globalart.astrum.testcommon.validation.ValidationExtension;

import com.codahale.metrics.health.HealthCheck.Result;
import com.codahale.metrics.health.HealthCheckRegistry;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.junit.Assert.assertEquals;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
@ExtendWith(ValidationExtension.class)
@ExtendWith(AuthenticationExtension.class)
@WithUser
class StatusCheckResourceTest
{
  @Mock
  private HealthCheckRegistry registry;

  @InjectMocks
  private StatusCheckResource resource;

  @Test
  void returnsTheSystemStatusChecks() {
    SortedMap<String, Result> expectedStatusChecks = mock(SortedMap.class);
    when(registry.runHealthChecks()).thenReturn(expectedStatusChecks);
    SortedMap<String, Result> systemStatusChecks = resource.getSystemStatusChecks();
    assertEquals(expectedStatusChecks, systemStatusChecks);
  }
}
