
package org.globalart.astrum.datastore.mybatis;

import java.util.function.Predicate;

import com.google.common.base.Splitter;
import com.google.common.collect.ImmutableSet;
import org.apache.ibatis.session.Configuration;

import static java.util.regex.Pattern.compile;
import static org.globalart.astrum.common.text.Strings2.isBlank;

/**
 * Filter sensitive attributes that require encryption at rest.
 *
 * By default this includes any attribute whose name contains 'password' or 'secret'. Additional attribute
 * names can be configured in {@code $install-dir/etc/fabric/config-store-mybatis.xml} as a comma-separated
 * list under the 'sensitiveAttributes' property, for example:
 *
 * <pre>
 * <property name="sensitiveAttributes" value="passcode,token"/>
 * </pre>
 *
 * @since 3.21
 */
final class SensitiveAttributes
{
  private static final String SENSITIVE_ATTRIBUTES_KEY = "sensitiveAttributes";

  private static final Predicate<String> SENSITIVE_FILTER = compile("(?i)(password|secret)").asPredicate();

  private SensitiveAttributes() {
    // static utility class
  }

  /**
   * Builds the filter that determines whether a JSON field is considered sensitive or not.
   */
  public static Predicate<String> buildSensitiveAttributeFilter(final Configuration config) {
    String additionalAttributes = config.getVariables().getProperty(SENSITIVE_ATTRIBUTES_KEY);

    if (isBlank(additionalAttributes)) {
      return SENSITIVE_FILTER;
    }

    Predicate<String> additionalFilter = ImmutableSet.copyOf(
        Splitter.on(',').trimResults().omitEmptyStrings().split(additionalAttributes))::contains;

    return SENSITIVE_FILTER.or(additionalFilter);
  }
}
