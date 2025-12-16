
package org.apache.shiro.astrum;

import java.util.Collections;
import java.util.Map;

import org.apache.shiro.session.Session;
import org.apache.shiro.session.mgt.SessionContext;
import org.apache.shiro.session.mgt.SessionFactory;
import org.apache.shiro.session.mgt.SimpleSession;
import org.apache.shiro.session.mgt.SimpleSessionFactory;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Custom {@link SessionFactory}.
 *
 * @since 3.0
 */
public class AstrumSessionFactory
    extends SimpleSessionFactory
{
  private static final Logger log = LoggerFactory.getLogger(AstrumSessionFactory.class);

  /**
   * Ensure {@link SimpleSessionImpl} is used and provides logging.
   */
  @Override
  public Session createSession(final SessionContext initData) {
    log.trace("Creating session w/init-data: {}", initData);

    // duplicated from SimpleSessionFactory, retaining class-hierarchy for sanity
    if (initData != null) {
      String host = initData.getHost();
      if (host != null) {
        return new SimpleSessionImpl(host);
      }
    }
    return new SimpleSessionImpl();
  }

  /**
   * Customized session impl to apply synchronized-treatment to attributes map.
   */
  private static class SimpleSessionImpl
      extends SimpleSession
  {
    public SimpleSessionImpl() {
      super();
    }

    public SimpleSessionImpl(final String host) {
      super(host);
    }

    /**
     * Work around bug in Shiro which uses a non-synchronized map to back attributes.
     *
     * This appears to only be called by {@link SimpleSession#getAttributesLazy()}.
     */
    @Override
    public void setAttributes(final Map<Object, Object> attributes) {
      super.setAttributes(attributes != null ? Collections.synchronizedMap(attributes) : null);
    }
  }
}
