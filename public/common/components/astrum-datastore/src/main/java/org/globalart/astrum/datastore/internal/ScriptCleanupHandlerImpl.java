
package org.globalart.astrum.datastore.internal;

import org.globalart.astrum.common.script.ScriptCleanupHandler;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Stub DataStore implementation of {@link ScriptCleanupHandler}.
 *
 * @since 3.24
 */
@Component
@Qualifier("datastore")
public class ScriptCleanupHandlerImpl
    implements ScriptCleanupHandler
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Override
  public void cleanup(final String context) {
    log.debug("Currently does nothing");
  }
}
