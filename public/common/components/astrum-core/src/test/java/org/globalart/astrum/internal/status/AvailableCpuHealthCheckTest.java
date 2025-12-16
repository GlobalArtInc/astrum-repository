
package org.globalart.astrum.internal.status;

import com.codahale.metrics.health.HealthCheck.Result;
import org.junit.Test;

import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.MatcherAssert.assertThat;

public class AvailableCpuHealthCheckTest
{
  private AvailableCpuHealthCheck underTest;

  @Test
  public void testCheck_plentyOfCores() {
    underTest = new AvailableCpuHealthCheck(0);
    Result result = underTest.check();
    assertThat(result.isHealthy(), is(true));
    assertThat(result.getMessage(), is("The host system is allocating a maximum of "
        + Runtime.getRuntime().availableProcessors() + " cores to the application."));
  }

  @Test
  public void testCheck_notEnoughCores() {
    underTest = new AvailableCpuHealthCheck(1024);
    Result result = underTest.check();
    assertThat(result.isHealthy(), is(false));
    assertThat(result.getMessage(),
        is("The host system is allocating a maximum of " + Runtime.getRuntime().availableProcessors()
            + " cores to the application." + " A minimum of 1024 is recommended."));
  }
}
