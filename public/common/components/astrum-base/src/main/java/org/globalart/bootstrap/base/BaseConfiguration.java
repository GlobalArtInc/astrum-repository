
package org.globalart.bootstrap.base;

import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class BaseConfiguration
{
  @Deprecated
  @Bean("astrum-uber")
  public ClassLoader uberClassLoader() {
    return BaseConfiguration.class.getClassLoader();
  }
}
