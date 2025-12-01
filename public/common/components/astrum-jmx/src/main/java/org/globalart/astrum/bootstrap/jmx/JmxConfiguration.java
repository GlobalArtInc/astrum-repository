
package org.globalart.astrum.bootstrap.jmx;

import java.lang.management.ManagementFactory;

import javax.management.MBeanServer;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class JmxConfiguration
{
  /**
   * Provides the platform {@link MBeanServer}.
   *
   * @since 3.0
   */
  @Qualifier("platform")
  @Bean
  public MBeanServer platform() {
    return ManagementFactory.getPlatformMBeanServer();
  }
}
