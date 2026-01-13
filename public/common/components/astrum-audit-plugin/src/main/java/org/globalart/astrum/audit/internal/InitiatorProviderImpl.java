
package org.globalart.astrum.audit.internal;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.audit.InitiatorProvider;
import org.globalart.astrum.security.ClientInfo;
import org.globalart.astrum.security.ClientInfoProvider;
import org.globalart.astrum.security.UserIdHelper;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Default {@link InitiatorProvider} implementation.
 *
 * @since 3.1
 */
@Component
public class InitiatorProviderImpl
    implements InitiatorProvider
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final ClientInfoProvider clientInfoProvider;

  @Autowired
  public InitiatorProviderImpl(final ClientInfoProvider clientInfoProvider) {
    this.clientInfoProvider = clientInfoProvider;
  }

  /**
   * When {@link ClientInfo} is available, returns {@code user-id/remote-ip}, else returns {@code user-id}.
   */
  @Override
  public String get() {
    ClientInfo clientInfo = clientInfoProvider.getCurrentThreadClientInfo();
    if (clientInfo != null) {
      return new StringBuilder()
          .append(clientInfo.getUserid())
          .append('/')
          .append(clientInfo.getRemoteIP())
          .toString();
    }
    else {
      return UserIdHelper.get();
    }
  }
}
