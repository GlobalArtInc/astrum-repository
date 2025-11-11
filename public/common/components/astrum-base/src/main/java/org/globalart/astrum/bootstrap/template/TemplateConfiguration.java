
package org.globalart.astrum.bootstrap.template;

import com.google.common.base.Joiner;
import com.google.common.base.Throwables;
import org.apache.velocity.app.VelocityEngine;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static org.apache.velocity.runtime.RuntimeConstants.PARSER_POOL_SIZE;
import static org.apache.velocity.runtime.RuntimeConstants.RESOURCE_LOADERS;
import static org.apache.velocity.runtime.RuntimeConstants.RESOURCE_LOADER_CACHE;
import static org.apache.velocity.runtime.RuntimeConstants.RESOURCE_LOADER_CHECK_INTERVAL;
import static org.apache.velocity.runtime.RuntimeConstants.RESOURCE_LOADER_CLASS;
import static org.apache.velocity.runtime.RuntimeConstants.RUNTIME_REFERENCES_STRICT;
import static org.apache.velocity.runtime.RuntimeConstants.VM_LIBRARY;
import static org.apache.velocity.runtime.RuntimeConstants.VM_PERM_INLINE_LOCAL;

@Configuration
public class TemplateConfiguration
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Bean
  public VelocityEngine velocityEngine(
      @Value("${astrum.velocity." + PARSER_POOL_SIZE + ":20}") final int velocityParserPoolSize)
  {
    VelocityEngine engine = new VelocityEngine();
    Joiner j = Joiner.on('.');

    // to avoid "unable to find resource 'VM_global_library.vm' in any resource loader."
    engine.setProperty(VM_LIBRARY, "");

    // to use classpath loader
    engine.setProperty(RESOURCE_LOADERS, RESOURCE_LOADER_CLASS);
    engine.setProperty(j.join(RESOURCE_LOADERS, RESOURCE_LOADER_CLASS, RESOURCE_LOADER_CLASS),
        "org.apache.velocity.runtime.resource.loader.ClasspathResourceLoader");

    // to set caching ON
    engine.setProperty(j.join(RESOURCE_LOADERS, RESOURCE_LOADER_CLASS, RESOURCE_LOADER_CACHE), "true");

    // to never check for template modification (they are JARred)
    engine.setProperty(j.join(RESOURCE_LOADERS, RESOURCE_LOADER_CLASS, RESOURCE_LOADER_CHECK_INTERVAL), "0");

    // to set strict mode OFF
    engine.setProperty(RUNTIME_REFERENCES_STRICT, "false");

    // to force templates having inline local scope for VM definitions
    engine.setProperty(VM_PERM_INLINE_LOCAL, "true");

    engine.setProperty(PARSER_POOL_SIZE, velocityParserPoolSize);

    log.debug("Initializing: {}", engine);
    try {
      engine.init();
    }
    catch (Exception e) {
      Throwables.throwIfUnchecked(e);
      throw new RuntimeException(e);
    }

    return engine;
  }

}
