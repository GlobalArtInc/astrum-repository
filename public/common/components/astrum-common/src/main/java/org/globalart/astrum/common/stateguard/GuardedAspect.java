
package org.globalart.astrum.common.stateguard;

import java.lang.reflect.Method;

import org.aspectj.lang.ProceedingJoinPoint;
import org.aspectj.lang.annotation.Around;
import org.aspectj.lang.annotation.Aspect;
import org.aspectj.lang.reflect.MethodSignature;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

@Aspect
public class GuardedAspect
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Around("@annotation(guarded) && execution(* *(..))")
  public Object around(final ProceedingJoinPoint joinPoint, final Guarded guarded) throws Throwable {
    Method method = ((MethodSignature) joinPoint.getSignature()).getMethod();
    Object target = joinPoint.getTarget();
    StateGuard states = ((StateGuardAware) target).getStateGuard();
    Guard guard = states.guard(guarded.by());
    log.trace("Invoking: {} -> {}", guard, method);

    return guard.run(() -> {
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
}
