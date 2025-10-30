
package org.globalart.astrum.common.template;

import com.google.common.base.Throwables;
import org.apache.commons.text.StringEscapeUtils;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Helper to deal with {@link Throwable} instances in a template.
 *
 * @since 3.0
 */
@TemplateAccessible
public class TemplateThrowableAdapter
{
  private final Throwable cause;

  public TemplateThrowableAdapter(final Throwable cause) {
    this.cause = checkNotNull(cause);
  }

  public Throwable getCause() {
    return cause;
  }

  public String getType() {
    return cause.getClass().getName();
  }

  public String getSimpleType() {
    return cause.getClass().getSimpleName();
  }

  public String getMessage() {
    return StringEscapeUtils.escapeHtml4(cause.getMessage());
  }

  public String getTrace() {
    return StringEscapeUtils.escapeHtml4(Throwables.getStackTraceAsString(cause));
  }

  public String toString() {
    return cause.toString();
  }
}
