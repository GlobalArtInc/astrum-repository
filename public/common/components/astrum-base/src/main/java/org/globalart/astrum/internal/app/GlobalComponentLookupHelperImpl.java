
package org.globalart.astrum.internal.app;

import javax.annotation.Nullable;

import org.globalart.astrum.common.app.GlobalComponentLookupHelper;

import org.springframework.beans.BeansException;
import org.springframework.context.ApplicationContext;
import org.springframework.context.ApplicationContextAware;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Default {@link GlobalComponentLookupHelper}.
 */
@Component
public class GlobalComponentLookupHelperImpl
    implements ApplicationContextAware, GlobalComponentLookupHelper
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private ApplicationContext applicationContext;

  @Override
  @Nullable
  public Object lookup(final String className) {
    checkNotNull(className);
    checkNotNull(applicationContext);
    try {
      log.trace("Looking up component by class-name: {}", className);
      Class<?> type = getClass().getClassLoader().loadClass(className);
      return lookup(type);
    }
    catch (Exception e) {
      log.trace("Unable to lookup component by class-name: {}; ignoring", className, e);
    }
    return null;
  }

  @Override
  @Nullable
  public <T> T lookup(final Class<T> clazz) {
    checkNotNull(clazz);

    return applicationContext.getBean(clazz);
  }

  @Override
  @Nullable
  public <T> T lookup(final Class<T> clazz, final String name) {
    checkNotNull(clazz);
    checkNotNull(name);

    return applicationContext.getBean(name, clazz);
  }

  @Override
  @Nullable
  public Class<?> type(final String className) {
    checkNotNull(className);
    try {
      log.trace("Looking up type: {}", className);
      return getClass().getClassLoader().loadClass(className);
    }
    catch (Exception e) {
      log.trace("Unable to lookup type: {}; ignoring", className, e);
    }
    return null;
  }

  @Override
  public void setApplicationContext(final ApplicationContext applicationContext) throws BeansException {
    this.applicationContext = applicationContext;
  }
}
