
package org.globalart.astrum.httpclient.config;

import java.util.Arrays;
import java.util.Collection;
import java.util.regex.Pattern;

import jakarta.validation.ConstraintValidatorContext;

import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.validation.ConstraintValidatorSupport;

/**
 * {@link NonProxyHosts} validator.
 *
 * @since 3.0
 */
public class NonProxyHostsValidator
    extends ConstraintValidatorSupport<NonProxyHosts, Collection<String>>
{
  /**
   * Adapter for validating array fields.
   */
  public static class ForArray
      extends ConstraintValidatorSupport<NonProxyHosts, String[]>
  {
    @Override
    public boolean isValid(final String[] values, final ConstraintValidatorContext context) {
      return values != null ? NonProxyHostsValidator.isValid(Arrays.asList(values)) : true;
    }
  }

  /**
   * Pattern checking for allowed characters, best we can do, as input may be:
   * <ul>
   * <li>hostname w/o or w/ wildcard (incomplete)</li>
   * <li>IPv4 address w/o or w/ wildcard (incomplete)</li>
   * <li>IPv6 non-compressed address w/o or w/ wildcard (incomplete)</li>
   * <li>IPv6 compressed address w/o or w/ wildcard (incomplete)</li>
   * </ul>
   * Due to the "incomplete" case (hostname may be missing domain, IPv4 might have one, two, three or four segments,
   * IPv6 might have three or more segments) the validation we perform here is basically just enforcing allowed
   * characters, and simply not treating these as hostname or IP address, but merely as an opaque pattern.
   * If wildcard present, wildcard in a nonProxyHost element may be only on it's beginning or end, nowhere else.
   */
  private static final Pattern CONTENT_PATTERN = Pattern
      .compile("\\*?[\\p{IsAlphabetic}|\\d|\\-|\\_|\\.|\\:|\\[|\\]]+\\*?");

  @Override
  public boolean isValid(final Collection<String> values, final ConstraintValidatorContext context) {
    return values != null ? isValid(values) : true;
  }

  static boolean isValid(Collection<String> values) {
    for (String value : values) {
      if (!isValid(value)) {
        return false;
      }
    }
    return true;
  }

  /**
   * Returns {@code true} if value is considered as valid nonProxyHosts expression. This is NOT validating the
   * single-string used to set system property (where expressions are delimited with "|")!
   * 
   * @since 3.1
   */
  public static boolean isValid(String value) {
    // A value should be a non-empty string optionally prefixed or suffixed with an asterisk
    // must be non-empty, non-blank
    if (Strings2.isBlank(value)) {
      return false;
    }
    // must not contain | separator (used to separate multiple values in system properties)
    if (value.indexOf('|') > -1) {
      return false;
    }
    // asterisk '*' can be only on beginning or end
    return CONTENT_PATTERN.matcher(value).matches();
  }
}
