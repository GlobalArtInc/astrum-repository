
package org.globalart.astrum.security.aspect;

import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.apache.shiro.authz.aop.PermissionAnnotationHandler;
import org.aspectj.lang.annotation.Aspect;
import org.aspectj.lang.annotation.Before;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * {@link Aspect} that ensures the current subject has the specified permissions
 */
@Aspect
public class RequiresPermissionsAspect
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final PermissionAnnotationHandler handler = new PermissionAnnotationHandler();

  @Before("@annotation(requiresPermissions) && execution(* *(..))")
  public void assertPermitted(
      final RequiresPermissions requiresPermissions) throws Throwable
  {
    handler.assertAuthorized(requiresPermissions);
  }
}
