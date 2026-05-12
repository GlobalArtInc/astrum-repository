
package org.globalart.astrum.rapture.internal.state;

import java.time.Instant;
import java.time.temporal.ChronoUnit;
import java.util.Collections;
import java.util.Date;
import java.util.List;
import java.util.Map;
import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.app.ApplicationLicense;
import org.globalart.astrum.common.app.ApplicationLicense.Attributes;
import org.globalart.astrum.rapture.StateContributor;

import com.google.common.collect.ImmutableMap;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Contributes {@code license} state.
 *
 * @since 3.0
 */
@Component
public class LicenseStateContributor
    implements StateContributor
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String STATE_ID = "license";

  private final ApplicationLicense applicationLicense;

  @Autowired
  public LicenseStateContributor(final ApplicationLicense applicationLicense) {
    this.applicationLicense = checkNotNull(applicationLicense);
  }

  @Nullable
  @Override
  public Map<String, Object> getState() {
    return ImmutableMap.of(STATE_ID, calculateLicense());
  }

  private Object calculateLicense() {
    LicenseXO result = new LicenseXO();
    result.setRequired(applicationLicense.isRequired());
    result.setInstalled(applicationLicense.isInstalled());
    result.setValid(applicationLicense.isValid());
    Map<String, Object> attributes = applicationLicense.getAttributes();
    if (attributes != null && attributes.get(Attributes.EXPIRATION_DATE.getKey()) != null) {
      Date expirationDate = (Date) attributes.get(Attributes.EXPIRATION_DATE.getKey());
      result.setDaysToExpiry(Math.toIntExact(ChronoUnit.DAYS.between(Instant.now(), expirationDate.toInstant())));
    }
    if (attributes != null && attributes.get(Attributes.FEATURES.getKey()) != null) {
      // noinspection unchecked
      result.setFeatures((List<String>) attributes.get(Attributes.FEATURES.getKey()));
    }
    else {
      result.setFeatures(Collections.emptyList());
    }
    return result;
  }
}
