
package org.globalart.astrum.validation.constraint;

import java.net.URI;
import java.net.URISyntaxException;

import jakarta.validation.ConstraintValidatorContext;

import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.validation.ConstraintValidatorSupport;

/**
 * Validates a url.
 *
 * @since 3.13
 */
public class UrlStringValidator
    extends ConstraintValidatorSupport<UrlString, String>
{
  private static final UrlValidator urlValidator = new UrlValidator();

  @Override
  public boolean isValid(final String url, final ConstraintValidatorContext constraintValidatorContext) {
    if (Strings2.isBlank(url)) {
      return true;
    }

    try {
      URI uri = new URI(url);
      return urlValidator.isValid(uri, constraintValidatorContext);
    }
    catch (URISyntaxException ignored) {
      return false;
    }
  }
}
