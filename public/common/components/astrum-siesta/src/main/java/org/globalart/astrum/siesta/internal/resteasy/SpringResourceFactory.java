
package org.globalart.astrum.siesta.internal.resteasy;

import org.jboss.resteasy.spi.HttpRequest;
import org.jboss.resteasy.spi.HttpResponse;
import org.jboss.resteasy.spi.PropertyInjector;
import org.jboss.resteasy.spi.ResourceFactory;
import org.jboss.resteasy.spi.ResteasyProviderFactory;
import org.springframework.context.ApplicationContext;

import static com.google.common.base.Preconditions.checkNotNull;

public class SpringResourceFactory
    implements ResourceFactory
{
  private final ApplicationContext context;

  private final String beanName;

  private PropertyInjector propertyInjector;

  public SpringResourceFactory(final ApplicationContext context, final String beanName) {
    this.context = checkNotNull(context);
    this.beanName = checkNotNull(beanName);
  }

  @Override
  public Class<?> getScannableClass() {
    return context.getType(beanName);
  }

  @Override
  public void registered(final ResteasyProviderFactory factory) {
    checkNotNull(factory);
    propertyInjector = factory.getInjectorFactory().createPropertyInjector(getScannableClass(), factory);
  }

  @Override
  public Object createResource(
      final HttpRequest request,
      final HttpResponse response,
      final ResteasyProviderFactory factory)
  {
    Object component = context.getBean(beanName);
    // ASTRUM-46395: RESTEasy 7 PropertyInjector#inject signature added a boolean unwrapAsync
    // tail argument and returns CompletionStage<Void>. We block on the result by passing
    // unwrapAsync=true so this code path stays synchronous (matching the previous behavior).
    propertyInjector.inject(request, response, component, true);
    return component;
  }

  @Override
  public void requestFinished(final HttpRequest request, final HttpResponse response, final Object resource) {
    // ignore
  }

  @Override
  public void unregistered() {
    // ignore
  }
}
