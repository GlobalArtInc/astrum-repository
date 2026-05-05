package org.globalart.astrum.repository.p2.internal;

import java.util.Set;

import org.globalart.astrum.repository.rest.api.AssetXODescriptor;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.p2.internal.util.P2PathUtils.PLUGIN_NAME;

@Component
@Qualifier(P2Format.NAME)
public class P2AssetXODescriptor
    implements AssetXODescriptor
{
  private static final Set<String> ATTRIBUTE_KEYS = Set.of(PLUGIN_NAME);

  @Override
  public Set<String> listExposedAttributeKeys() {
    return ATTRIBUTE_KEYS;
  }
}
