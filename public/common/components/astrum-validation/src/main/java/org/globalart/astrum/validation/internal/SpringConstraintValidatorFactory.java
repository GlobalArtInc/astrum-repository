
package org.globalart.astrum.validation.internal;

import java.lang.reflect.Constructor;

import jakarta.validation.ConstraintValidator;
import jakarta.validation.ConstraintValidatorFactory;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.NoSuchBeanDefinitionException;
import org.springframework.context.ApplicationContext;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Allow {@link ConstraintValidator validators} to be enhanced.
 *
 * @since 3.0
 */
@Component
public final class SpringConstraintValidatorFactory
    implements ConstraintValidatorFactory
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final ApplicationContext applicationContext;

  @Autowired
  public SpringConstraintValidatorFactory(final ApplicationContext applicationContext) {
    this.applicationContext = checkNotNull(applicationContext);
  }

  @Override
  public <T extends ConstraintValidator<?, ?>> T getInstance(final Class<T> key) {
    log.trace("Resolving validator instance for type: {}", key);
    try {
      T validator = applicationContext.getBean(key);
      if (validator != null) {
        return validator;
      }
    }
    catch (NoSuchBeanDefinitionException e) {
      log.debug("No bean found in context, trying manual creation");
    }
    return create(key);
  }

  @Override
  public void releaseInstance(final ConstraintValidator<?, ?> instance) {
    // empty
  }

  private static <T extends ConstraintValidator<?, ?>> T create(final Class<T> key) {
    try {
      Constructor<T> ctor = key.getDeclaredConstructor();
      ctor.setAccessible(true);
      return ctor.newInstance();
    }
    catch (Exception e) {
      throw new NoSuchBeanDefinitionException(key);
    }
  }
}
