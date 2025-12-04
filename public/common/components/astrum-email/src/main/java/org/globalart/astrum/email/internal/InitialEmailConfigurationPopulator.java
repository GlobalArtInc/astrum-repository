
package org.globalart.astrum.email.internal;

import java.util.function.Function;

import org.globalart.astrum.email.EmailConfiguration;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Initial {@link EmailConfiguration} populator.
 * Fills in the object with defaults
 *
 * @since 3.0
 */
@Component
@Qualifier("initial")
public class InitialEmailConfigurationPopulator
    implements Function<EmailConfiguration, EmailConfiguration>
{
  @Override
  public EmailConfiguration apply(final EmailConfiguration configuration) {
    configuration.setEnabled(false);
    configuration.setHost("localhost");
    configuration.setPort(25);
    configuration.setFromAddress("astrum@example.org");
    return configuration;
  }
}
