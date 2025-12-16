
package org.globalart.astrum.security.aspect;

import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.aop.AuthenticatedAnnotationHandler;
import org.aspectj.lang.annotation.Aspect;
import org.aspectj.lang.annotation.Before;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * {@link Aspect} that ensures the current subject is authenticated
 */
@Aspect
public class RequiresAuthenticationAspect
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final AuthenticatedAnnotationHandler handler = new AuthenticatedAnnotationHandler();

  @Before("@annotation(requiresAuthentication) && execution(* *(..))")
  public void assertAuthorized(
      final RequiresAuthentication requiresAuthentication) throws Throwable
  {
    handler.assertAuthorized(requiresAuthentication);
  }
}
