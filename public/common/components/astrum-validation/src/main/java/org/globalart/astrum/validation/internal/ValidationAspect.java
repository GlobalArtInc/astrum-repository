
package org.globalart.astrum.validation.internal;

import java.lang.reflect.Method;
import java.util.Arrays;
import java.util.Set;

import jakarta.validation.ConstraintViolation;
import jakarta.validation.ConstraintViolationException;

import org.globalart.astrum.bootstrap.validation.ValidationConfiguration;
import org.globalart.astrum.validation.Validate;

import org.aspectj.lang.ProceedingJoinPoint;
import org.aspectj.lang.annotation.Around;
import org.aspectj.lang.annotation.Aspect;
import org.aspectj.lang.reflect.MethodSignature;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * {@link Aspect} that validates method arguments and return values.
 */
@Aspect
public class ValidationAspect
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Around("@annotation(validate) && execution(* *(..))")
  public Object validateMethod(final ProceedingJoinPoint joinPoint, final Validate validate) throws Throwable {
    final ClassLoader tccl = Thread.currentThread().getContextClassLoader();
    try {
      Thread.currentThread().setContextClassLoader(ValidationAspect.class.getClassLoader());

      Method method = ((MethodSignature) joinPoint.getSignature()).getMethod();

      validateParameters(joinPoint.getTarget(), method, joinPoint.getArgs(), validate.groups());

      Object result = joinPoint.proceed();

      validateReturnValue(joinPoint.getTarget(), method, result, validate.groups());

      return result;
    }
    finally {
      Thread.currentThread().setContextClassLoader(tccl);
    }
  }

  private void validateParameters(final Object obj, final Method method, final Object[] args, final Class<?>[] groups) {
    Set<ConstraintViolation<Object>> violations =
        ValidationConfiguration.EXECUTABLE_VALIDATOR.validateParameters(obj, method, args, groups);
    if (!violations.isEmpty()) {
      String message = "Invalid arguments calling '" + method + "' with " + Arrays.deepToString(args);
      throw new ConstraintViolationException(message, violations);
    }
  }

  private void validateReturnValue(final Object obj, final Method method, final Object value, final Class<?>[] groups) {
    Set<ConstraintViolation<Object>> violations =
        ValidationConfiguration.EXECUTABLE_VALIDATOR.validateReturnValue(obj, method, value, groups);
    if (!violations.isEmpty()) {
      String message = "Invalid value returned by '" + method + "' was " + value;
      throw new ConstraintViolationException(message, violations);
    }
  }
}
