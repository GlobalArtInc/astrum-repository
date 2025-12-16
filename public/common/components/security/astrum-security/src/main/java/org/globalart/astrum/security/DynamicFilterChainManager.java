
package org.globalart.astrum.security;

import java.util.List;
import java.util.Optional;

import jakarta.servlet.Filter;
import jakarta.servlet.annotation.WebFilter;

import org.slf4j.LoggerFactory;
import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.common.text.Strings2;

import org.springframework.beans.factory.annotation.Autowired;
import org.apache.shiro.web.filter.mgt.DefaultFilterChainManager;
import org.apache.shiro.web.filter.mgt.FilterChainManager;
import org.slf4j.Logger;
import org.springframework.context.event.ContextRefreshedEvent;
import org.springframework.context.event.EventListener;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Dynamic {@link FilterChainManager} that reacts to {@link Filter}s and {@link FilterChain}s as they come and go.
 *
 * @since 3.0
 */
@Component
class DynamicFilterChainManager
    extends DefaultFilterChainManager
{
  private static final Logger log = LoggerFactory.getLogger(DynamicFilterChainManager.class);

  private final List<FilterChain> filterChains;

  private volatile boolean refreshChains;

  @Autowired
  public DynamicFilterChainManager(final List<FilterChain> filterChains) {
    this.filterChains = checkNotNull(filterChains);
  }

  @EventListener
  public void on(final ContextRefreshedEvent event) {
    // install the watchers for dynamic components contributed by other bundles
    event.getApplicationContext()
        .getBeansOfType(Filter.class)
        .values()
        .stream()
        .sorted(QualifierUtil::compareByOrder)
        .forEach(this::addFilter);

    this.refreshChains = true;
  }

  private void addFilter(final Filter filter) {
    String name = Optional.ofNullable(filter.getClass().getAnnotation(WebFilter.class))
        .map(WebFilter::filterName)
        .filter(Strings2::notBlank)
        .orElseGet(filter.getClass()::getName);

    // TODO maybe we shouldn't configure filters in JettyServer

    addFilter(name, filter, false);
  }

  @Override
  public boolean hasChains() {
    refreshChains();

    return super.hasChains();
  }

  /**
   * Regenerates the cached chain data based on the latest list of {@link FilterChain}s.
   */
  private void refreshChains() {
    if (refreshChains) { // only refresh once for the first request after any change
      synchronized (this) {
        if (refreshChains) {
          getChainNames().clear(); // completely replace old chains with latest list

          filterChains.stream()
              // Simple heuristic, longer URL patterns are more selective so we prioritize them
              .sorted((f1, f2) -> f2.getPathPattern().length() - f1.getPathPattern().length())
              .forEach(filterChain -> {
                try {
                  createChain(filterChain.getPathPattern(), filterChain.getFilterExpression());
                }
                catch (IllegalArgumentException e) {
                  log.warn("Problem registering: {}", filterChain, e);
                }
              });

          refreshChains = false;
        }
      }
    }
  }
}
