
package org.globalart.astrum.repository.httpbridge.internal;

import java.io.IOException;
import jakarta.servlet.Filter;
import jakarta.servlet.FilterChain;
import jakarta.servlet.ServletException;
import jakarta.servlet.ServletRequest;
import jakarta.servlet.ServletResponse;
import jakarta.servlet.annotation.WebFilter;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;

import org.globalart.astrum.capability.CapabilityEvent;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.repository.httpbridge.legacy.LegacyUrlCapabilityDescriptor;
import org.globalart.astrum.repository.httpbridge.legacy.LegacyUrlEnabledHelper;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.stereotype.Component;

import static jakarta.servlet.http.HttpServletResponse.SC_NOT_FOUND;

@WebFilter(urlPatterns = {"/content/groups/*", "/content/repositories/*", "/content/sites/*", "/service/local/*"})
@Component
public class LegacyViewFilter
    implements EventAware, Filter
{
  private final LegacyUrlEnabledHelper legacyUrlEnabledHelper;

  private volatile boolean isEnabled;

  public LegacyViewFilter(final LegacyUrlEnabledHelper legacyUrlEnabledHelper) {
    this.legacyUrlEnabledHelper = legacyUrlEnabledHelper;
    this.isEnabled = legacyUrlEnabledHelper.isEnabled();
  }

  @Override
  public void doFilter(
      final ServletRequest req,
      final ServletResponse resp,
      final FilterChain chain) throws IOException, ServletException
  {
    final HttpServletRequest request = (HttpServletRequest) req;
    final HttpServletResponse response = (HttpServletResponse) resp;

    if (!isEnabled) {
      response.sendError(SC_NOT_FOUND);
      return;
    }

    chain.doFilter(request, response);
  }

  @AllowConcurrentEvents
  @Subscribe
  public void handle(final CapabilityEvent event) {
    if (event.getReference().context().descriptor().type().equals(LegacyUrlCapabilityDescriptor.TYPE)) {
      toggleLegacyHttpBridgeModule();
    }
  }

  private synchronized void toggleLegacyHttpBridgeModule() {
    isEnabled = legacyUrlEnabledHelper.isEnabled();
  }
}
