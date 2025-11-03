
package org.globalart.astrum.mime;

import java.util.LinkedHashMap;
import java.util.Map;
import java.util.regex.Pattern;

import com.google.common.collect.Maps;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Ordered regular-expression {@link MimeRulesSource} implementation.
 *
 * @since 2.0
 */
public class RegexpMimeRulesSource
    implements MimeRulesSource
{
  private final LinkedHashMap<Pattern, MimeRule> rules = Maps.newLinkedHashMap();

  public void addRule(final String pattern, final String mimeType) {
    addRule(Pattern.compile(pattern), mimeType);
  }

  public void addRule(final Pattern pattern, final String mimeType) {
    rules.put(checkNotNull(pattern), new MimeRule(false, mimeType));
  }

  @Override
  public MimeRule getRuleForName(final String name) {
    for (Map.Entry<Pattern, MimeRule> entry : rules.entrySet()) {
      if (entry.getKey().matcher(name).matches()) {
        return entry.getValue();
      }
    }
    return null;
  }
}
