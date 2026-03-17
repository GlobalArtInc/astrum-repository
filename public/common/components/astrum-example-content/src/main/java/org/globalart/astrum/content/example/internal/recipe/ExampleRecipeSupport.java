
package org.globalart.astrum.content.example.internal.recipe;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.inject.Provider;

import org.globalart.astrum.content.example.ExampleContentFacet;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.RecipeSupport;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.http.PartialFetchHandler;
import org.globalart.astrum.repository.security.SecurityHandler;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.handlers.ConditionalRequestHandler;
import org.globalart.astrum.repository.view.handlers.ContentHeadersHandler;
import org.globalart.astrum.repository.view.handlers.ExceptionHandler;
import org.globalart.astrum.repository.view.handlers.HandlerContributor;
import org.globalart.astrum.repository.view.handlers.IndexHtmlForwardHandler;
import org.globalart.astrum.repository.view.handlers.LastDownloadedHandler;
import org.globalart.astrum.repository.view.handlers.TimingHandler;

import static com.google.common.base.Preconditions.checkNotNull;

public abstract class ExampleRecipeSupport
    extends RecipeSupport
{
  protected Provider<ExampleSecurityFacet> securityFacet;

  protected Provider<ConfigurableViewFacet> viewFacet;

  protected Provider<ExampleContentFacet> contentFacet;

  protected ExceptionHandler exceptionHandler;

  protected TimingHandler timingHandler;

  protected IndexHtmlForwardHandler indexHtmlForwardHandler;

  protected SecurityHandler securityHandler;

  protected PartialFetchHandler partialFetchHandler;

  protected ExampleContentHandler contentHandler;

  protected ConditionalRequestHandler conditionalRequestHandler;

  protected ContentHeadersHandler contentHeadersHandler;

  protected LastDownloadedHandler lastDownloadedHandler;

  protected HandlerContributor handlerContributor;

  protected ExampleRecipeSupport(
      final Type type,
      final Format format)
  {
    super(type, format);
  }

  @Autowired
  public final void setDependencies(
      final Provider<ExampleSecurityFacet> securityFacet,
      final Provider<ConfigurableViewFacet> viewFacet,
      final Provider<ExampleContentFacet> contentFacet,
      final ExceptionHandler exceptionHandler,
      final TimingHandler timingHandler,
      final IndexHtmlForwardHandler indexHtmlForwardHandler,
      final SecurityHandler securityHandler,
      final PartialFetchHandler partialFetchHandler,
      final ExampleContentHandler contentHandler,
      final ConditionalRequestHandler conditionalRequestHandler,
      final ContentHeadersHandler contentHeadersHandler,
      final LastDownloadedHandler lastDownloadedHandler,
      final HandlerContributor handlerContributor)
  {
    this.securityFacet = checkNotNull(securityFacet);
    this.viewFacet = checkNotNull(viewFacet);
    this.contentFacet = checkNotNull(contentFacet);
    this.exceptionHandler = checkNotNull(exceptionHandler);
    this.timingHandler = checkNotNull(timingHandler);
    this.indexHtmlForwardHandler = checkNotNull(indexHtmlForwardHandler);
    this.securityHandler = checkNotNull(securityHandler);
    this.partialFetchHandler = checkNotNull(partialFetchHandler);
    this.contentHandler = checkNotNull(contentHandler);
    this.conditionalRequestHandler = checkNotNull(conditionalRequestHandler);
    this.contentHeadersHandler = checkNotNull(contentHeadersHandler);
    this.lastDownloadedHandler = checkNotNull(lastDownloadedHandler);
    this.handlerContributor = checkNotNull(handlerContributor);
  }
}
