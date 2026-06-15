
package org.globalart.astrum.coreui.internal.http;

import java.util.Map;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.httpclient.HttpDefaultsCustomizer;
import org.globalart.astrum.rapture.StateContributor;

import com.google.common.collect.ImmutableMap;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

@Component
public class HttpStateContributor
    implements StateContributor
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final HttpDefaultsCustomizer customizer;

  private boolean featureFlag;

  private final String contextPath;

  @Autowired
  public HttpStateContributor(
      @Value("${astrum.react.httpSettings:true}") final Boolean featureFlag,
      @Value("${astrum-context-path:/}") final String contextPath,
      final HttpDefaultsCustomizer customizer)
  {
    this.customizer = customizer;
    this.featureFlag = featureFlag;
    this.contextPath = contextPath;
  }

  @Override
  public Map<String, Object> getState() {
    return ImmutableMap.of(
        "astrum.react.httpSettings", featureFlag,
        "requestTimeout", customizer.getRequestTimeout(),
        "retryCount", customizer.getRetryCount(),
        "astrum-context-path", contextPath);
  }
}
