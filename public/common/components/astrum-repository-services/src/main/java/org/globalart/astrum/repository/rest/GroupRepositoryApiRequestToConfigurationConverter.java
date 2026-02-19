
package org.globalart.astrum.repository.rest;

import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.rest.api.AbstractRepositoryApiRequestToConfigurationConverter;
import org.globalart.astrum.repository.rest.api.model.GroupAttributes;
import org.globalart.astrum.repository.rest.api.model.GroupDeployAttributes;
import org.globalart.astrum.repository.rest.api.model.GroupRepositoryApiRequest;

import static org.globalart.astrum.repository.config.ConfigurationConstants.BLOB_STORE_NAME;
import static org.globalart.astrum.repository.config.ConfigurationConstants.GROUP_WRITE_MEMBER;
import static org.globalart.astrum.repository.config.ConfigurationConstants.STORAGE;
import static org.globalart.astrum.repository.config.ConfigurationConstants.STRICT_CONTENT_TYPE_VALIDATION;
import org.springframework.stereotype.Component;

/**
 * @since 3.20
 */
@Component
public class GroupRepositoryApiRequestToConfigurationConverter<T extends GroupRepositoryApiRequest>
    extends AbstractRepositoryApiRequestToConfigurationConverter<T>
{
  public Configuration convert(final T request) {
    Configuration configuration = super.convert(request);

    configuration.attributes(STORAGE).set(BLOB_STORE_NAME, request.getStorage().getBlobStoreName());
    configuration.attributes(STORAGE)
        .set(STRICT_CONTENT_TYPE_VALIDATION, request.getStorage().getStrictContentTypeValidation());
    maybeAddDataStoreName(configuration);

    configuration.attributes("group").set("memberNames", request.getGroup().getMemberNames());

    GroupAttributes group = request.getGroup();
    if (group instanceof GroupDeployAttributes) {
      GroupDeployAttributes groupDeployAttributes = (GroupDeployAttributes) group;
      String writableMember = groupDeployAttributes.getWritableMember();
      if (writableMember != null && !writableMember.isEmpty()) {
        configuration.attributes("group").set(GROUP_WRITE_MEMBER, writableMember);
      }
    }
    return configuration;
  }
}
