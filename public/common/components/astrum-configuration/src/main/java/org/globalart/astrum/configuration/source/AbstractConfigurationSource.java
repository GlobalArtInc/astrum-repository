
package org.globalart.astrum.configuration.source;

import java.util.List;

import org.globalart.astrum.configuration.model.ApplyStatusXO;
import org.globalart.astrum.configuration.model.InstanceConfigurationXO;
import org.globalart.astrum.configuration.validation.MissingBlobStoreException;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static org.globalart.astrum.configuration.ApplyStatus.APPLIED;
import static org.globalart.astrum.configuration.ApplyStatus.FAILED;
import static org.globalart.astrum.configuration.ApplyStatus.NOT_APPLIED;

public abstract class AbstractConfigurationSource
    implements ConfigurationSource
{
  private static final Logger LOG = LoggerFactory.getLogger(AbstractConfigurationSource.class);

  @Override
  public final List<ApplyStatusXO> applyToInstance(final InstanceConfigurationXO instanceConfigurationXO) {
    try {
      return doApplyToInstance(instanceConfigurationXO);
    }
    catch (MissingBlobStoreException e) {
      throw e;
    }
    catch (RuntimeException e) {
      LOG.error("Failed to apply configuration for type: {}", getConfigurationTypeId(), e);
      return List.of(getFailedStatus(" " + e.getMessage()));
    }
    catch (Exception e) {
      LOG.error("Failed to apply configuration for type: {}", getConfigurationTypeId(), e);
      return List.of(getFailedStatus(" " + e.getMessage()));
    }
  }

  protected abstract List<ApplyStatusXO> doApplyToInstance(final InstanceConfigurationXO instanceConfigurationXO);

  protected ApplyStatusXO getAppliedStatus(final String message) {
    return ApplyStatusXO.from(getConfigurationTypeId(), "Applied" + message, APPLIED);
  }

  protected ApplyStatusXO getNotAppliedStatus(final String message) {
    return ApplyStatusXO.from(getConfigurationTypeId(), "Not applied" + message, NOT_APPLIED);
  }

  protected ApplyStatusXO getFailedStatus(final String message) {
    return ApplyStatusXO.from(getConfigurationTypeId(), "Failed to apply" + message, FAILED);
  }
}
