
package org.globalart.astrum.validation.constraint;

import java.net.URI;

import org.globalart.astrum.validation.ConstraintValidatorSupport;
import org.globalart.astrum.validation.ssrf.AntiSsrfService;

import jakarta.validation.ConstraintValidatorContext;
import jakarta.validation.ValidationException;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.springframework.beans.factory.config.ConfigurableBeanFactory.SCOPE_PROTOTYPE;

/**
 * Validates that a URL follows configuration for accessing private/local network address.
 */
@Component
@Scope(SCOPE_PROTOTYPE)
public class SsrfSafeUrlValidator
    extends ConstraintValidatorSupport<SsrfSafeUrl, URI>
{
  private final AntiSsrfService antiSsrfService;

  @Autowired
  public SsrfSafeUrlValidator(final AntiSsrfService antiSsrfService) {
    this.antiSsrfService = checkNotNull(antiSsrfService);
  }

  @Override
  public boolean isValid(final URI url, final ConstraintValidatorContext context) {
    if (url == null) {
      return true;
    }

    String host = url.getHost();
    if (host == null || host.isBlank()) {
      return true;
    }

    try {
      antiSsrfService.validateHostWithoutCache(host);
      return true;
    }
    catch (ValidationException e) {
      context.disableDefaultConstraintViolation();
      context.buildConstraintViolationWithTemplate(e.getMessage())
          .addConstraintViolation();
      return false;
    }
  }
}
