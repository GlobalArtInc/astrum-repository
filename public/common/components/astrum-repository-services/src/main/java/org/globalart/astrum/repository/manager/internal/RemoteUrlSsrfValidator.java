
package org.globalart.astrum.repository.manager.internal;

import java.net.URI;
import java.util.Map;

import javax.annotation.Nullable;
import jakarta.validation.ConstraintViolation;
import jakarta.validation.ValidationException;

import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.manager.ConfigurationValidator;
import org.globalart.astrum.validation.ConstraintViolationFactory;
import org.globalart.astrum.validation.ssrf.AntiSsrfService;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
public class RemoteUrlSsrfValidator
    implements ConfigurationValidator
{
  private static final String PROXY = "proxy";

  private static final String REMOTE_URL = "remoteUrl";

  private final ConstraintViolationFactory constraintViolationFactory;

  private final AntiSsrfService antiSsrfService;

  @Autowired
  public RemoteUrlSsrfValidator(
      final ConstraintViolationFactory constraintViolationFactory,
      final AntiSsrfService antiSsrfService)
  {
    this.constraintViolationFactory = checkNotNull(constraintViolationFactory);
    this.antiSsrfService = checkNotNull(antiSsrfService);
  }

  @Nullable
  @Override
  public ConstraintViolation<?> validate(final Configuration configuration) {
    Map<String, Map<String, Object>> attributes = configuration.getAttributes();
    if (attributes != null && attributes.containsKey(PROXY)) {
      Map<String, Object> proxyAttributes = attributes.get(PROXY);
      if (proxyAttributes != null && proxyAttributes.containsKey(REMOTE_URL)) {
        Object remoteUrlObj = proxyAttributes.get(REMOTE_URL);
        if (remoteUrlObj != null) {
          try {
            URI remoteUrl;
            if (remoteUrlObj instanceof String uri) {
              remoteUrl = new URI(uri);
            }
            else {
              return null;
            }

            String host = remoteUrl.getHost();
            if (host != null) {
              antiSsrfService.validateHostWithoutCache(host);
            }
          }
          catch (ValidationException e) {
            return constraintViolationFactory.createViolation(
                PROXY + "." + REMOTE_URL,
                "Proxy URL blocked: " + e.getMessage());
          }
          catch (Exception e) {
            return constraintViolationFactory.createViolation(
                PROXY + "." + REMOTE_URL,
                "Invalid proxy URL format");
          }
        }
      }
    }
    return null;
  }
}
