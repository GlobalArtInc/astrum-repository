
package org.globalart.astrum.cleanup.internal.datastore.search.criteria;

import java.util.function.Predicate;
import java.util.regex.Pattern;
import java.util.regex.PatternSyntaxException;

import org.globalart.astrum.cleanup.datastore.search.criteria.AssetCleanupEvaluator;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.Asset;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.REGEX_KEY;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Creates an predicate which determines whether an asset's path matches the specified regular expression
 *
 * @since 3.38
 */
@Component
@Qualifier(REGEX_KEY)
public class RegexCleanupEvaluator
    implements AssetCleanupEvaluator
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  /*
   * Value is expected to be a regular expression which Java understands.
   */
  @Override
  public Predicate<Asset> getPredicate(final Repository repository, final String value) {
    try {
      Pattern matcher = Pattern.compile(value);
      return asset -> matcher.matcher(asset.path()).matches();
    }
    catch (PatternSyntaxException e) {
      log.error("Repository {} has cleanup policies specifying an invalid regular expression: '{}'",
          repository.getName(), value);

      throw new RuntimeException(
          String.format("Repository %s specifies an invalid regular expression.", repository.getName()), e);
    }
  }
}
