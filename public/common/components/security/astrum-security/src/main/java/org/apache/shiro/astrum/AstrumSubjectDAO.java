
package org.apache.shiro.astrum;

import org.apache.shiro.mgt.DefaultSubjectDAO;
import org.apache.shiro.mgt.SubjectDAO;
import org.apache.shiro.subject.Subject;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Custom {@link SubjectDAO}.
 *
 * @since 3.0
 */
public class AstrumSubjectDAO
    extends DefaultSubjectDAO
{
  private static final Logger log = LoggerFactory.getLogger(AstrumSubjectDAO.class);

  @Override
  public Subject save(final Subject subject) {
    log.trace("Saving: {}", subject);
    return super.save(subject);
  }
}
