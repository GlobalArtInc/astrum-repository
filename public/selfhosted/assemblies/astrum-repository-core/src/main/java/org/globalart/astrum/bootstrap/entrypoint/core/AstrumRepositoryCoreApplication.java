
package org.globalart.astrum.bootstrap.entrypoint.core;

import org.globalart.astrum.bootstrap.entrypoint.configuration.AstrumDirectoryConfiguration;
import org.globalart.astrum.bootstrap.entrypoint.configuration.AstrumProperties;

import org.springframework.boot.WebApplicationType;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.boot.builder.SpringApplicationBuilder;
import org.springframework.context.ConfigurableApplicationContext;

import static org.springframework.boot.Banner.Mode.OFF;

// Minimalist list of root packages to scan, to start bootstrapping
@SpringBootApplication(
    scanBasePackages = {
        "org.globalart.astrum.bootstrap.entrypoint"
    })
public class AstrumRepositoryCoreApplication
{
  public static void main(final String[] args) {
    System.setProperty("spring.config.location", "etc/default-application.properties");
    // Ensure karaf.data is set as logback needs it early
    AstrumDirectoryConfiguration.load();
    // since logback is going to start on its own, we need to tell it where to find its configuration
    System.setProperty("logging.config", "etc/logback/logback.xml");

    // ASTRUM-47740: Disable the Spring logging shutdown hook is disabled
    // as it is abruptly terminating logging during shutdown
    // and thus preventing the logs from Astrum's shutdown sequence from being added to the log file
    System.setProperty("logging.register-shutdown-hook", "false");
    System.setProperty("astrum.edition", "CORE");

    new SpringApplicationBuilder(AstrumRepositoryCoreApplication.class)
        .bannerMode(OFF)
        .initializers(AstrumRepositoryCoreApplication::initialize)
        .web(WebApplicationType.NONE)
        .run(args);
  }

  private static void initialize(final ConfigurableApplicationContext applicationContext) {
    applicationContext.getEnvironment()
        .getPropertySources()
        .addFirst(new AstrumProperties());
  }
}
