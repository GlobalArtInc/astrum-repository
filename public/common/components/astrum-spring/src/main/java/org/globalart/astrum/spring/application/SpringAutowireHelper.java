
package org.globalart.astrum.spring.application;

import org.springframework.beans.BeansException;
import org.springframework.beans.factory.config.AutowireCapableBeanFactory;
import org.springframework.context.ApplicationContext;
import org.springframework.context.ApplicationContextAware;
import org.springframework.stereotype.Component;

/**
 * Helper class to autowire beans out of the Spring context.
 */
@Component
public class SpringAutowireHelper
    implements ApplicationContextAware
{
  private static AutowireCapableBeanFactory autowireBeanFactory;

  @Override
  public void setApplicationContext(final ApplicationContext applicationContext) throws BeansException {
    autowireBeanFactory = applicationContext.getAutowireCapableBeanFactory();
  }

  public static void autowireBean(final Object bean) throws BeansException {
    if (bean != null && autowireBeanFactory != null) {
      autowireBeanFactory.autowireBean(bean);
    }
  }
}
