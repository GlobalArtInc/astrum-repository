
package org.globalart.astrum.common.stateguard;

import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;

import org.aspectj.lang.ProceedingJoinPoint;
import org.aspectj.lang.annotation.Around;
import org.aspectj.lang.annotation.Aspect;
import org.aspectj.lang.reflect.MethodSignature;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Preconditions.checkState;

@Aspect
public class TransitionsAspect
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Around("@annotation(config) && execution(* *(..))")
  public Object aroundTransitionsMethod(
      final ProceedingJoinPoint joinPoint,
      final Transitions config) throws Throwable
  {
    checkNotNull(joinPoint);

    Object target = joinPoint.getTarget(); // Get the *target* object, not the proxy
    Method method = ((MethodSignature) joinPoint.getSignature()).getMethod();

    checkState(target instanceof StateGuardAware, "Invocation target (%s) does not implement: %s",
        target.getClass(), StateGuardAware.class);
    StateGuard states = ((StateGuardAware) target).getStateGuard();

    Transition transition =
        states.transition(config.to(), config.silent(), config.ignore(), config.requiresWriteLock());
    if (config.from() != null && config.from().length != 0) {
      transition = transition.from(config.from());
    }

    log.trace("Invoking: {} -> {}", transition, method);

    try {
      return transition.run(() -> {
        try {
          return joinPoint.proceed();
        }
        catch (Throwable t) {
          if (t instanceof Exception e) {
            throw e;
          }
          if (t instanceof Error e) {
            throw e;
          }
          throw new RuntimeException(t);
        }
      });
    }
    catch (InvocationTargetException e) {
      throw e.getCause();
    }
    catch (RuntimeException e) {
      if (e.getCause() != null) {
        throw e.getCause();
      }
      throw e;
    }
  }
}
