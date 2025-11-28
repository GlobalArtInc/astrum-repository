
package org.globalart.astrum.validation.constraint;

import java.util.regex.Pattern;

import jakarta.validation.ConstraintValidatorContext;

import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.validation.ConstraintValidatorSupport;

/**
 * {@link Hostname} validator.
 *
 * @since 3.3
 */
public class HostnameValidator
    extends ConstraintValidatorSupport<Hostname, String>
{
  // See also http-headers-patterns.properties and Validator.js for other uses of this regex.
  private static final Pattern RFC_1123_HOST = Pattern.compile(
      "^(((([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])\\.){3}([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5]))|" +
          "(\\[(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}\\])|" +
          "(\\[((?:[0-9A-Fa-f]{1,4}(?::[0-9A-Fa-f]{1,4})*)?)::((?:[0-9A-Fa-f]{1,4}(?::[0-9A-Fa-f]{1,4})*)?)\\])|" +
          "(([a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9\\-]*[a-zA-Z0-9])\\.)*([A-Za-z0-9]|[A-Za-z0-9][A-Za-z0-9\\-]*[A-Za-z0-9]))(:([0-9]+))?$");

  @Override
  public boolean isValid(final String value, final ConstraintValidatorContext context) {
    if (Strings2.isBlank(value)) {
      return true;
    }

    return RFC_1123_HOST.matcher(value).matches();
  }
}
