
package org.globalart.astrum.repository.npm.internal;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import java.util.Set;
import java.util.stream.Collectors;
import java.util.stream.Stream;


import org.globalart.astrum.repository.rest.api.AssetXODescriptor;

import static org.globalart.astrum.repository.npm.internal.NpmAttributes.P_NAME;
import static org.globalart.astrum.repository.npm.internal.NpmAttributes.P_VERSION;

/**
 * @since 3.29
 */
@Component
@Qualifier(NpmFormat.NAME)
public class NpmAssetXODescriptor
    implements AssetXODescriptor
{
  private static final Set<String> attributeKeys =
      Stream.of(P_NAME, P_VERSION).collect(Collectors.toSet());

  @Override
  public Set<String> listExposedAttributeKeys() {
    return attributeKeys;
  }
}
