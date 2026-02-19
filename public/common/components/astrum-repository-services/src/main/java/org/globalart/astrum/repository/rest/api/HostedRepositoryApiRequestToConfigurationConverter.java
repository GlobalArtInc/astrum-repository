
package org.globalart.astrum.repository.rest.api;

import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.rest.api.model.HostedRepositoryApiRequest;

import com.google.common.collect.Sets;

import static org.globalart.astrum.repository.config.ConfigurationConstants.BLOB_STORE_NAME;
import static org.globalart.astrum.repository.config.ConfigurationConstants.COMPONENT;
import static org.globalart.astrum.repository.config.ConfigurationConstants.PROPRIETARY_COMPONENTS;
import static org.globalart.astrum.repository.config.ConfigurationConstants.STORAGE;
import static org.globalart.astrum.repository.config.ConfigurationConstants.STRICT_CONTENT_TYPE_VALIDATION;
import static org.globalart.astrum.repository.config.ConfigurationConstants.WRITE_POLICY;
import static org.globalart.astrum.repository.manager.internal.BaseRepositoryManager.CLEANUP_ATTRIBUTES_KEY;
import static org.globalart.astrum.repository.manager.internal.BaseRepositoryManager.CLEANUP_NAME_KEY;
import org.springframework.stereotype.Component;

/**
 * @since 3.20
 */
@Component
public class HostedRepositoryApiRequestToConfigurationConverter<T extends HostedRepositoryApiRequest>
    extends AbstractRepositoryApiRequestToConfigurationConverter<T>
{
  @Override
  public Configuration convert(final T request) {
    Configuration configuration = super.convert(request);

    configuration.attributes(STORAGE).set(BLOB_STORE_NAME, request.getStorage().getBlobStoreName());
    configuration.attributes(STORAGE)
        .set(STRICT_CONTENT_TYPE_VALIDATION, request.getStorage().getStrictContentTypeValidation());
    configuration.attributes(STORAGE).set(WRITE_POLICY, request.getStorage().getWritePolicy());

    if (request.getComponent() != null) {
      configuration.attributes(COMPONENT)
          .set(PROPRIETARY_COMPONENTS, request.getComponent().getProprietaryComponents());
    }

    maybeAddDataStoreName(configuration);

    if (request.getCleanup() != null) {
      configuration.attributes(CLEANUP_ATTRIBUTES_KEY)
          .set(CLEANUP_NAME_KEY, Sets.newHashSet(request.getCleanup().getPolicyNames()));
    }
    return configuration;
  }
}
