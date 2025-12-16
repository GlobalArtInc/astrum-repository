
package org.globalart.astrum.security.aspect;

import org.apache.shiro.authz.annotation.RequiresUser;
import org.apache.shiro.authz.aop.UserAnnotationHandler;
import org.aspectj.lang.annotation.Aspect;
import org.aspectj.lang.annotation.Before;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * {@link Aspect} that ensures there is a current subject
 */
@Aspect
public class RequiresUserAspect
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final UserAnnotationHandler handler = new UserAnnotationHandler();

  @Before("@annotation(requiresUser) && execution(* *(..))")
  public void assertAuthorized(
      final RequiresUser requiresUser) throws Throwable
  {
    handler.assertAuthorized(requiresUser);
  }
}
