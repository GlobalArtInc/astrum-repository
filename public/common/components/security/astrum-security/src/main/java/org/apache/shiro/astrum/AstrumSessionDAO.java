
package org.apache.shiro.astrum;

import java.io.Serializable;

import org.apache.shiro.session.Session;
import org.apache.shiro.session.mgt.eis.EnterpriseCacheSessionDAO;
import org.apache.shiro.session.mgt.eis.SessionDAO;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Custom {@link SessionDAO}.
 *
 * @since 3.0
 */
public class AstrumSessionDAO
    extends EnterpriseCacheSessionDAO
{
  private static final Logger log = LoggerFactory.getLogger(AstrumSessionDAO.class);

  @Override
  protected Serializable doCreate(final Session session) {
    Serializable id = super.doCreate(session);
    log.trace("Created session-id: {} for session: {}", id, session);
    return id;
  }
}
