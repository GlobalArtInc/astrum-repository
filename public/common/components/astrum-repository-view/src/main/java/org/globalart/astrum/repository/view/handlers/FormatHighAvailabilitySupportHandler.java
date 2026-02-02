
package org.globalart.astrum.repository.view.handlers;

import javax.annotation.Nonnull;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.repository.HighAvailabilitySupportChecker;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;

import static java.lang.String.format;
import static org.globalart.astrum.repository.http.HttpResponses.badRequest;
import org.springframework.stereotype.Component;

/**
 * Returns the message for client in case format doesn't support High Availability infrastructure.
 *
 * @since 3.17
 */
@Component
public class FormatHighAvailabilitySupportHandler
    implements Handler
{
  private HighAvailabilitySupportChecker highAvailabilitySupportChecker;

  @Autowired
  public FormatHighAvailabilitySupportHandler(final HighAvailabilitySupportChecker highAvailabilitySupportChecker) {
    this.highAvailabilitySupportChecker = highAvailabilitySupportChecker;
  }

  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    String format = context.getRepository().getFormat().getValue();
    if (!highAvailabilitySupportChecker.isSupported(format)) {
      return badRequest(format("%s is not supported in clustered mode.", format));
    }

    return context.proceed();
  }
}
