
package org.globalart.astrum.internal.security.apikey;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.scheduling.Cancelable;
import org.globalart.astrum.scheduling.TaskSupport;
import org.globalart.astrum.security.authc.apikey.ApiKeyService;
import org.globalart.astrum.security.usertoken.event.UserTokenPurgedEvent;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * Purge orphaned API keys task.
 *
 * @since 3.0
 * @see ApiKeyService#purgeApiKeys()
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class PurgeApiKeysTask
    extends TaskSupport
    implements Cancelable
{
  private final ApiKeyInternalService apiKeyService;

  private final EventManager eventManager;

  @Autowired
  public PurgeApiKeysTask(final ApiKeyInternalService store, final EventManager eventManager) {
    this.apiKeyService = checkNotNull(store);
    this.eventManager = checkNotNull(eventManager);
  }

  @Override
  protected Void execute() throws Exception {
    int deleted = apiKeyService.purgeApiKeys();
    if (deleted > 0) {
      eventManager.post(new UserTokenPurgedEvent(deleted));
    }
    return null;
  }

  @Override
  public String getMessage() {
    return "Deleting orphaned API keys";
  }
}
