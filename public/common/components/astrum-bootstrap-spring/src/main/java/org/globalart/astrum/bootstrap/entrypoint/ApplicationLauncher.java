
package org.globalart.astrum.bootstrap.entrypoint;

import java.util.Map;

import org.globalart.astrum.bootstrap.entrypoint.configuration.AstrumProperties;
import org.globalart.astrum.bootstrap.entrypoint.edition.AstrumEdition;
import org.globalart.astrum.bootstrap.entrypoint.edition.AstrumEditionSelector;

import jakarta.annotation.PostConstruct;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.slf4j.bridge.SLF4JBridgeHandler;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.context.ConfigurableApplicationContext;
import org.springframework.context.event.ContextRefreshedEvent;
import org.springframework.context.event.EventListener;
import org.springframework.core.convert.support.ConfigurableConversionService;
import org.springframework.core.env.MapPropertySource;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
public class ApplicationLauncher
{
  public static final String SYSTEM_USERID = "*SYSTEM";

  private static final Logger LOG = LoggerFactory.getLogger(ApplicationLauncher.class);

  private final AstrumEditionSelector astrumEditionSelector;

  private final ConfigurableApplicationContext context;

  private final SpringComponentScan springComponentScan;

  private final AstrumProperties astrumProperties;

  @Autowired
  public ApplicationLauncher(
      final AstrumEditionSelector astrumEditionSelector,
      final ConfigurableApplicationContext context,
      final SpringComponentScan springComponentScan,
      final AstrumProperties astrumProperties)

  {
    this.astrumEditionSelector = checkNotNull(astrumEditionSelector);
    this.context = checkNotNull(context);
    this.springComponentScan = checkNotNull(springComponentScan);
    this.astrumProperties = checkNotNull(astrumProperties);
  }

  @PostConstruct
  void initialize() {
    SLF4JBridgeHandler.removeHandlersForRootLogger();
    SLF4JBridgeHandler.install();
    AstrumEdition astrumEdition = astrumEditionSelector.getCurrent();

    LOG.info("Starting astrum with edition {}", astrumEdition.getShortName());

    warnIfDeprecatedAnalyticsPropertySet();

    // Allow the edition value below to resolve as a String when requested that way.
    ConfigurableConversionService conversionService = context.getEnvironment().getConversionService();
    conversionService.addConverter(AstrumEdition.class, String.class, AstrumEdition::getShortName);

    context
        .getEnvironment()
        .getPropertySources()
        .addFirst(new MapPropertySource(
            "application-launcher",
            Map.of("astrum.edition", astrumEdition)));
  }

  private void warnIfDeprecatedAnalyticsPropertySet() {
    String analyticsEnabled = astrumProperties.getProperty("astrum.analytics.enabled");
    if (analyticsEnabled != null) {
      LOG.warn("{} property is deprecated and has no effect. " +
          "Telemetry configuration is managed through license provisioning. " +
          "For opt-out provisioning, please contact GlobalArt Sales.",
          "astrum.analytics.enabled");
    }
  }

  /**
   * This method is called when the application context is refreshed. It will trigger the 2nd level component scanning
   * and start the jetty server.
   */
  @EventListener
  public void onContextRefreshed(final ContextRefreshedEvent event) {
    // we only want this event from the parent context to start jetty
    if (event.getApplicationContext().getParent() != null) {
      LOG.debug("Application already started, skipping event");
      return;
    }

    try {
      springComponentScan.finishBootstrapComponentScanning();
    }
    catch (Exception e) {
      throw new RuntimeException("Failed to configure application", e);
    }
  }
}
