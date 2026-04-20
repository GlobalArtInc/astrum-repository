
package org.globalart.astrum.repository.composer.internal.proxy;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.composer.internal.ComposerJsonProcessor;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.http.HttpStatus;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Handler that rewrites the content of responses containing Composer provider JSON files so that they point to the
 * proxy repository rather than the repository being proxied.
 */
@Component
public class ComposerProviderHandler
    implements Handler
{
  public static final String DO_NOT_REWRITE = "ComposerProviderHandler.doNotRewrite";

  private final ComposerJsonProcessor composerJsonProcessor;

  @Autowired
  public ComposerProviderHandler(final ComposerJsonProcessor composerJsonProcessor) {
    this.composerJsonProcessor = checkNotNull(composerJsonProcessor);
  }

  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    Response response = context.proceed();
    if (!Boolean.parseBoolean(context.getRequest().getAttributes().get(DO_NOT_REWRITE, String.class))) {
      if (response.getStatus().getCode() == HttpStatus.OK && response.getPayload() != null) {
        response = HttpResponses
            .ok(composerJsonProcessor.rewriteProviderJson(context.getRepository(), response.getPayload()));
      }
    }
    return response;
  }
}
