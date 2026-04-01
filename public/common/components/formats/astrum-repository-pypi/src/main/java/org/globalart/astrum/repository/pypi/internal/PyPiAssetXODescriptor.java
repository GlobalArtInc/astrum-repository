
package org.globalart.astrum.repository.pypi.internal;

import java.util.Set;

import org.globalart.astrum.repository.rest.api.AssetXODescriptor;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.pypi.internal.PyPiAttributes.P_NAME;
import static org.globalart.astrum.repository.pypi.internal.PyPiAttributes.P_PLATFORM;
import static org.globalart.astrum.repository.pypi.internal.PyPiAttributes.P_REQUIRES_PYTHON;
import static org.globalart.astrum.repository.pypi.internal.PyPiAttributes.P_SUMMARY;
import static org.globalart.astrum.repository.pypi.internal.PyPiAttributes.P_VERSION;

@Component
@Qualifier(PyPiFormat.NAME)
public class PyPiAssetXODescriptor
    implements AssetXODescriptor
{
  private static final Set<String> ATTRIBUTE_KEYS = Set.of(P_NAME, P_VERSION, P_PLATFORM, P_SUMMARY, P_REQUIRES_PYTHON);

  @Override
  public Set<String> listExposedAttributeKeys() {
    return ATTRIBUTE_KEYS;
  }
}
