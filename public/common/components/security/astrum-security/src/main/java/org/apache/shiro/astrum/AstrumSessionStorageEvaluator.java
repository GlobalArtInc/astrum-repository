
package org.apache.shiro.astrum;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.security.anonymous.AnonymousHelper;

import org.apache.shiro.mgt.SessionStorageEvaluator;
import org.apache.shiro.subject.Subject;
import org.apache.shiro.web.mgt.DefaultWebSessionStorageEvaluator;
import org.springframework.beans.factory.annotation.Value;

/**
 * Custom {@link SessionStorageEvaluator}.
 *
 * @since 3.0
 */
public class AstrumSessionStorageEvaluator
    extends DefaultWebSessionStorageEvaluator
{
  @Autowired
  @Value("${astrum.session.enabled:true}")
  private boolean sessionsEnabled;

  /**
   * Disable storage for anonymous subject.
   */
  @Override
  public boolean isSessionStorageEnabled(final Subject subject) {
    if (sessionsEnabled) {
      return !AnonymousHelper.isAnonymous(subject) && super.isSessionStorageEnabled(subject);
    }
    return false;
  }
}
