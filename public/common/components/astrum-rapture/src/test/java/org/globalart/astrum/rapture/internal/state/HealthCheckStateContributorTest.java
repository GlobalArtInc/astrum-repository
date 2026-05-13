
package org.globalart.astrum.rapture.internal.state;

import java.util.Arrays;
import java.util.Collection;
import java.util.Collections;

import org.globalart.astrum.systemchecks.NodeSystemCheckResult;
import org.globalart.astrum.systemchecks.SystemCheckService;

import com.codahale.metrics.health.HealthCheck.Result;
import com.google.common.collect.ImmutableMap;
import org.junit.Rule;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.junit.runners.Parameterized;
import org.junit.runners.Parameterized.Parameter;
import org.junit.runners.Parameterized.Parameters;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnit;
import org.mockito.junit.MockitoRule;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasEntry;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.rapture.internal.state.HealthCheckStateContributor.HC_FAILED_KEY;

@RunWith(Parameterized.class)
public class HealthCheckStateContributorTest
{
  @Rule
  public MockitoRule mockitoRule = MockitoJUnit.rule().silent();

  @Parameters
  public static Collection<Object[]> data() {
    return Arrays.asList(new Object[][]{
        {true, Result.unhealthy("Not healthy")},
        {false, Result.healthy()}});
  }

  @Parameter
  public boolean expectedState;

  @Parameter(1)
  public Result result;

  @Mock
  private SystemCheckService systemCheckService;

  @Mock
  private NodeSystemCheckResult nodeA;

  @Mock
  private NodeSystemCheckResult nodeB;

  @InjectMocks
  private HealthCheckStateContributor subject;

  @Test
  public void testIndicator() {
    when(nodeA.getResult()).thenReturn(Collections.emptyMap());
    when(nodeB.getResult()).thenReturn(ImmutableMap.of(
        "a", Result.healthy(),
        "b", result,
        "c", Result.healthy()));
    when(systemCheckService.getResults()).thenReturn(Arrays.asList(nodeA, nodeB).stream());

    assertThat(subject.getState(), hasEntry(HC_FAILED_KEY, expectedState));
  }
}
