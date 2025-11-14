
package org.globalart.astrum.internal.web;

import java.io.IOException;

import jakarta.servlet.Filter;
import jakarta.servlet.FilterChain;
import jakarta.servlet.ServletException;
import jakarta.servlet.ServletRequest;
import jakarta.servlet.ServletResponse;
import jakarta.servlet.annotation.WebFilter;
import jakarta.servlet.http.HttpServletRequest;

import org.globalart.astrum.common.app.WebFilterPriority;

import org.springframework.beans.factory.FactoryBean;
import org.springframework.core.annotation.Order;
import org.springframework.stereotype.Component;

/**
 * This class provides the HttpServletRequest for injection on the current thread as we are not using a Spring managed
 * Jetty
 */
@WebFilter("/*")
@Order(WebFilterPriority.SERVLET_FILTER)
@Component
public class HttpRequestProvider
    implements FactoryBean<HttpServletRequest>, Filter
{
  private final ThreadLocal<HttpServletRequest> currentRequest = new ThreadLocal<>();

  @Override
  public void doFilter(
      final ServletRequest request,
      final ServletResponse response,
      final FilterChain chain) throws IOException, ServletException
  {
    currentRequest.set((HttpServletRequest) request);
    try {
      chain.doFilter(request, response);
    }
    finally {
      currentRequest.set(null);
    }
  }

  @Override
  public HttpServletRequest getObject() throws Exception {
    return currentRequest.get();
  }

  @Override
  public Class<?> getObjectType() {
    return HttpServletRequest.class;
  }

  @Override
  public boolean isSingleton() {
    return false;
  }
}
