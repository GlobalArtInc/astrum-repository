
package org.globalart.astrum.coreui.internal.repository;

import java.util.Map;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.rapture.StateContributor;

import com.google.common.collect.ImmutableMap;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

@Component
public class RepositoryStateContributor
    implements StateContributor
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final Map<String, Object> state;

  @Autowired
  public RepositoryStateContributor(
      @Value("${astrum.react.repositories:false}") final Boolean featureFlag)
  {
    state = ImmutableMap.of("astrum.react.repositories", featureFlag);
  }

  @Override
  public Map<String, Object> getState() {
    return state;
  }
}
