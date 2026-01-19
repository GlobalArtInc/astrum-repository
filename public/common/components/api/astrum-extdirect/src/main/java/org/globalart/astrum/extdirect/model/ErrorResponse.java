
package org.globalart.astrum.extdirect.model;

import com.google.common.collect.Lists;
import org.apache.shiro.SecurityUtils;
import org.apache.shiro.authz.UnauthenticatedException;
import org.apache.shiro.subject.Subject;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Ext.Direct error response.
 *
 * @since 3.0
 */
public class ErrorResponse
    extends Response<Object>
{
  private String message;

  private boolean authenticationRequired;

  public ErrorResponse(final Throwable cause) {
    this(checkNotNull(cause).getMessage() == null ? cause.getClass().getName() : cause.getMessage());
    authenticationRequired = cause instanceof UnauthenticatedException;
    if (authenticationRequired) {
      Subject subject = SecurityUtils.getSubject();
      if (subject == null || !(subject.isRemembered() || subject.isAuthenticated())) {
        message = "Access denied (authentication required)";
      }
    }
  }

  public ErrorResponse(final String message) {
    super(false, Lists.newArrayList());
    this.message = checkNotNull(message);
  }

  public String getMessage() {
    return message;
  }
}
