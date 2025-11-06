
package org.globalart.astrum.bootstrap.entrypoint;

import org.springframework.context.ApplicationContext;
import org.springframework.context.ApplicationContextAware;
import org.springframework.stereotype.Component;

@Component
public class ApplicationContextProvider
    implements ApplicationContextAware
{
  private static ApplicationContext context;

  @Override
  public void setApplicationContext(ApplicationContext applicationContext) {
    ApplicationContextProvider.context = applicationContext;
  }

  public static ApplicationContext getApplicationContext() {
    if (context == null) {
      throw new IllegalStateException("The spring context has not yet been initialized.");
    }
    return context;
  }

  public static <T> T getBean(final Class<T> clazz) {
    if (context == null) {
      throw new IllegalStateException("The spring context has not yet been initialized.");
    }
    return context.getBean(clazz);
  }
}
