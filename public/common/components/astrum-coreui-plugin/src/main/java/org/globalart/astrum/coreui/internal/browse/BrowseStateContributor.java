
package org.globalart.astrum.coreui.internal.browse;

import java.util.Map;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.rapture.StateContributor;

import com.google.common.collect.ImmutableMap;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

@Component
public class BrowseStateContributor
    implements StateContributor
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final Map<String, Object> state;

  @Autowired
  public BrowseStateContributor(
      @Value("${astrum.react.browse:false}") final boolean featureFlag)
  {
    state = ImmutableMap.of("astrum.react.browse", featureFlag);
  }

  @Override
  public Map<String, Object> getState() {
    return state;
  }
}
