
package org.globalart.astrum.extdirect.internal;

import java.sql.SQLException;
import java.util.Arrays;
import java.util.List;
import java.util.Set;

import jakarta.validation.ConstraintViolation;
import jakarta.validation.ConstraintViolationException;

import org.apache.shiro.authz.AuthorizationException;
import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.extdirect.model.Response;
import org.globalart.astrum.rest.ValidationErrorsException;

import com.softwarementors.extjs.djn.api.RegisteredMethod;
import org.apache.commons.collections4.ListUtils;
import org.apache.http.conn.HttpHostConnectException;
import org.apache.shiro.authc.AuthenticationException;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.SERVICES;
import static org.globalart.astrum.extdirect.model.Responses.error;
import static org.globalart.astrum.extdirect.model.Responses.invalid;
import org.springframework.stereotype.Component;

/**
 * @since 3.15
 */
@Component
@ManagedLifecycle(phase = SERVICES)
public class ExtDirectExceptionHandler
{
  private static final Logger log = LoggerFactory.getLogger(ExtDirectExceptionHandler.class);

  /**
   * Exceptions in this list are logged at DEBUG level (not ERROR) to avoid stack trace noise.
   * These represent expected operational scenarios (e.g., permission denied, authentication failure).
   * <p>
   * Note: Support teams should enable DEBUG logging on this class
   * (org.globalart.astrum.extdirect.internal.ExtDirectExceptionHandler)
   * when troubleshooting authorization/authentication issues.
   */
  private static final List<Class<? extends RuntimeException>> SUPPRESSED_EXCEPTIONS = ListUtils.unmodifiableList(
      Arrays.asList(
          AuthorizationException.class, // Includes UnauthenticatedException and UnauthorizedException
          AuthenticationException.class,
          ValidationErrorsException.class));

  public Response handleException(final RegisteredMethod method, final Throwable e) {
    // debug logging for sanity (without stacktrace for suppressed exception)
    log.debug("Failed to invoke action method: {}, java-method: {}, exception message: {}",
        method.getFullName(), method.getFullJavaMethodName(), e.getMessage(),
        isSuppressedException(e) ? null : e);

    // handle validation message responses which have contents
    if (e instanceof ConstraintViolationException) {
      ConstraintViolationException cause = (ConstraintViolationException) e;
      Set<ConstraintViolation<?>> violations = cause.getConstraintViolations();
      if (violations != null && !violations.isEmpty()) {
        return invalid(cause);
      }
    }

    // exception logging for all non-suppressed exceptions
    if (!isSuppressedException(e)) {
      log.error("Failed to invoke action method: {}, java-method: {}",
          method.getFullName(), method.getFullJavaMethodName(), e);
    }

    String exceptionName = e.getClass().getName();
    if (e instanceof SQLException
        || exceptionName.contains("org.apache.ibatis")
        || exceptionName.contains("org.globalart.astrum.datastore")) {
      return error(new Exception("A database error occurred"));
    }

    if (e instanceof HttpHostConnectException || exceptionName.contains("com.globalart.insight.rm.rest.HttpException")) {
      return error(new Exception("Connection unsuccessful."));
    }

    return error(e);
  }

  private boolean isSuppressedException(final Throwable e) {
    return SUPPRESSED_EXCEPTIONS.stream().anyMatch(ex -> ex.isInstance(e) || ex.isInstance(e.getCause()));
  }
}
