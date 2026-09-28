
package com.globalart.astrum.ssl.plugin.validator;

import java.security.cert.CertificateException;

import jakarta.validation.ConstraintValidatorContext;

import org.globalart.astrum.ssl.CertificateUtil;
import org.globalart.astrum.validation.ConstraintValidatorSupport;

/**
 * {@link PemCertificate} validator.
 *
 * @since 3.0
 */
public class PemCertificateValidator
    extends ConstraintValidatorSupport<PemCertificate, String>
{
  @Override
  public boolean isValid(final String value, final ConstraintValidatorContext context) {
    try {
      CertificateUtil.decodePEMFormattedCertificate(value);
      return true;
    }
    catch (CertificateException e) {
      return false;
    }
  }
}
