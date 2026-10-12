package org.globalart.astrum.repository.rubygems.internal;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.IllegalOperationException;
import org.globalart.astrum.repository.InvalidContentException;
import org.globalart.astrum.repository.RedeployDisabledException;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Component;

@Component
public class RubygemsApiErrorHandler
    implements Handler
{
  private static final Logger log = LoggerFactory.getLogger(RubygemsApiErrorHandler.class);

  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    try {
      return context.proceed();
    }
    catch (RubygemsException e) {
      log.debug("RubyGems request {} {} failed: {}", context.getRequest().getAction(), context.getRequest().getPath(),
          e.getMessage());
      return RubygemsResponses.text(e.getStatus(), e.getMessage());
    }
    catch (RedeployDisabledException e) {
      return RubygemsResponses.text(409, "Repushing of gem versions is not allowed: repository "
          + context.getRepository().getName() + " does not allow redeployment");
    }
    catch (IllegalOperationException | InvalidContentException | IllegalArgumentException e) {
      log.debug("RubyGems request {} {} rejected", context.getRequest().getAction(), context.getRequest().getPath(), e);
      return RubygemsResponses.text(422, e.getMessage());
    }
  }
}
