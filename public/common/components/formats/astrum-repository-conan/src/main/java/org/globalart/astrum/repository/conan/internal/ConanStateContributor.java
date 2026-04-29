package org.globalart.astrum.repository.conan.internal;

import java.util.Map;

import org.globalart.astrum.rapture.StateContributor;

import org.springframework.stereotype.Component;

@Component
public class ConanStateContributor
    implements StateContributor
{
  private static final Map<String, Object> STATE = Map.of("conanV2Enabled", true);

  @Override
  public Map<String, Object> getState() {
    return STATE;
  }
}
