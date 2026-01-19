
package org.globalart.astrum.extdirect.internal;

import java.util.concurrent.Callable;

import org.globalart.astrum.common.app.BaseUrlHolder;
import org.globalart.astrum.security.UserIdMdcHelper;

import com.google.common.base.Throwables;
import com.softwarementors.extjs.djn.servlet.ssm.SsmJsonRequestProcessorThread;
import org.apache.shiro.SecurityUtils;
import org.apache.shiro.subject.Subject;
import org.apache.shiro.subject.support.SubjectThreadState;

import static com.google.common.base.Preconditions.checkState;

/**
 * An {@link SsmJsonRequestProcessorThread} that is binds the thread to Shiro subject as well as setting user id in
 * MDC.
 *
 * @since 3.0
 */
public class ExtDirectJsonRequestProcessorThread
    extends SsmJsonRequestProcessorThread
{

  private final SubjectThreadState threadState;

  private final Callable<String> processRequest;

  public ExtDirectJsonRequestProcessorThread() {
    Subject subject = SecurityUtils.getSubject();
    checkState(subject != null, "Subject is not set");
    // create the thread state by this moment as this is created in the master (web container) thread
    threadState = new SubjectThreadState(subject);

    final String baseUrl = BaseUrlHolder.get();
    final String relativePath = BaseUrlHolder.getRelativePath();

    processRequest = () -> {
      threadState.bind();
      UserIdMdcHelper.set();
      try {
        // apply base-url from the original thread
        BaseUrlHolder.set(baseUrl, relativePath);

        return ExtDirectJsonRequestProcessorThread.super.processRequest();
      }
      finally {
        UserIdMdcHelper.unset();
        threadState.restore();
      }
    };
  }

  @Override
  public String processRequest() {
    try {
      return processRequest.call();
    }
    catch (Exception e) {
      Throwables.throwIfUnchecked(e);
      throw new RuntimeException(e);
    }
  }

}
