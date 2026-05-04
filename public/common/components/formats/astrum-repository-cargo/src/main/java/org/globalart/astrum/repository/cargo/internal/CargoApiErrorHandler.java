package org.globalart.astrum.repository.cargo.internal;

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
public class CargoApiErrorHandler
    implements Handler
{
  private static final Logger log = LoggerFactory.getLogger(CargoApiErrorHandler.class);

  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    try {
      return context.proceed();
    }
    catch (CargoException e) {
      log.debug("Cargo request {} {} failed: {}", context.getRequest().getAction(), context.getRequest().getPath(),
          e.getMessage());
      return CargoResponses.error(e.getStatus(), e.getMessage());
    }
    catch (RedeployDisabledException e) {
      return CargoResponses.error(409,
          "crate version already exists and repository " + context.getRepository().getName()
              + " does not allow redeployment");
    }
    catch (IllegalOperationException | InvalidContentException | IllegalArgumentException e) {
      log.debug("Cargo request {} {} rejected", context.getRequest().getAction(), context.getRequest().getPath(), e);
      return CargoResponses.error(400, e.getMessage());
    }
  }
}
