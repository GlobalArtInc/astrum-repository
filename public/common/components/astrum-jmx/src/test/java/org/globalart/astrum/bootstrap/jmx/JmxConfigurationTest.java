
package org.globalart.astrum.bootstrap.jmx;

import java.lang.management.ManagementFactory;

import javax.management.MBeanServer;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.notNullValue;
import static org.hamcrest.Matchers.sameInstance;

public class JmxConfigurationTest
{
  @Test
  public void platformReturnsPlatformMBeanServer() {
    MBeanServer mbeanServer = new JmxConfiguration().platform();

    assertThat(mbeanServer, notNullValue());
    assertThat(mbeanServer, sameInstance(ManagementFactory.getPlatformMBeanServer()));
  }
}
