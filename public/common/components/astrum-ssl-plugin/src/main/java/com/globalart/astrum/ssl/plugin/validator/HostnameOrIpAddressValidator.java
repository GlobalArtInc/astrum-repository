
package com.globalart.astrum.ssl.plugin.validator;

import jakarta.validation.ConstraintValidatorContext;

import org.globalart.astrum.validation.ConstraintValidatorSupport;

import com.google.common.net.InetAddresses;
import com.google.common.net.InternetDomainName;

/**
 * Hostname or IP address validator.
 *
 * @since 3.36
 */
public class HostnameOrIpAddressValidator
    extends ConstraintValidatorSupport<HostnameOrIpAddress, String>
{
  @Override
  public boolean isValid(final String value, final ConstraintValidatorContext context) {
    return InternetDomainName.isValid(value) || InetAddresses.isInetAddress(value);
  }
}
