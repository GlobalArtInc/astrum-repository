
package org.globalart.astrum.coreui.internal.upload;

import java.util.Map;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.rapture.StateContributor;

import com.google.common.collect.ImmutableMap;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

@Component
public class UploadStateContributor
    implements StateContributor
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final Map<String, Object> state;

  @Autowired
  public UploadStateContributor(
      @Value("${astrum.react.upload:true}") final boolean featureFlag)
  {
    state = ImmutableMap.of("astrum.react.upload", featureFlag);
  }

  @Override
  public Map<String, Object> getState() {
    return state;
  }
}
